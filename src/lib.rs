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
use crate::library::{println, read_int};
use crate::parser::{KalosParser, parse_program, Rule};

mod ast;
mod parser;
mod env;
mod codegen;
mod library;

pub fn run(filename: &str) -> anyhow::Result<()> {
    let context = Context::create();
    let mut codegen = LLVMCodeGen::new(&context);

    let input = read_to_string(filename)?;
    let parse = KalosParser::parse(Rule::program, &input)?;
    let program = parse_program(parse);
    codegen.compile_program(&program)?;

    codegen.module.print_to_stderr();

    codegen.add_fn("println", println as usize);
    codegen.add_fn("read_int", read_int as usize);

    let jit_main = unsafe {
        codegen.engine.get_function::<unsafe extern "C" fn() -> i64>("main")
    }?;

    let s = unsafe { jit_main.call() };
    println!("main() returned with {}", s);

    Ok(())
}
