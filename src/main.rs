#![feature(box_syntax)]
#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::read_to_string;

use pest::Parser;

use crate::eval::{create_default_ctx, run_program};
use crate::parser::{KalosParser, parse_toplevel, Rule};

mod ast;
mod parser;
mod eval;

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
    let filename = std::env::args().nth(1).ok_or(MainError::ArgError)?;
    let source = read_to_string(&filename)?;
    let parse_result = KalosParser::parse(Rule::program, &source)?.next().unwrap();
    let program = parse_result.into_inner()
        .take_while(|p| p.as_rule() != Rule::EOI)
        .map(parse_toplevel).collect();
    let mut ctx = create_default_ctx();
    run_program(&mut ctx, &program)?;
    Ok(())
}
