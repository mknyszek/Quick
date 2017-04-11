use std::collections::LinkedList;

use string_table::StringToken;

pub type Ast = LinkedList<Stmt>;

#[derive(Debug)]
pub enum Stmt {
    DefFunc(StringToken, LinkedList<StringToken>, Expr),
    DefVar(StringToken, Expr),
    While(Expr, Expr),
    Expr(Expr),
    Print(StringToken, LinkedList<Expr>),
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
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
    BNot,
}

pub type Bxpr = Box<Expr>;

#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    Ref(StringToken),
    If(Bxpr, Bxpr, Bxpr),
    Block(LinkedList<Stmt>, Bxpr),
    Call(StringToken, LinkedList<Expr>),
    Assign(StringToken, Bxpr),
    //Alloc(Bxpr),
    //Take(StringToken, Bxpr),
    //Slice(StringToken, Bxpr, Bxpr),
    UnOp(UnOp, Bxpr),
    BinOp(Bxpr, BinOp, Bxpr),
}
