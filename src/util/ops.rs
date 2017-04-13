#[derive(Debug, Clone, Copy)]
pub enum TriOp {
    Put,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    BAnd,
    BOr,
    BXor,
    Get,
    Cat,
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
    BNot,
}
