use std::collections::HashMap;
use std::convert::TryInto;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::IntPredicate;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::types::FunctionType;
use inkwell::values::{AnyValueEnum, BasicValueEnum, FunctionValue, PointerValue};

use crate::ast::{KalosBuiltin, KalosExpr, KalosProgram, KalosSignature, KalosStmt, KalosToplevel, KalosError};
use crate::env::Env;

pub struct LLVMCodeGen<'ctx, 'm> {
    context: &'ctx Context,
    module: &'m Module<'ctx>,
    builder: Builder<'ctx>,
    fpm: PassManager<FunctionValue<'ctx>>,
    env: Env<String, AnyValueEnum<'ctx>>,
    current_fn: Option<FunctionValue<'ctx>>,
}

impl<'ctx, 'm> LLVMCodeGen<'ctx, 'm> {
    pub fn new(context: &'ctx Context, module: &'m Module<'ctx>) -> Self {
        let builder = context.create_builder();
        let env = Env::from(vec![HashMap::new()]);
        let fpm = PassManager::create(module);
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
            fpm,
            env,
            current_fn: None,
        }
    }
}

impl<'ctx> LLVMCodeGen<'ctx, '_> {
    fn new_block(&self) -> BasicBlock<'ctx> {
        self.context.append_basic_block(self.current_fn.unwrap(), "")
    }

    fn compile_signature(&self, prototype: &KalosSignature) -> FunctionType<'ctx> {
        let i64_type = self.context.i64_type();
        let n = prototype.params.len();
        let param_types = vec![i64_type.into(); n];
        i64_type.fn_type(&param_types, prototype.variadic)
    }

    pub fn compile_lvalue(&self, expr: &KalosExpr) -> Result<PointerValue<'ctx>, KalosError> {
        match expr {
            KalosExpr::Identifier(name) =>
                Ok(self.env.get(name).ok_or(KalosError::NameError)?.into_pointer_value()),
            _ => Err(KalosError::LvalueError),
        }
    }

    fn compile_builtin(&self, builtin: KalosBuiltin, args: &[KalosExpr]) -> Result<BasicValueEnum<'ctx>, KalosError> {
        use KalosBuiltin::*;
        let lhs = self.compile_expr(&args[0])?.into_int_value();
        let rhs = self.compile_expr(&args[1])?.into_int_value();
        Ok(match builtin {
            Add => self.builder.build_int_add(lhs, rhs, ""),
            Subtract => self.builder.build_int_sub(lhs, rhs, ""),
            Multiply => self.builder.build_int_mul(lhs, rhs, ""),
            Divide => self.builder.build_int_signed_div(lhs, rhs, ""),
            Modulo => self.builder.build_int_signed_rem(lhs, rhs, ""),
            Power => unimplemented!()
        }.into())
    }

    pub fn compile_expr(&self, expr: &KalosExpr) -> Result<AnyValueEnum<'ctx>, KalosError> {
        use KalosExpr::*;
        Ok(match expr {
            IntLiteral(x) => self.context.i64_type().const_int(*x as u64, true).into(),
            BoolLiteral(x) => self.context.bool_type().const_int(*x as u64, false).into(),
            StringLiteral(x) => todo!(),
            Call { func, args } => {
                let func = self.compile_expr(func)?.into_function_value();
                let args = args.iter().map(|e| self.compile_expr(e)
                    .map(|v| v.try_into().unwrap()))
                    .collect::<Result<Vec<BasicValueEnum>, KalosError>>()?;
                self.builder.build_call(func, &args[..], "").
                    try_as_basic_value().unwrap_left().into()
            }
            Builtin { builtin, args } => self.compile_builtin(*builtin, args)?.into(),
            Identifier(name) => {
                let var = self.env.get(name).copied().ok_or(KalosError::NameError)?;
                if var.is_pointer_value() {
                    self.builder.build_load(var.into_pointer_value(), "").into()
                } else {
                    var
                }
            }
        })
    }

    pub fn compile_stmt(&mut self, stmt: &KalosStmt) -> Result<(), KalosError> {
        use KalosStmt::*;
        match stmt {
            Compound(s) => {
                self.env.push_empty();
                s.iter().try_for_each(|stmt| self.compile_stmt(stmt))?;
                self.env.pop();
            }
            Assignment { lhs, rhs } => {
                let lhs = self.compile_lvalue(lhs)?;
                let rhs: BasicValueEnum = self.compile_expr(rhs)?.try_into().unwrap();
                self.builder.build_store(lhs, rhs);
            }
            Var { name, ty: _ty, initializer } => {
                let var = self.builder.build_alloca(self.context.i64_type(), name);
                self.env.put(name.clone(), var.into());
                if let Some(initializer) = initializer {
                    let init_val: BasicValueEnum = self.compile_expr(initializer)?.try_into().unwrap();
                    self.builder.build_store(var, init_val);
                }
            }
            Return(expr) => {
                if let Some(expr) = expr {
                    let expr_value: BasicValueEnum = self.compile_expr(expr)?.try_into().unwrap();
                    self.builder.build_return(Some(&expr_value));
                } else {
                    self.builder.build_return(None);
                }
            }
            If { cond, then_part, else_part } => {
                let cond_value = self.compile_expr(cond)?.into_int_value();
                let cond_value = self.builder.build_int_compare(
                    IntPredicate::NE, cond_value, self.context.i64_type().const_zero(), "");
                let then_block = self.new_block();
                let else_block = self.new_block();
                let cont_block = self.new_block();
                self.builder.build_conditional_branch(cond_value, then_block, else_block);
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
            While { cond, body } => {
                let cond_value = self.compile_expr(cond)?.into_int_value();
                let cond_value = self.builder.build_int_compare(
                    IntPredicate::NE, cond_value, self.context.i64_type().const_zero(), "");
                let loop_block = self.new_block();
                let cont_block = self.new_block();
                self.builder.build_conditional_branch(cond_value, loop_block, cont_block);
                self.builder.position_at_end(loop_block);
                self.compile_stmt(body)?;
                let cond_value_recheck = self.compile_expr(cond)?.into_int_value();
                let cond_value_recheck = self.builder.build_int_compare(
                    IntPredicate::NE, cond_value_recheck, self.context.i64_type().const_zero(), "");
                self.builder.build_conditional_branch(cond_value_recheck, loop_block, cont_block);
                self.builder.position_at_end(cont_block);
            }
            Expression(expr) => {
                self.compile_expr(expr)?;
            }
        }
        Ok(())
    }

    pub fn compile_toplevel(&mut self, toplevel: &KalosToplevel)
                            -> Result<FunctionValue<'ctx>, KalosError> {
        match toplevel {
            KalosToplevel::Def { name, signature, body } => {
                let fn_type = self.compile_signature(signature);
                let func = self.module.add_function(name, fn_type, None);
                self.env.put(name.clone(), func.into());
                if let Some(body) = body {
                    self.env.push(signature.params.iter()
                        .zip(func.get_param_iter())
                        .map(|((name, _), param)| (name.clone(), param.into()))
                        .collect());
                    let block = self.context.append_basic_block(func, "");
                    self.builder.position_at_end(block);
                    self.current_fn = Some(func);
                    self.compile_stmt(body)?;
                    self.current_fn = None;
                    assert!(func.verify(true));
                    self.fpm.run_on(&func);
                    self.env.pop();
                }
                Ok(func)
            }
        }
    }

    pub fn compile_program(&mut self, program: &KalosProgram) -> Result<(), KalosError> {
        program.program.iter().try_for_each(|t| self.compile_toplevel(t).map(|_| ()))
    }
}
