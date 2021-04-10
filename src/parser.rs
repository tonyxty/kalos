use pest::iterators::{Pair, Pairs};
use pest::prec_climber::PrecClimber;

use crate::ast::{KalosBinOp::*, KalosExpr::{self, *}, KalosProgram, KalosPrototype, KalosStmt, KalosToplevel, KalosTypeExpr};

#[derive(Parser)]
#[grammar = "kalos.pest"]
pub struct KalosParser;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use pest::prec_climber::{Assoc::*, Operator};
        use Rule::*;
        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left) |
                Operator::new(modulo, Left),
            Operator::new(power, Right),
        ])
    };
}

fn parse_identifier(id: Pair<Rule>) -> String {
    assert!(id.as_rule() == Rule::identifier);
    id.as_str().to_owned()
}

fn parse_atom(atom: Pair<Rule>) -> KalosExpr {
    match atom.as_rule() {
        Rule::literal => Literal(atom.as_str().parse::<i64>().unwrap()),
        Rule::identifier => Identifier(parse_identifier(atom)),
        Rule::expr => parse_expr(atom),
        _ => unreachable!(),
    }
}

fn parse_type_expr(type_expr: Pair<Rule>) -> KalosTypeExpr {
    assert!(type_expr.as_rule() == Rule::type_expr);
    KalosTypeExpr::Auto
}

pub fn parse_expr(expr: Pair<Rule>) -> KalosExpr {
    assert!(expr.as_rule() == Rule::expr);
    PREC_CLIMBER.climb(
        expr.into_inner(),
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::call => {
                let mut parts = pair.into_inner();
                let func = box parse_atom(parts.next().unwrap());
                let params = parts.next().unwrap().into_inner().map(parse_expr).collect();
                Call(func, params)
            }
            _ => parse_atom(pair),
        },
        |lhs: KalosExpr, op: Pair<Rule>, rhs: KalosExpr| match op.as_rule() {
            Rule::add => BinOp(Add, box lhs, box rhs),
            Rule::subtract => BinOp(Subtract, box lhs, box rhs),
            Rule::multiply => BinOp(Multiply, box lhs, box rhs),
            Rule::divide => BinOp(Divide, box lhs, box rhs),
            Rule::modulo => BinOp(Modulo, box lhs, box rhs),
            Rule::power => BinOp(Power, box lhs, box rhs),
            _ => unreachable!(),
        },
    )
}

pub fn parse_stmt(stmt: Pair<Rule>) -> KalosStmt {
    match stmt.as_rule() {
        Rule::assignment_stmt => {
            let mut parts = stmt.into_inner();
            let lvalue = parse_expr(parts.next().unwrap());
            let expr = parse_expr(parts.next().unwrap());
            KalosStmt::Assignment(lvalue, expr)
        }
        Rule::compound_stmt => KalosStmt::Compound(stmt.into_inner().map(parse_stmt).collect()),
        Rule::var_stmt => {
            let mut parts = stmt.into_inner();
            let name = parse_identifier(parts.next().unwrap());
            let mut type_annotation = KalosTypeExpr::Auto;
            let mut initializer = None;
            parts.for_each(|p| match p.as_rule() {
                Rule::type_expr => type_annotation = parse_type_expr(p),
                Rule::expr => initializer = Some(parse_expr(p)),
                _ => unreachable!(),
            });
            KalosStmt::Var(name, type_annotation, initializer)
        }
        Rule::return_stmt => KalosStmt::Return(stmt.into_inner().next().map(parse_expr)),
        Rule::if_stmt => {
            let mut parts = stmt.into_inner();
            let expr = parse_expr(parts.next().unwrap());
            let then_body = box parse_stmt(parts.next().unwrap());
            let else_body = parts.next().map(|p| box parse_stmt(p));
            KalosStmt::If(expr, then_body, else_body)
        }
        Rule::while_stmt => {
            let mut parts = stmt.into_inner();
            let expr = parse_expr(parts.next().unwrap());
            let body = box parse_stmt(parts.next().unwrap());
            KalosStmt::While(expr, body)
        }
        Rule::expr_stmt => KalosStmt::Expression(parse_expr(stmt.into_inner().next().unwrap())),
        _ => unreachable!(),
    }
}

fn parse_prototype(prototype: Pair<Rule>) -> KalosPrototype {
    assert!(prototype.as_rule() == Rule::prototype);
    let mut parts = prototype.into_inner();
    let name = parse_identifier(parts.next().unwrap());
    let params = parts.next().unwrap().into_inner().map(parse_identifier).collect();
    let return_type = parts.next().map(parse_type_expr);
    KalosPrototype {
        name,
        params,
        return_type,
    }
}

pub fn parse_toplevel(t: Pair<Rule>) -> KalosToplevel {
    match t.as_rule() {
        Rule::def => {
            let mut parts = t.into_inner();
            let prototype = parse_prototype(parts.next().unwrap());
            let body = parse_stmt(parts.next().unwrap());
            KalosToplevel::Def(prototype, body)
        }
        Rule::extern_stmt =>
            KalosToplevel::Extern(parse_prototype(t.into_inner().next().unwrap())),
        _ => unreachable!(),
    }
}

pub fn parse_program(t: Pairs<Rule>) -> KalosProgram {
    let program = t.take_while(|p| p.as_rule() != Rule::EOI)
        .map(parse_toplevel).collect();
    KalosProgram {
        program,
    }
}
