#![feature(box_syntax)]
#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fs::read_to_string;

use pest::Parser;

use crate::eval::{create_default_ctx, run_program};
use crate::parser::{KalosParser, parse_toplevel, Rule};

mod ast;
mod parser;
mod eval;

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let source = read_to_string(&filename).unwrap();
    let parse_result = KalosParser::parse(Rule::program, &source).unwrap().next().unwrap();
    let program = parse_result.into_inner().map(parse_toplevel).collect();
    let mut ctx = create_default_ctx();
    if let Err(e) = run_program(&mut ctx, &program) {
        println!("{}", e);
    }
}
