use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::stdin;

use crate::ast::KalosBinOp::{self, *};
use crate::ast::KalosExpr::{self, *};
use crate::ast::KalosStmt::{self, *};
use crate::eval::KalosValue::*;

pub struct KalosCtx {
    vars: HashMap<String, KalosValue>,
    builtins: HashMap<String, fn(&[KalosValue]) -> KalosValue>,
    bin_ops: HashMap<KalosBinOp, fn(&KalosValue, &KalosValue) -> KalosValue>,
}

#[derive(Copy, Clone, Debug)]
pub enum KalosValue {
    Integer(i64),
    Unit,
}

impl Display for KalosValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Integer(x) => write!(f, "{}", x),
            Unit => f.write_str("()"),
        }
    }
}

impl KalosValue {
    fn is_true(&self) -> bool {
        match self {
            Integer(x) => *x != 0,
            Unit => true,
        }
    }

    fn unwrap_integer(&self) -> i64 {
        match self {
            Integer(x) => *x,
            _ => 0,
        }
    }
}

pub fn eval_expr(ctx: &mut KalosCtx, expr: &KalosExpr) -> Option<KalosValue> {
    match expr {
        Literal(x) => Some(Integer(*x)),
        Call(f, arg_exprs) => {
            let mut args = Vec::with_capacity(arg_exprs.len());
            for arg in arg_exprs {
                args.push(eval_expr(ctx, arg)?);
            }
            let func = if let Identifier(name) = f.borrow() {
                ctx.builtins.get(name)
            } else { None }?;
            Some(func(args.as_slice()))
        }
        BinOp(op, x, y) => {
            let op = ctx.bin_ops.get(op)?;
            Some(op(&eval_expr(ctx, x)?, &eval_expr(ctx, y)?))
        }
        Identifier(ident) => ctx.vars.get(ident).copied(),
    }
}

pub fn run_stmt(ctx: &mut KalosCtx, stmt: &KalosStmt) -> Option<()> {
    match stmt {
        Compound(s) => Some({
            let t: Option<Vec<()>> = s.iter().map(|s| run_stmt(ctx, s)).collect();
            t?;
        }),
        Assignment(lhs, rhs) => {
            if let Identifier(name) = lhs {
                let val = eval_expr(ctx, rhs)?;
                ctx.vars.insert(name.to_owned(), val);
                Some(())
            } else { None }
        }
        If(cond, then_part, else_part) => if eval_expr(ctx, cond)?.is_true() {
            run_stmt(ctx, then_part)
        } else if let Some(else_part) = else_part {
            run_stmt(ctx, else_part)
        } else {
            Some(())
        }
        While(cond, body) => Some(while eval_expr(ctx, cond)?.is_true() {
            run_stmt(ctx, body)?
        }),
        Expression(expr) => Some({ eval_expr(ctx, expr)?; })
    }
}

pub fn create_default_ctx() -> KalosCtx {
    let mut ctx = KalosCtx {
        vars: HashMap::new(),
        builtins: HashMap::new(),
        bin_ops: HashMap::new(),
    };

    ctx.builtins.insert(String::from("println"), |args| {
        println!("{}", args.iter()
            .map(KalosValue::to_string)
            .reduce(|a, b| a + " " + b.as_str())
            .unwrap_or_default());
        Unit
    });
    ctx.builtins.insert(String::from("read_int"), |_| {
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        Integer(line.trim().parse().unwrap())
    });

    ctx.bin_ops.insert(Add, |x, y|
        Integer(x.unwrap_integer() + y.unwrap_integer()));
    ctx.bin_ops.insert(Subtract, |x, y|
        Integer(x.unwrap_integer() - y.unwrap_integer()));
    ctx.bin_ops.insert(Multiply, |x, y|
        Integer(x.unwrap_integer() * y.unwrap_integer()));
    ctx.bin_ops.insert(Divide, |x, y|
        Integer(x.unwrap_integer() / y.unwrap_integer()));
    ctx.bin_ops.insert(Power, |x, y|
        Integer(x.unwrap_integer().pow(y.unwrap_integer() as u32)));
    ctx.bin_ops.insert(Modulo, |x, y|
        Integer(x.unwrap_integer() % y.unwrap_integer()));

    ctx
}
