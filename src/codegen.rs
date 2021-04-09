use std::convert::TryInto;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::OptimizationLevel;
use inkwell::values::{AnyValue, AnyValueEnum, BasicValueEnum, IntValue};

use crate::ast::{KalosBinOp, KalosExpr, KalosProgram};
use crate::env::Env;
use crate::eval::KalosError;
use std::collections::HashMap;

pub struct LLVMCodeGen<'ctx> {
    context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub engine: ExecutionEngine<'ctx>,
    env: Env<String, AnyValueEnum<'ctx>>,
}

impl<'ctx> LLVMCodeGen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("tmp");
        let builder = context.create_builder();
        let engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
        let env = Env::from(vec![HashMap::new()]);
        Self {
            context,
            module,
            builder,
            engine,
            env,
        }
    }

    pub fn codegen(&mut self, program: &KalosProgram) {}

    pub fn visit_expr(&self, expr: &KalosExpr) -> Result<AnyValueEnum<'ctx>, KalosError> {
        match expr {
            KalosExpr::Literal(x) =>
                Ok(self.context.i64_type().const_int(*x as u64, true).into()),
            KalosExpr::Call(func, args) => {
                let func = self.visit_expr(func)?.into_function_value();
                let args = args.iter().map(|e| self.visit_expr(e)
                    .and_then(|v| v.try_into().map_err(|_| KalosError::TypeError)))
                    .collect::<Result<Vec<BasicValueEnum>, KalosError>>()?;
                Ok(self.builder.build_call(func, &args[..], "").
                    try_as_basic_value().unwrap_left().as_any_value_enum())
            }
            KalosExpr::BinOp(op, x, y) => {
                let x = self.visit_expr(x)?.into_int_value();
                let y = self.visit_expr(y)?.into_int_value();
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
}