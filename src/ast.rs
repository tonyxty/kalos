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
    Call { func: Box<Self>, args: Vec<Self> },
    BinOp { op: KalosBinOp, lhs: Box<Self>, rhs: Box<Self> },
    Identifier(String),
}

#[derive(Clone, Debug)]
pub enum KalosType {
    Auto,
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

pub struct KalosPrototype {
    pub name: String,
    pub params: Vec<String>,
    pub return_type: Option<KalosType>,
}

pub enum KalosToplevel {
    Def { prototype: KalosPrototype, body: KalosStmt },
    Extern(KalosPrototype),
}

pub struct KalosProgram {
    pub program: Vec<KalosToplevel>,
}
