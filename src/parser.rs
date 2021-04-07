use pest::iterators::Pair;
use pest::prec_climber::PrecClimber;

use crate::ast::{KalosBinOp::*, KalosExpr::{self, *}, KalosStmt, KalosToplevel};

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
    assert!(atom.as_rule() == Rule::atom);
    let atom = atom.into_inner().next().unwrap();
    match atom.as_rule() {
        Rule::literal => Literal(atom.as_str().parse::<i64>().unwrap()),
        Rule::identifier => Identifier(parse_identifier(atom)),
        Rule::expr => parse_expr(atom),
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
                let func = parts.next().unwrap();
                let params = parts.next().unwrap().into_inner();
                Call(box parse_atom(func), params.map(|p| parse_expr(p)).collect())
            }
            Rule::atom => parse_atom(pair),
            _ => unreachable!(),
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
    assert!(stmt.as_rule() == Rule::stmt);
    let stmt = stmt.into_inner().next().unwrap();
    match stmt.as_rule() {
        Rule::assignment_stmt => {
            let mut parts = stmt.into_inner();
            let lvalue = parts.next().unwrap();
            let expr = parts.next().unwrap();
            KalosStmt::Assignment(parse_expr(lvalue), parse_expr(expr))
        }
        Rule::compound_stmt => KalosStmt::Compound(stmt.into_inner().map(parse_stmt).collect()),
        Rule::return_stmt => KalosStmt::Return(parse_expr(stmt.into_inner().next().unwrap())),
        Rule::if_stmt => {
            let mut parts = stmt.into_inner();
            let expr = parts.next().unwrap();
            let then_body = parts.next().unwrap();
            let else_body = parts.next().map(|p| box parse_stmt(p));
            KalosStmt::If(parse_expr(expr), box parse_stmt(then_body), else_body)
        }
        Rule::while_stmt => {
            let mut parts = stmt.into_inner();
            let expr = parts.next().unwrap();
            let body = parts.next().unwrap();
            KalosStmt::While(parse_expr(expr), box parse_stmt(body))
        }
        Rule::expr_stmt => KalosStmt::Expression(parse_expr(stmt.into_inner().next().unwrap())),
        _ => unreachable!(),
    }
}

pub fn parse_toplevel(t: Pair<Rule>) -> KalosToplevel {
    assert!(t.as_rule() == Rule::toplevel);
    let t = t.into_inner().next().unwrap();
    match t.as_rule() {
        Rule::def => {
            let mut parts = t.into_inner();
            let name = parse_identifier(parts.next().unwrap());
            let param_list: Vec<String> = parts.next().unwrap().into_inner()
                .map(|p| p.as_str().to_owned()).collect();
            let body = parse_stmt(parts.next().unwrap());
            KalosToplevel::Def(name, param_list, body)
        }
        _ => unreachable!()
    }
}
