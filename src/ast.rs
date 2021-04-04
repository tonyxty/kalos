#[derive(Hash, PartialEq, Eq, Debug)]
pub enum KalosBinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
}

#[derive(Debug)]
pub enum KalosExpr {
    Literal(i64),
    Call(Box<Self>, Vec<Self>),
    BinOp(KalosBinOp, Box<Self>, Box<Self>),
    Identifier(String),
}

#[derive(Debug)]
pub enum KalosStmt {
    Compound(Vec<Self>),
    Assignment(KalosExpr, KalosExpr),
    If(KalosExpr, Box<Self>, Option<Box<Self>>),
    While(KalosExpr, Box<Self>),
    Expression(KalosExpr),
}
