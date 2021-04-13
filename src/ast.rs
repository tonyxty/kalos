use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
pub enum KalosBuiltin {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
}

#[derive(Clone, Debug)]
pub enum KalosExpr {
    IntLiteral(i64),
    BoolLiteral(bool),
    StringLiteral(String),
    Call { func: Box<Self>, args: Vec<Self> },
    Builtin { builtin: KalosBuiltin, args: Vec<Self> },
    Identifier(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KalosType {
    Auto,

    Unit,
    Bool,
    Integer { signed: bool, width: usize },

    Text,

    Function { signature: KalosSignature },
}

impl KalosType {
    pub fn try_unify<'a>(&'a self, other: &'a Self) -> Result<&'a Self, KalosError> {
        use KalosType::*;
        if let Auto = self {
            Ok(other)
        } else if *self == *other {
            Ok(other)
        } else {
            Err(KalosError::TypeError {
                expect: Some(self.to_owned()),
                found: Some(other.to_owned())
            })
        }
    }
}

#[derive(Clone, Debug)]
pub enum KalosStmt {
    Compound(Vec<Self>),
    Assignment { lhs: KalosExpr, rhs: KalosExpr },
    Var { name: String, ty: KalosType, initializer: Option<KalosExpr> },
    Return(Option<KalosExpr>),
    If { cond: KalosExpr, then_part: Box<Self>, else_part: Option<Box<Self>> },
    While { cond: KalosExpr, body: Box<Self> },
    Expression(KalosExpr),
}

#[derive(Clone, Debug)]
pub struct KalosSignature {
    pub params: Vec<(String, KalosType)>,
    pub return_type: Box<KalosType>,
    pub variadic: bool,
}

impl PartialEq for KalosSignature {
    fn eq(&self, other: &Self) -> bool {
        self.params == other.params && *self.return_type == *other.return_type &&
            self.variadic == other.variadic
    }
}
impl Eq for KalosSignature {}

pub enum KalosToplevel {
    Def { name: String, signature: KalosSignature, body: Option<KalosStmt> },
}

pub struct KalosProgram {
    pub program: Vec<KalosToplevel>,
}

#[derive(Debug)]
pub enum KalosError {
    NameError,
    TypeError { expect: Option<KalosType>, found: Option<KalosType> },
    LvalueError,
    ArgError,
}

impl Display for KalosError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use KalosError::*;
        match self {
            NameError => f.write_str("NameError"),
            TypeError { expect, found } => f.write_str("TypeError"),
            LvalueError => f.write_str("LvalueError"),
            ArgError => f.write_str("ArgError"),
        }
    }
}

impl Error for KalosError {}
