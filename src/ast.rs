#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
pub enum KalosBinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
}

#[derive(Clone, Debug)]
pub enum KalosExpr {
    Literal(i64),
    Call(Box<Self>, Vec<Self>),
    BinOp(KalosBinOp, Box<Self>, Box<Self>),
    Identifier(String),
}

#[derive(Clone, Debug)]
pub enum KalosTypeExpr {
    Auto,
}

#[derive(Clone, Debug)]
pub enum KalosStmt {
    Compound(Vec<Self>),
    Assignment(KalosExpr, KalosExpr),
    Var(String, KalosTypeExpr, Option<KalosExpr>),
    Return(KalosExpr),
    If(KalosExpr, Box<Self>, Option<Box<Self>>),
    While(KalosExpr, Box<Self>),
    Expression(KalosExpr),
}

pub enum KalosToplevel {
    Def(String, Vec<String>, KalosStmt),
}
