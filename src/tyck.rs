use std::collections::HashMap;

use crate::ast::{KalosBuiltin, KalosError, KalosExpr, KalosProgram, KalosStmt, KalosToplevel, KalosType::{self, *}};
use crate::env::Env;

pub struct Tycker {
    env: Env<String, KalosType>,
    current_fn_return_type: Option<KalosType>,
}

impl Tycker {
    pub fn new() -> Self {
        Self {
            env: Env::from(vec![HashMap::new()]),
            current_fn_return_type: None,
        }
    }

    pub fn get_globals(&self) -> &HashMap<String, KalosType> {
        self.env.tables.first().unwrap()
    }

    fn tyck_builtin(&self, builtin: KalosBuiltin,
                    args: &[KalosExpr]) -> Result<KalosType, KalosError> {
        use KalosBuiltin::*;
        let lhs = self.tyck_expr(&args[0])?;
        let rhs = self.tyck_expr(&args[1])?;
        match builtin {
            Add => Ok(lhs),
            Subtract => Ok(lhs),
            Multiply => Ok(lhs),
            Divide => Ok(lhs),
            Modulo => Ok(lhs),
            Power => Ok(lhs),
            LessThan => Ok(Bool),
            LessEqual => Ok(Bool),
            Equal => Ok(Bool),
            GreaterEqual => Ok(Bool),
            GreaterThan => Ok(Bool),
            NotEqual => Ok(Bool),
        }
    }

    pub fn tyck_expr(&self, expr: &KalosExpr) -> Result<KalosType, KalosError> {
        use KalosExpr::*;
        match expr {
            UnitLiteral => Ok(Unit),
            IntLiteral(_) => Ok(Integer { signed: true, width: 64 }),
            BoolLiteral(_) => Ok(Bool),
            StringLiteral(_) => Ok(Text),
            Call { func, args } => {
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
                    Err(KalosError::TypeError { expect: Auto, found: ty })
                }
            }
            Builtin { builtin, args } => self.tyck_builtin(*builtin, args),
            Identifier(name) => Ok(self.env.get(name).ok_or(KalosError::NameError)?.to_owned()),
        }
    }

    pub fn tyck_stmt(&mut self, stmt: &KalosStmt) -> Result<(), KalosError> {
        use KalosStmt::*;
        match stmt {
            Compound(s) => {
                self.env.push_empty();
                s.iter().try_for_each(|stmt| self.tyck_stmt(stmt))?;
                self.env.pop();
            }
            Assignment { lhs, rhs } => {
                let lhs_type = self.tyck_expr(lhs)?;
                let rhs_type = self.tyck_expr(rhs)?;
                lhs_type.try_unify(&rhs_type)?;
            }
            Var { name, ty, initializer } => {
                let ty = if let Some(initializer) = initializer {
                    let init_ty = self.tyck_expr(initializer)?;
                    if ty.try_unify(&init_ty)? == &init_ty { init_ty } else { ty.to_owned() }
                } else { ty.to_owned() };
                self.env.put(name.to_owned(), ty);
            }
            Return(expr) => {
                let ty = self.tyck_expr(expr)?;
                &self.current_fn_return_type.as_ref().unwrap().try_unify(&ty)?;
            }
            If { cond, then_part, else_part } => {
                Bool.try_unify(&self.tyck_expr(cond)?)?;
                self.tyck_stmt(then_part)?;
                if let Some(else_part) = else_part {
                    self.tyck_stmt(else_part)?;
                }
            }
            While { cond, body } => {
                Bool.try_unify(&self.tyck_expr(cond)?)?;
                self.tyck_stmt(body)?;
            }
            Expression(expr) => { self.tyck_expr(expr)?; }
        }
        Ok(())
    }

    pub fn tyck_toplevel(&mut self, toplevel: &KalosToplevel) -> Result<(), KalosError> {
        match toplevel {
            KalosToplevel::Def { name, signature, body, .. } => {
                self.env.put(name.to_owned(), Function { signature: signature.to_owned() });
                if let Some(body) = body {
                    self.env.push(signature.params.iter().map(|x| x.to_owned()).collect());
                    self.current_fn_return_type = Some(*signature.return_type.to_owned());
                    self.tyck_stmt(body)?;
                    self.env.pop();
                }
            }
        }
        Ok(())
    }

    pub fn tyck_program(&mut self, program: &KalosProgram) -> Result<(), KalosError> {
        program.program.iter().try_for_each(|t| self.tyck_toplevel(t))
    }
}
