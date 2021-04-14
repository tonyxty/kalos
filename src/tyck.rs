use crate::env::Env;
use crate::ast::{KalosType, KalosExpr, KalosError, KalosProgram, KalosStmt, KalosToplevel};
use std::collections::HashMap;

pub struct Tycker {
    env: Env<String, KalosType>,
}

impl Tycker {
    pub fn new() -> Self {
        Self { env: Env::from(vec![HashMap::new()]) }
    }

    pub fn tyck_expr(&self, expr: &KalosExpr) -> Result<KalosType, KalosError> {
        use KalosType::*;
        match expr {
            KalosExpr::IntLiteral(_) => Ok(Integer { signed: true, width: 64 }),
            KalosExpr::BoolLiteral(_) => Ok(Bool),
            KalosExpr::StringLiteral(_) => Ok(Text),
            KalosExpr::Call { func, args } => {
                let ty = self.tyck_expr(func)?;
                if let Function { signature } = ty {
                    let n = signature.params.len();
                    if args.len() == n || (signature.variadic && args.len() > n) {
                        signature.params.iter().zip(args).try_for_each(|((_, ty), arg)| {
                            let arg_type = self.tyck_expr(arg)?;
                            ty.try_unify(&arg_type).map(|_| ())
                        })?;
                        Ok(*signature.return_type)
                    } else {
                        Err(KalosError::ArgError)
                    }
                } else {
                    Err(KalosError::TypeError { expect: None, found: Some(ty) })
                }
            }
            KalosExpr::Builtin { builtin, args } =>
                self.tyck_expr(&args[0]),
            KalosExpr::Identifier(name) =>
                Ok(self.env.get(name).ok_or(KalosError::NameError)?.to_owned()),
        }
    }

    pub fn tyck_stmt(&mut self, stmt: &KalosStmt) -> Result<(), KalosError> {
        match stmt {
            KalosStmt::Compound(s) => {
                self.env.push_empty();
                s.iter().try_for_each(|stmt| self.tyck_stmt(stmt))?;
                self.env.pop();
            }
            KalosStmt::Assignment { lhs, rhs } => {
                let lhs_type = self.tyck_expr(lhs)?;
                let rhs_type = self.tyck_expr(rhs)?;
                lhs_type.try_unify(&rhs_type)?;
            }
            KalosStmt::Var { name, ty, initializer } => {
                let ty = if let Some(initializer) = initializer {
                    let init_ty = self.tyck_expr(initializer)?;
                    if ty.try_unify(&init_ty)? == &init_ty { init_ty } else { ty.to_owned() }
                } else { ty.to_owned() };
                self.env.put(name.to_owned(), ty);
            }
            KalosStmt::Return(expr) => {
                // TODO: check type of expr matches the return type of current function
            }
            KalosStmt::If { cond, then_part, else_part } => {
                self.tyck_stmt(then_part)?;
                if let Some(else_part) = else_part {
                    self.tyck_stmt(else_part)?;
                }
            }
            KalosStmt::While { cond, body } => {
                self.tyck_stmt(body)?;
            }
            KalosStmt::Expression(expr) => {
                self.tyck_expr(expr)?;
            }
        }
        Ok(())
    }

    pub fn tyck_toplevel(&mut self, toplevel: &KalosToplevel) -> Result<(), KalosError> {
        match toplevel {
            KalosToplevel::Def { name, signature, body, .. } => {
                self.env.put(name.to_owned(),
                             KalosType::Function { signature: signature.to_owned() });
                self.env.push(signature.params.iter().map(|x| x.to_owned()).collect());
                if let Some(body) = body {
                    self.tyck_stmt(body)?;
                }
                self.env.pop();
            }
        }
        Ok(())
    }

    pub fn tyck_program(&mut self, program: &KalosProgram) -> Result<(), KalosError> {
        program.program.iter().try_for_each(|t| self.tyck_toplevel(t))
    }
}
