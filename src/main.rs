#![feature(box_syntax)]
#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fs::read_to_string;

use pest::Parser;

use crate::eval::{create_default_ctx, run_stmt};
use crate::parser::{KalosParser, parse_stmt, Rule};

mod ast;
mod parser;
mod eval;

fn main() {
    let filename = std::env::args().skip(1).next().unwrap();
    let source = read_to_string(&filename).unwrap();
    let parse_result = KalosParser::parse(Rule::stmt, &source).unwrap().next().unwrap();
    let stmt = parse_stmt(parse_result);
    let mut ctx = create_default_ctx();
    run_stmt(&mut ctx, &stmt);
}
