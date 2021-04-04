use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};

use crate::ast::{KalosBinOp, KalosExpr, KalosStmt};

#[derive(Parser)]
#[grammar = "kalos.pest"]
pub struct KalosParser;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Assoc::*;
        use Rule::*;
        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left) |
                Operator::new(modulo, Left),
            Operator::new(power, Right),
        ])
    };
}

pub fn parse_expr(expr: Pairs<Rule>) -> KalosExpr {
    PREC_CLIMBER.climb(
        expr,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::call => {
                let mut parts = pair.into_inner();
                let func = parts.next().unwrap();
                let params = parts.next().unwrap().into_inner();
                KalosExpr::Call(box parse_expr(Pairs::single(func)),
                                params.map(|p| parse_expr(p.into_inner())).collect())
            }
            Rule::literal => KalosExpr::Literal(pair.as_str().parse::<i64>().unwrap()),
            Rule::identifier => KalosExpr::Identifier(pair.as_str().to_owned()),
            Rule::atom | Rule::expr => parse_expr(pair.into_inner()),
            _ => {
                println!("{:?}", pair);
                unreachable!()
            }
        },
        |lhs: KalosExpr, op: Pair<Rule>, rhs: KalosExpr| match op.as_rule() {
            Rule::add => KalosExpr::BinOp(KalosBinOp::Add, box lhs, box rhs),
            Rule::subtract => KalosExpr::BinOp(KalosBinOp::Subtract, box lhs, box rhs),
            Rule::multiply => KalosExpr::BinOp(KalosBinOp::Multiply, box lhs, box rhs),
            Rule::divide => KalosExpr::BinOp(KalosBinOp::Divide, box lhs, box rhs),
            Rule::modulo => KalosExpr::BinOp(KalosBinOp::Modulo, box lhs, box rhs),
            Rule::power => KalosExpr::BinOp(KalosBinOp::Power, box lhs, box rhs),
            _ => unreachable!(),
        },
    )
}

pub fn parse_stmt(stmt: Pair<Rule>) -> KalosStmt {
    match stmt.as_rule() {
        Rule::stmt => parse_stmt(stmt.into_inner().next().unwrap()),
        Rule::assignment_stmt => {
            let mut parts = stmt.into_inner();
            let assignee = parts.next().unwrap();
            let expr = parts.next().unwrap();
            KalosStmt::Assignment(parse_expr(Pairs::single(assignee)),
                                  parse_expr(Pairs::single(expr)))
        }
        Rule::compound_stmt => KalosStmt::Compound(stmt.into_inner().map(parse_stmt).collect()),
        Rule::if_stmt => {
            let mut parts = stmt.into_inner();
            let expr = parts.next().unwrap();
            let then_body = parts.next().unwrap();
            let else_body = parts.next().map(|p| box parse_stmt(p));
            KalosStmt::If(parse_expr(Pairs::single(expr)), box parse_stmt(then_body), else_body)
        }
        Rule::while_stmt => {
            let mut parts = stmt.into_inner();
            let expr = parts.next().unwrap();
            let body = parts.next().unwrap();
            KalosStmt::While(parse_expr(Pairs::single(expr)), box parse_stmt(body))
        }
        Rule::expr_stmt => KalosStmt::Expression(parse_expr(stmt.into_inner())),
        _ => {
            println!("{:?}", stmt);
            unreachable!()
        }
    }
}
