use pest::iterators::{Pair, Pairs};
use pest::prec_climber;
use pest::prec_climber::PrecClimber;
use pest_derive::Parser;

use crate::ast::{KalosBuiltin::*, KalosExpr::{self, *}, KalosProgram, KalosSignature, KalosStmt, KalosToplevel, KalosType};

#[derive(Parser)]
#[grammar = "kalos.pest"]
pub struct KalosParser;

const PREC_CLIMBER: PrecClimber<Rule> = prec_climber![
    L   add | subtract,
    L   multiply | divide | modulo,
    R   power,
];

fn parse_identifier(id: Pair<Rule>) -> String {
    assert!(id.as_rule() == Rule::identifier);
    id.as_str().to_owned()
}

fn parse_atom(atom: Pair<Rule>) -> KalosExpr {
    match atom.as_rule() {
        Rule::literal => IntLiteral(atom.as_str().parse::<i64>().unwrap()),
        Rule::identifier => Identifier(parse_identifier(atom)),
        Rule::expr => parse_expr(atom),
        _ => unreachable!(),
    }
}

fn parse_type(type_expr: Pair<Rule>) -> KalosType {
    use KalosType::*;
    assert!(type_expr.as_rule() == Rule::type_expr);
    match type_expr.into_inner().next().unwrap().as_rule() {
        Rule::auto => Auto,
        Rule::int => Integer { signed: true, width: 64 },
        Rule::boolean => Bool,
        _ => unreachable!(),
    }
}

pub fn parse_expr(expr: Pair<Rule>) -> KalosExpr {
    assert!(expr.as_rule() == Rule::expr);
    PREC_CLIMBER.climb(
        expr.into_inner(),
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::call => {
                let mut parts = pair.into_inner();
                let func = box parse_atom(parts.next().unwrap());
                let args = parts.next().unwrap().into_inner().map(parse_expr).collect();
                Call { func, args }
            }
            _ => parse_atom(pair),
        },
        |lhs: KalosExpr, op: Pair<Rule>, rhs: KalosExpr| {
            let op = match op.as_rule() {
                Rule::add => Add,
                Rule::subtract => Subtract,
                Rule::multiply => Multiply,
                Rule::divide => Divide,
                Rule::modulo => Modulo,
                Rule::power => Power,
                _ => unreachable!(),
            };
            Builtin { builtin: op, args: vec![lhs, rhs] }
        },
    )
}

pub fn parse_stmt(stmt: Pair<Rule>) -> KalosStmt {
    match stmt.as_rule() {
        Rule::assignment_stmt => {
            let mut parts = stmt.into_inner();
            let lhs = parse_expr(parts.next().unwrap());
            let rhs = parse_expr(parts.next().unwrap());
            KalosStmt::Assignment { lhs, rhs }
        }
        Rule::compound_stmt => KalosStmt::Compound(stmt.into_inner().map(parse_stmt).collect()),
        Rule::var_stmt => {
            let mut parts = stmt.into_inner();
            let name = parse_identifier(parts.next().unwrap());
            let mut ty = KalosType::Auto;
            let mut initializer = None;
            parts.for_each(|p| match p.as_rule() {
                Rule::type_expr => ty = parse_type(p),
                Rule::expr => initializer = Some(parse_expr(p)),
                _ => unreachable!(),
            });
            KalosStmt::Var { name, ty, initializer }
        }
        Rule::return_stmt => KalosStmt::Return(stmt.into_inner().next().map(parse_expr)),
        Rule::if_stmt => {
            let mut parts = stmt.into_inner();
            let cond = parse_expr(parts.next().unwrap());
            let then_part = box parse_stmt(parts.next().unwrap());
            let else_part = parts.next().map(|p| box parse_stmt(p));
            KalosStmt::If { cond, then_part, else_part }
        }
        Rule::while_stmt => {
            let mut parts = stmt.into_inner();
            let cond = parse_expr(parts.next().unwrap());
            let body = box parse_stmt(parts.next().unwrap());
            KalosStmt::While { cond, body }
        }
        Rule::expr_stmt => KalosStmt::Expression(parse_expr(stmt.into_inner().next().unwrap())),
        _ => unreachable!(),
    }
}

fn parse_signature(signature: Pair<Rule>) -> KalosSignature {
    assert!(signature.as_rule() == Rule::signature);
    let mut parts = signature.into_inner();

    // FIXME: is there a more idiomatic way for this?  (fold works but makes little difference)
    let mut variadic = false;
    let mut params = Vec::new();
    for p in parts.next().unwrap().into_inner() {
        match p.as_rule() {
            Rule::param => {
                let mut parts = p.into_inner();
                let name = parse_identifier(parts.next().unwrap());
                let ty = parse_type(parts.next().unwrap());
                params.push((name, ty));
            }
            Rule::ellipsis => { variadic = true; }
            _ => unreachable!(),
        }
    }
    let return_type = box parts.next().map(parse_type).unwrap_or(KalosType::Unit);
    KalosSignature {
        params,
        return_type,
        variadic,
    }
}

pub fn parse_toplevel(t: Pair<Rule>) -> KalosToplevel {
    match t.as_rule() {
        Rule::def => {
            let mut parts = t.into_inner();
            let name = parts.next().unwrap().as_str().to_owned();
            let signature = parse_signature(parts.next().unwrap());
            let body = parts.next().map(parse_stmt);
            KalosToplevel::Def { name, signature, body }
        }
        _ => unreachable!(),
    }
}

pub fn parse_program(t: Pairs<Rule>) -> KalosProgram {
    let program = t.take_while(|p| p.as_rule() != Rule::EOI).map(parse_toplevel).collect();
    KalosProgram {
        program,
    }
}
