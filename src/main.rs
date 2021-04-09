#![feature(box_syntax)]
#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::error::Error;
use std::fmt::{Display, Formatter, Debug};
use std::fs::read_to_string;

use pest::Parser;

use crate::eval::{create_default_ctx, run_program};
use crate::parser::{KalosParser, parse_toplevel, Rule, parse_program, parse_expr};
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::module::Linkage;
use inkwell::execution_engine::JitFunction;
use crate::codegen::LLVMCodeGen;
use std::io::stdin;
use std::convert::TryInto;
use inkwell::values::BasicValueEnum;

mod ast;
mod parser;
mod eval;
mod env;
mod codegen;

#[derive(Debug)]
enum MainError {
    ArgError,
}

impl Display for MainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("some arg thing failed")
    }
}

impl Error for MainError {}

fn main() -> anyhow::Result<()> {
    let context = Context::create();
    let codegen = LLVMCodeGen::new(&context);

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let expr = parse_expr(KalosParser::parse(Rule::expr, &input)?.next().unwrap());

    let fn_type = context.i64_type().fn_type(&[], false);
    let main_fn = codegen.module.add_function("main", fn_type, None);
    let basic_block = context.append_basic_block(main_fn, "");
    codegen.builder.position_at_end(basic_block);
    let result: BasicValueEnum = codegen.visit_expr(&expr)?.try_into().unwrap();
    codegen.builder.build_return(Some(&result));
    codegen.module.print_to_stderr();
    let jit_main = unsafe {
        codegen.engine.get_function::<unsafe extern "C" fn() -> i64>("main")
    }?;

    let s = unsafe { jit_main.call() };
    println!("{}", s);

    Ok(())
}
