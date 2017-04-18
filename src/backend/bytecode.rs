use util::ops::{TriOp, BinOp, UnOp};
use util::string_table::StringToken;

use backend::runtime::IRT_TABLE;

use std::vec::Vec;

#[derive(Debug)]
pub struct Program {
    pub instructions: Vec<Bytecode>,
    pub call_table: Vec<FunctionEntry>
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FunctionToken {
    id: usize
}

impl FunctionToken {
    pub fn from_index(id: usize) -> FunctionToken {
        FunctionToken { id: id }
    }

    pub fn inc(self) -> FunctionToken {
        FunctionToken::from_index(self.id + 1)
    }

    pub fn is_native(self) -> bool {
        self.id < IRT_TABLE.len()
    }

    pub fn to_native_index(self) -> usize {
        self.id
    }

    pub fn to_call_index(self) -> usize {
        assert!(self.id >= IRT_TABLE.len());
        self.id - IRT_TABLE.len()
    }
}

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
}
