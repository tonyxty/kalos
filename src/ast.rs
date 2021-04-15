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
    LessThan,
    LessEqual,
    Equal,
    GreaterEqual,
    GreaterThan,
    NotEqual,
}

#[derive(Clone, Debug)]
pub enum KalosExpr {
    UnitLiteral,
    BoolLiteral(bool),
    IntLiteral(i64),
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

impl Display for KalosType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KalosType::Auto => write!(f, "_"),
            KalosType::Unit => write!(f, "()"),
            KalosType::Bool => write!(f, "bool"),
            KalosType::Integer { signed, width } =>
                write!(f, "{}{}", if *signed {'i'} else {'u'}, width),
            KalosType::Text => write!(f, "text"),
            KalosType::Function { signature } => {
                write!(f, "fn (")?;
                let mut sep = false;
                for (name, ty) in &signature.params {
                    if sep { write!(f, ", ")? }
                    write!(f, "{}: {}", name, ty)?;
                    sep = true;
                }
                if signature.variadic {
                    if sep { write!(f, ", ")? }
                    write!(f, "...")?;
                }
                write!(f, ") -> {}", signature.return_type)
            }
        }
    }
}

impl KalosType {
    pub fn try_unify<'a>(&'a self, other: &'a Self) -> Result<&'a Self, KalosError> {
        use KalosType::*;
        if let Auto = self {
            Ok(other)
        } else if *self == *other {
            Ok(other)
        } else {
            Err(KalosError::TypeError { expect: self.to_owned(), found: other.to_owned() })
        }
    }
}

#[derive(Clone, Debug)]
pub enum KalosStmt {
    Compound(Vec<Self>),
    Assignment { lhs: KalosExpr, rhs: KalosExpr },
    Var { name: String, ty: KalosType, initializer: Option<KalosExpr> },
    Return(KalosExpr),
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
    TypeError { expect: KalosType, found: KalosType },
    LvalueError,
    ArgError,
}

impl Display for KalosError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use KalosError::*;
        match self {
            NameError => write!(f, "NameError"),
            TypeError { expect, found } =>
                write!(f, "TypeError: expect {} found {}", expect, found),
            LvalueError => write!(f, "LvalueError"),
            ArgError => write!(f, "ArgError"),
        }
    }
}

impl Error for KalosError {}
