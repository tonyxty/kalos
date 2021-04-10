#![feature(box_syntax)]
#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::read_to_string;

use inkwell::context::Context;
use pest::Parser;

use crate::codegen::{LLVMCodeGen, KalosError};
use crate::parser::{KalosParser, parse_program, Rule};
use crate::library::{println, read_int};
use inkwell::values::AnyValueEnum;
use inkwell::types::{BasicTypeEnum, BasicType};

mod ast;
mod parser;
mod env;
mod codegen;
mod library;

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
