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

use util::ops::*;
use util::string_table;

use backend::bytecode::*;
use backend::runtime::{self, IRT_TABLE};
use backend::runtime::value::Value;

use std::borrow::Borrow;
use std::vec::Vec;

pub fn interpret(program: Program) {
    let mut stack: Vec<Value> = Vec::with_capacity(program.call_table[0].locals);
    let mut aux: Vec<Value> = Vec::new();
    for _ in 0..program.call_table[0].locals {
        stack.push(Value::Null);
    }
    let mut pc: usize = 0;
    let mut fp: usize = 0;

    // Top-of-stack optimization
    let mut a0 = Value::Null;

    loop {
        //println!("{}: {:?}\n {:?}, {:?}", pc, program.instructions[pc], stack, a0);
        match program.instructions[pc] {
            Bytecode::Null => {
                stack.push(a0);
                a0 = Value::Null;
            },
            Bytecode::Int(v) => {
                stack.push(a0);
                a0 = Value::Int(v);
            },
            Bytecode::Float(v) => {
                stack.push(a0);
                a0 = Value::Float(v);
            },
            Bytecode::Bool(v) => {
                stack.push(a0);
                a0 = Value::Bool(v);
            },
            Bytecode::Func(ft) => {
                stack.push(a0);
                a0 = Value::Func(ft);
            },
            Bytecode::Array(len) => {
                let mut v = Vec::with_capacity(len);
                if len != 0 {
                    let sp = stack.len();
                    for i in (0..len-1).rev() {
                        v.push(stack[sp-1-i].clone());
                    }
                    v.push(a0);
                    for _ in 0..len-1 {
                        let _ = stack.pop().unwrap();
                    }
                } else {
                    stack.push(a0);
                }
                a0 = Value::new_array(v);
            },
            Bytecode::Op2(kind, op) => {
                if let Call::Inverse = kind {
                    let _ = stack.pop().unwrap();
                    stack.push(aux.pop().unwrap());
                    a0 = aux.pop().unwrap();
                } else {
                    let t0 = stack.pop().unwrap();
                    if let Call::Reverse = kind {
                        aux.push(t0.clone());
                        aux.push(a0.clone());
                    }
                    match op {
                        BinOp::Add => a0 = t0.add(a0),
                        BinOp::Sub => a0 = t0.sub(a0),
                        BinOp::Mul => a0 = t0.mul(a0),
                        BinOp::Div => a0 = t0.div(a0),
                        BinOp::Rem => a0 = t0.rem(a0),
                        BinOp::Pow => a0 = t0.pow(a0),
                        BinOp::Lt => a0 = t0.lt(a0),
                        BinOp::Gt => a0 = t0.gt(a0),
                        BinOp::Le => a0 = t0.le(a0),
                        BinOp::Ge => a0 = t0.ge(a0),
                        BinOp::Eq => a0 = t0.eq(a0),
                        BinOp::Ne => a0 = t0.ne(a0),
                        BinOp::And => a0 = t0.and(a0),
                        BinOp::Or => a0 = t0.or(a0),
                        BinOp::BAnd => a0 = t0.band(a0),
                        BinOp::BOr => a0 = t0.bor(a0),
                        BinOp::BXor => a0 = t0.bxor(a0),
                    }
                }
            },
            Bytecode::Op1(_, op) => {
                // Turns out all supported unary ops are inherently
                // reversible, and their own inverse. Don't need to
                // do anything.
                match op {
                    UnOp::Neg => a0 = a0.neg(),
                    UnOp::Not => a0 = a0.not(),
                    UnOp::BNot => a0 = a0.bnot(),
                }
            },
            Bytecode::Call(kind, arity) => {
                let ft = a0.as_func();
                if ft.is_native() {
                    let ref nfe = IRT_TABLE[ft.to_native_index()];
                    assert_eq!(arity, nfe.arity);
                    match kind {
                        Call::Regular => (nfe.entry.irr)(&mut stack),
                        Call::Reverse => (nfe.entry.rev)(&mut stack, &mut aux),
                        Call::Inverse => (nfe.entry.inv)(&mut stack, &mut aux),
                    }
                    a0 = stack.pop().unwrap();
                } else {
                    let ref fe = program.call_table[ft.to_call_index()];
                    assert_eq!(arity, fe.arity);
                    for _ in 0..(fe.locals - fe.arity) {
                        stack.push(Value::Null);
                    }
                    let old_fp = fp;
                    fp = stack.len() - fe.locals;
                    stack.push(Value::Addr(pc + 1));
                    a0 = Value::Addr(old_fp);
                    pc = fe.addr;
                    continue;
                }
            },
            Bytecode::Discard => a0 = stack.pop().unwrap(),
            Bytecode::Return(locals) => {
                if fp == 0 { return; }
                fp = stack.pop().unwrap().as_addr();
                pc = stack.pop().unwrap().as_addr();
                let sp = stack.len();
                stack.truncate(sp - locals);
                continue;
            },
            Bytecode::PutLocal(index) => stack[fp + index] = a0.clone(),
            Bytecode::GetLocal(index) => {
                stack.push(a0);
                a0 = stack[fp + index].clone();
            },
            Bytecode::Jump(offset) => {
                pc = (pc as isize + offset) as usize;
                continue;
            },
            Bytecode::Branch(offset) => {
                let pred = a0.as_bool();
                a0 = stack.pop().unwrap();
                if pred {
                    pc = (pc as isize + offset) as usize;
                    continue;
                }
            },
            Bytecode::Print(fmt, nargs) => {
                stack.push(a0);
                let sp = stack.len();
                runtime::printf(string_table::get(fmt).borrow(), &stack[sp-nargs..sp]);
                for _ in 0..nargs {
                    stack.pop().unwrap();
                }
                a0 = stack.pop().unwrap();
            }
        }
        pc += 1;
    }
}
