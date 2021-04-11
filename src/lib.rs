#![feature(box_syntax)]
#![feature(c_variadic)]
#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fs::read_to_string;

use inkwell::context::Context;
use pest::Parser;

use crate::codegen::LLVMCodeGen;
use crate::execution::JITExecutionEngine;
use crate::parser::{KalosParser, parse_program, Rule};
use crate::runtime::DEFAULT_RUNTIME;

mod ast;
mod parser;
mod env;
mod codegen;
mod execution;
mod runtime;

pub fn run(filename: &str) -> anyhow::Result<()> {
    let input = read_to_string(filename)?;
    let parse = KalosParser::parse(Rule::program, &input)?;
    let program = parse_program(parse);

    let context = Context::create();
    let module = context.create_module("");
    let mut codegen = LLVMCodeGen::new(&context, &module);
    codegen.compile_program(&program)?;

    module.print_to_stderr();

    let engine = JITExecutionEngine::new(&module);
    engine.attach_runtime(&*DEFAULT_RUNTIME);
    let fn_main = engine.get_main();
    let s = unsafe { fn_main.call() };
    println!("main() returned with {}", s);

    Ok(())
}
