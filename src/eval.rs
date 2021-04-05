use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::io::stdin;
use std::iter::FromIterator;

use crate::ast::KalosBinOp::{self, *};
use crate::ast::KalosExpr::{self, *};
use crate::ast::KalosStmt::{self, *};
use crate::ast::KalosToplevel;
use crate::eval::KalosError::*;
use crate::eval::KalosValue::*;

pub struct KalosCtx {
    globals: HashMap<String, KalosValue>,
    frames: Vec<HashMap<String, KalosValue>>,
    builtins: HashMap<String, fn(&[KalosValue]) -> KalosValue>,
    bin_ops: HashMap<KalosBinOp, fn(&KalosValue, &KalosValue) -> KalosValue>,
}

#[derive(Clone, Debug)]
pub enum KalosValue {
    Unit,
    Integer(i64),
    Function(Vec<String>, KalosStmt),
}

impl Display for KalosValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit => f.write_str("()"),
            Integer(x) => write!(f, "{}", x),
            Function(params, _) => {
                f.write_str("<function>(")?;
                f.write_str(&params.join(", "))?;
                f.write_char(')')
            }
        }
    }
}

impl KalosValue {
    fn is_true(&self) -> bool {
        match self {
            Unit => true,
            Integer(x) => *x != 0,
            Function(_, _) => true,
        }
    }

    fn unwrap_integer(&self) -> i64 {
        match self {
            Integer(x) => *x,
            _ => 0,
        }
    }
}

pub enum KalosError {
    NameError,
    TypeError,
}

impl Display for KalosError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NameError => f.write_str("NameError"),
            TypeError => f.write_str("TypeError"),
        }
    }
}

pub fn call_function(ctx: &mut KalosCtx, params: &Vec<String>, args: &Vec<KalosValue>,
                     body: &KalosStmt) -> Result<KalosValue, KalosError> {
    let local = HashMap::from_iter(Iterator::zip(params.iter(), args.iter())
        .map(|(k, v)| (k.clone(), v.clone()) ));
    ctx.frames.push(local);
    run_stmt(ctx, body)?;
    ctx.frames.pop();
    Ok(ctx.globals.remove("$").unwrap_or(Unit))
}

pub fn eval_expr(ctx: &mut KalosCtx, expr: &KalosExpr) -> Result<KalosValue, KalosError> {
    match expr {
        Literal(x) => Ok(Integer(*x)),
        Call(f, arg_exprs) => {
            let mut args = Vec::with_capacity(arg_exprs.len());
            for arg in arg_exprs {
                args.push(eval_expr(ctx, arg)?);
            }
            if let Identifier(name) = f.borrow() {
                if let Some(func) = ctx.globals.get(name).cloned() {
                    if let Function(params, body) = func {
                        call_function(ctx, &params, &args, &body)
                    } else { Err(TypeError) }
                } else if let Some(builtin) = ctx.builtins.get(name) {
                    Ok(builtin(args.as_slice()))
                } else { Err(NameError) }
            } else { Err(TypeError) }
        }
        BinOp(op, x, y) => {
            let op = ctx.bin_ops.get(op).ok_or(NameError)?;
            Ok(op(&eval_expr(ctx, x)?, &eval_expr(ctx, y)?))
        }
        Identifier(ident) => ctx.frames.last().unwrap().get(ident).cloned().ok_or(NameError),
    }
}

pub fn run_stmt(ctx: &mut KalosCtx, stmt: &KalosStmt) -> Result<(), KalosError> {
    match stmt {
        Compound(s) => {
            let t: Result<Vec<()>, KalosError> = s.iter().map(|s| run_stmt(ctx, s)).collect();
            t?;
        }
        Assignment(lhs, rhs) => {
            if let Identifier(name) = lhs {
                let val = eval_expr(ctx, rhs)?;
                ctx.frames.last_mut().unwrap().insert(name.to_owned(), val);
            } else { return Err(TypeError); }
        }
        Return(expr) => {
            let val = eval_expr(ctx, expr)?;
            ctx.globals.insert(String::from("$"), val);
        }
        If(cond, then_part, else_part) => if eval_expr(ctx, cond)?.is_true() {
            run_stmt(ctx, then_part)?;
        } else if let Some(else_part) = else_part {
            run_stmt(ctx, else_part)?;
        }
        While(cond, body) => while eval_expr(ctx, cond)?.is_true() {
            run_stmt(ctx, body)?;
        }
        Expression(expr) => { eval_expr(ctx, expr)?; }
    }
    Ok(())
}

pub fn run_program(ctx: &mut KalosCtx, program: &Vec<KalosToplevel>) -> Result<KalosValue, KalosError> {
    for t in program {
        match t {
            KalosToplevel::Def(name, params, body) => {
                ctx.globals.insert(name.to_owned(), KalosValue::Function(params.clone(), body.clone()));
            }
        }
    }
    if let Function(_, body) = ctx.globals.get("main").cloned().ok_or(NameError)? {
        call_function(ctx, &Vec::new(), &Vec::new(), &body)
    } else {
        Err(TypeError)
    }
}

pub fn create_default_ctx() -> KalosCtx {
    let mut ctx = KalosCtx {
        globals: HashMap::new(),
        frames: Vec::new(),
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
