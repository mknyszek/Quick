use util::ops::{TriOp, BinOp, UnOp};
use util::string_table::StringToken;

use std::vec::Vec;

#[derive(Debug)]
pub struct Program {
    pub instructions: Vec<Bytecode>,
    pub call_table: Vec<FunctionEntry>
}

pub type FunctionToken = usize;

#[derive(Debug)]
pub struct FunctionEntry {
    pub addr: usize,
    pub arity: usize,
    pub locals: usize
}

#[derive(Debug, Clone)]
pub enum Bytecode {
    Int(i64),
    Float(f64),
    Bool(bool),
    Func(FunctionToken),
    Array(usize),
    Op3(TriOp),
    Op2(BinOp),
    Op1(UnOp),
    Call(usize),
    Discard,
    Return(usize),
    PutLocal(usize),
    GetLocal(usize),
    Jump(isize),
    Branch(isize),
    Print(StringToken, usize),
    //QAlloc(),
}
