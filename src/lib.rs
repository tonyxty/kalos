#![feature(box_syntax)]
#![feature(c_variadic)]

use std::fs::read_to_string;
use std::io::Write;

use inkwell::context::Context;
use pest::Parser;

use crate::codegen::LLVMCodeGen;
use crate::execution::JITExecutionEngine;
use crate::parser::{KalosParser, parse_program, Rule};
pub use crate::runtime::DEFAULT_RUNTIME;
use crate::tyck::Tycker;

mod ast;
mod parser;
mod env;
mod tyck;
mod codegen;
mod execution;
mod runtime;

pub fn run<'a, T>(filename: &str, runtime: impl IntoIterator<Item=&'a (&'a T, usize)>)
    where T: 'a + ?Sized + AsRef<str>
{
    let input = read_to_string(filename).expect("some read thing failed");
    let parse = KalosParser::parse(Rule::program, &input).expect("some parse thing failed");
    let program = parse_program(parse);
    let mut tycker = Tycker::new();
    tycker.tyck_program(&program).expect("some type thing failed");

    let context = Context::create();
    let module = context.create_module("");
    let mut codegen = LLVMCodeGen::new(&context, &module);
    codegen.compile_program(&program).expect("some compile thing failed");

    {
        let stderr = std::io::stderr();
        let mut stderr = stderr.lock();
        writeln!(&mut stderr, "file: {}", filename).unwrap();
        for (name, ty) in tycker.get_globals() {
            writeln!(&mut stderr, "{}: {}", name, ty).unwrap();
        }
        module.print_to_stderr();
    }

    let engine = JITExecutionEngine::new(&module);
    engine.attach_runtime(runtime);
    let fn_main = engine.get_main();
    unsafe { fn_main.call() }
}
