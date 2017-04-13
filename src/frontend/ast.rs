use std::collections::LinkedList;

use util::ops::{UnOp, BinOp};
use util::string_table::StringToken;

pub type Ast = LinkedList<Stmt>;

#[derive(Debug)]
pub enum Stmt {
    DefFunc(StringToken, LinkedList<StringToken>, Expr),
    DefVar(StringToken, Expr),
    Block(LinkedList<Stmt>),
    While(Expr, Box<Stmt>),
    Expr(Expr),
    Return(Expr),
    Print(StringToken, LinkedList<Expr>),
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
    Call(Bxpr, LinkedList<Expr>),
    Assign(StringToken, Bxpr),
    Get(StringToken, Bxpr),
    Put(StringToken, Bxpr, Bxpr),
    Array(LinkedList<Expr>),
    //QAlloc(Bxpr),
    UnOp(UnOp, Bxpr),
    BinOp(Bxpr, BinOp, Bxpr),
}
