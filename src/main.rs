#![feature(box_syntax)]
#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::read_to_string;
use std::io::stdin;

use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use inkwell::module::Linkage;
use inkwell::OptimizationLevel;
use inkwell::values::BasicValueEnum;
use pest::Parser;

use crate::codegen::LLVMCodeGen;
use crate::eval::{create_default_ctx, run_program};
use crate::parser::{KalosParser, parse_expr, parse_program, parse_toplevel, Rule};

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
    let mut codegen = LLVMCodeGen::new(&context);

    let filename = std::env::args().nth(1).ok_or(MainError::ArgError)?;
    let input = read_to_string(filename)?;
    let parse = KalosParser::parse(Rule::program, &input)?;
    let program = parse_program(parse);
    codegen.compile_program(&program)?;

    /*
    let fn_type = context.i64_type().fn_type(&[], false);
    let main_fn = codegen.module.add_function("main", fn_type, None);
    let basic_block = context.append_basic_block(main_fn, "");
    codegen.builder.position_at_end(basic_block);
    let result: BasicValueEnum = codegen.compile_expr(&expr)?.try_into().unwrap();
    codegen.builder.build_return(Some(&result));
    codegen.module.print_to_stderr();
    let jit_main = unsafe {
        codegen.engine.get_function::<unsafe extern "C" fn() -> i64>("main")
    }?;

    let s = unsafe { jit_main.call() };
    println!("{}", s);
     */
    codegen.module.print_to_stderr();

    Ok(())
}
