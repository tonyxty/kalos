use std::collections::HashMap;
use std::convert::TryInto;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::{Linkage, Module};
use inkwell::OptimizationLevel;
use inkwell::types::FunctionType;
use inkwell::values::{AnyValue, AnyValueEnum, BasicValueEnum, FunctionValue};

use crate::ast::{KalosBinOp, KalosExpr, KalosProgram, KalosPrototype, KalosToplevel, KalosTypeExpr, KalosStmt};
use crate::env::Env;
use crate::eval::KalosError;
use inkwell::passes::{PassManager, PassManagerBuilder};

pub struct LLVMCodeGen<'ctx> {
    context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub engine: ExecutionEngine<'ctx>,
    fpm: PassManager<FunctionValue<'ctx>>,
    env: Env<String, AnyValueEnum<'ctx>>,
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

        Self {
            context,
            module,
            builder,
            engine,
            fpm,
            env,
        }
    }

    fn compile_prototype(&self, prototype: &KalosPrototype) -> FunctionType<'ctx> {
        let i64_type = self.context.i64_type();
        let n = prototype.params.len();
        let param_types = vec![i64_type.into(); n];
        i64_type.fn_type(&param_types, false)
    }

    pub fn compile_expr(&self, expr: &KalosExpr) -> Result<AnyValueEnum<'ctx>, KalosError> {
        match expr {
            KalosExpr::Literal(x) =>
                Ok(self.context.i64_type().const_int(*x as u64, true).into()),
            KalosExpr::Call(func, args) => {
                let func = self.compile_expr(func)?.into_function_value();
                let args = args.iter().map(|e| self.compile_expr(e)
                    .and_then(|v| v.try_into().map_err(|_| KalosError::TypeError)))
                    .collect::<Result<Vec<BasicValueEnum>, KalosError>>()?;
                Ok(self.builder.build_call(func, &args[..], "").
                    try_as_basic_value().unwrap_left().as_any_value_enum())
            }
            KalosExpr::BinOp(op, x, y) => {
                let x = self.compile_expr(x)?.into_int_value();
                let y = self.compile_expr(y)?.into_int_value();
                Ok(match op {
                    KalosBinOp::Add => self.builder.build_int_add(x, y, ""),
                    KalosBinOp::Subtract => self.builder.build_int_sub(x, y, ""),
                    KalosBinOp::Multiply => self.builder.build_int_mul(x, y, ""),
                    KalosBinOp::Divide => self.builder.build_int_signed_div(x, y, ""),
                    KalosBinOp::Modulo => self.builder.build_int_signed_rem(x, y, ""),
                    KalosBinOp::Power => unimplemented!()
                }.as_any_value_enum())
            }
            KalosExpr::Identifier(name) =>
                self.env.get(name).copied().ok_or(KalosError::NameError),
        }
    }

    pub fn compile_stmt(&mut self, stmt: &KalosStmt) -> Result<(), KalosError> {
        println!("{:?}", stmt);
        match stmt {
            KalosStmt::Compound(s) => s.iter().try_for_each(|stmt| self.compile_stmt(stmt))?,
            KalosStmt::Return(expr) => {
                let expr: BasicValueEnum = self.compile_expr(expr)?.try_into().unwrap();
                self.builder.build_return(Some(&expr));
            },
            _ => todo!(),
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
                self.compile_stmt(body)?;
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
