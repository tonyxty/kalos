use pest::iterators::{Pair, Pairs};
use pest::prec_climber::PrecClimber;

use crate::ast::{KalosBinOp::*, KalosExpr::{self, *}, KalosProgram, KalosPrototype, KalosStmt, KalosToplevel, KalosType};

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

fn parse_type_expr(type_expr: Pair<Rule>) -> KalosType {
    assert!(type_expr.as_rule() == Rule::type_expr);
    KalosType::Auto
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
            BinOp { op, lhs: box lhs, rhs: box rhs }
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
                Rule::type_expr => ty = parse_type_expr(p),
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
            KalosToplevel::Def { prototype, body }
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
