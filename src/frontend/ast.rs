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
    ForEach(StringToken, Expr, Box<Stmt>),
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
    UnOp(UnOp, Bxpr),
    BinOp(Bxpr, BinOp, Bxpr),
}
