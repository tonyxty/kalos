use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Display, Formatter};

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::{Linkage, Module};
use inkwell::OptimizationLevel;
use inkwell::passes::PassManager;
use inkwell::types::FunctionType;
use inkwell::values::{AnyValue, AnyValueEnum, BasicValueEnum, FunctionValue, PointerValue};

use crate::ast::{KalosBinOp, KalosExpr, KalosProgram, KalosPrototype, KalosStmt, KalosToplevel, KalosTypeExpr};
use crate::env::Env;
use inkwell::basic_block::BasicBlock;

#[derive(Debug)]
pub enum KalosError {
    NameError,
    TypeError,
    LvalueError,
}

impl Display for KalosError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NameError => f.write_str("NameError"),
            TypeError => f.write_str("TypeError"),
            LvalueError => f.write_str("LvalueError"),
        }
    }
}

impl Error for KalosError {}

pub struct LLVMCodeGen<'ctx> {
    context: &'ctx Context,
    pub module: Module<'ctx>,
    builder: Builder<'ctx>,
    engine: ExecutionEngine<'ctx>,
    fpm: PassManager<FunctionValue<'ctx>>,
    env: Env<String, AnyValueEnum<'ctx>>,
    current_fn: Option<FunctionValue<'ctx>>,
}

impl<'ctx> LLVMCodeGen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("tmp");
        let builder = context.create_builder();
        let engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
        let env = Env::from(vec![HashMap::new()]);
        let fpm = PassManager::create(&module);
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_gvn_pass();
        fpm.add_cfg_simplification_pass();
        fpm.add_basic_alias_analysis_pass();
        fpm.add_promote_memory_to_register_pass();
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.initialize();

        Self {
            context,
            module,
            builder,
            engine,
            fpm,
            env,
            current_fn: None,
        }
    }

    fn new_block(&self) -> BasicBlock<'ctx> {
        self.context.append_basic_block(self.current_fn.unwrap(), "")
    }

    fn compile_prototype(&self, prototype: &KalosPrototype) -> FunctionType<'ctx> {
        let i64_type = self.context.i64_type();
        let n = prototype.params.len();
        let param_types = vec![i64_type.into(); n];
        i64_type.fn_type(&param_types, false)
    }

    pub fn compile_lvalue(&self, expr: &KalosExpr) -> Result<PointerValue<'ctx>, KalosError> {
        match expr {
            KalosExpr::Identifier(name) =>
                Ok(self.env.get(name).ok_or(KalosError::NameError)?.into_pointer_value()),
            _ => Err(KalosError::LvalueError),
        }
    }

    pub fn compile_expr(&self, expr: &KalosExpr) -> Result<AnyValueEnum<'ctx>, KalosError> {
        Ok(match expr {
            KalosExpr::Literal(x) =>
                self.context.i64_type().const_int(*x as u64, true).into(),
            KalosExpr::Call(func, args) => {
                let func = self.compile_expr(func)?.into_function_value();
                let args = args.iter().map(|e| self.compile_expr(e)
                    .and_then(|v| v.try_into().map_err(|_| KalosError::TypeError)))
                    .collect::<Result<Vec<BasicValueEnum>, KalosError>>()?;
                self.builder.build_call(func, &args[..], "").
                    try_as_basic_value().unwrap_left()
            }
            KalosExpr::BinOp(op, x, y) => {
                let x = self.compile_expr(x)?.into_int_value();
                let y = self.compile_expr(y)?.into_int_value();
                match op {
                    KalosBinOp::Add => self.builder.build_int_add(x, y, ""),
                    KalosBinOp::Subtract => self.builder.build_int_sub(x, y, ""),
                    KalosBinOp::Multiply => self.builder.build_int_mul(x, y, ""),
                    KalosBinOp::Divide => self.builder.build_int_signed_div(x, y, ""),
                    KalosBinOp::Modulo => self.builder.build_int_signed_rem(x, y, ""),
                    KalosBinOp::Power => unimplemented!()
                }.into()
            }
            KalosExpr::Identifier(name) => {
                let var = self.env.get(name).copied().ok_or(KalosError::NameError)?;
                if var.is_pointer_value() {
                    self.builder.build_load(var.into_pointer_value(), "")
                } else {
                    var.try_into().unwrap()
                }
            }
        }.as_any_value_enum())
    }

    pub fn compile_stmt(&mut self, stmt: &KalosStmt) -> Result<(), KalosError> {
        println!("{:?}", stmt);
        match stmt {
            KalosStmt::Compound(s) => s.iter().try_for_each(|stmt| self.compile_stmt(stmt))?,
            KalosStmt::Assignment(lhs, rhs) => {
                let lhs = self.compile_lvalue(lhs)?;
                let rhs: BasicValueEnum = self.compile_expr(rhs)?.try_into().unwrap();
                self.builder.build_store(lhs, rhs);
            }
            KalosStmt::Var(name, _, initializer) => {
                let var = self.builder.build_alloca(self.context.i64_type(), name);
                self.env.put(name.clone(), var.into());
                if let Some(initializer) = initializer {
                    let init_val: BasicValueEnum = self.compile_expr(initializer)?.try_into().unwrap();
                    self.builder.build_store(var, init_val);
                }
            }
            KalosStmt::Return(expr) => {
                let expr: BasicValueEnum = self.compile_expr(expr)?.try_into().unwrap();
                self.builder.build_return(Some(&expr));
            }
            KalosStmt::If(cond, then_part, else_part) => {
                let then_block = self.new_block();
                let else_block = self.new_block();
                let cont_block = self.new_block();
                self.builder.build_conditional_branch(self.compile_expr(cond)?.into_int_value(),
                                                      then_block, else_block);
                self.builder.position_at_end(then_block);
                self.compile_stmt(then_part)?;
                self.builder.build_unconditional_branch(cont_block);
                self.builder.position_at_end(else_block);
                if let Some(else_part) = else_part {
                    self.compile_stmt(else_part)?;
                }
                self.builder.build_unconditional_branch(cont_block);
                self.builder.position_at_end(cont_block);
            }
            KalosStmt::While(_, _) => {}
            KalosStmt::Expression(_) => {}
        }
        Ok(())
    }

    pub fn compile_toplevel(&mut self, toplevel: &KalosToplevel)
                            -> Result<FunctionValue<'ctx>, KalosError> {
        match toplevel {
            KalosToplevel::Def(prototype, body) => {
                let fn_type = self.compile_prototype(prototype);
                let func = self.module.add_function(&prototype.name, fn_type, None);
                self.env.put(prototype.name.clone(), func.into());
                self.env.push(prototype.params.iter()
                    .zip(func.get_param_iter())
                    .map(|(name, param)| (name.clone(), param.into()))
                    .collect());
                let basic_block = self.context.append_basic_block(func, "");
                self.builder.position_at_end(basic_block);
                self.current_fn = Some(func);
                self.compile_stmt(body)?;
                self.current_fn = None;
                self.env.pop();
                Ok(func)
            }
            KalosToplevel::Extern(prototype) => {
                let fn_type = self.compile_prototype(prototype);
                let func = self.module.add_function(&prototype.name, fn_type, Some(Linkage::External));
                self.env.put(prototype.name.clone(), func.into());
                Ok(func)
            }
        }
    }

    pub fn compile_program(&mut self, program: &KalosProgram) -> Result<(), KalosError> {
        program.program.iter().try_for_each(|t| self.compile_toplevel(t).map(|_| ()))
    }
}
