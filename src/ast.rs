
use std::collections::LinkedList;

pub type Iden = String;

pub type Program = LinkedList<Stmt>;

#[derive(Debug)]
pub enum Stmt {
    DefFunc(Iden, LinkedList<Iden>, Expr),
    DefVar(Iden, Expr),
    While(Expr, Expr),
    Expr(Expr),
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
    Eq,
    Ne,
    And,
    Or,
    BAnd,
    BOr,
    BXor,
    Cat,
}

#[derive(Debug)]
pub enum UnOp {
    Not,
    BNot,
}

pub type Bxpr = Box<Expr>;

#[derive(Debug)]
pub enum Expr {
    Num(i64),
    Ref(Iden),
    If(Bxpr, Bxpr, Bxpr),
    Block(LinkedList<Stmt>, Bxpr),
    Call(Iden, LinkedList<Expr>),
    Assign(Iden, Bxpr),
    //Alloc(Bxpr),
    //Take(Iden, Bxpr),
    //Slice(Iden, Bxpr, Bxpr),
    UnOp(UnOp, Bxpr),
    BinOp(Bxpr, BinOp, Bxpr),
}
