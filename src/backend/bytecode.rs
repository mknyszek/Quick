// Copyright (C) 2017 Michael Anthony Knyszek
//
// This file is part of QScript
//
// QScript is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// QScript is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

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

#[derive(Clone, Copy, Debug)]
pub enum Call {
    Regular,
    Reverse,
    Inverse
}

#[derive(Debug, Clone)]
pub enum Bytecode {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    Func(FunctionToken),
    Array(usize),
    Op3(TriOp),
    Op2(BinOp),
    Op1(UnOp),
    Call(Call, usize),
    Discard,
    Return(usize),
    PutLocal(usize),
    GetLocal(usize),
    Jump(isize),
    Branch(isize),
    Print(StringToken, usize),
}
