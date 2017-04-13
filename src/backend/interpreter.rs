use util::ops::*;
use util::string_table;

use backend::bytecode::*;
use backend::runtime::{self, Value};

use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;

pub fn interpret(program: Program) {
    let mut stack: Vec<Value> = Vec::with_capacity(program.call_table[0].locals);
    for _ in 0..program.call_table[0].locals {
        stack.push(Value::Empty);
    }
    let mut pc: usize = 0;
    let mut fp: usize = 0;

    // Top-of-stack optimization
    let mut a0 = Value::Empty;

    loop {
        //println!("{}: {:?}\n {:?}, {:?}", pc, program.instructions[pc], stack, a0);
        match program.instructions[pc] {
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
                let sp = stack.len();
                for i in (0..len-1).rev() {
                    v.push(stack[sp-1-i].clone());
                }
                v.push(a0);
                for _ in 0..len-1 {
                    let _ = stack.pop().unwrap();
                }
                a0 = Value::Array(Rc::new(RefCell::new(v)));
            },
            Bytecode::Op3(op) => {
                let t0 = stack.pop().unwrap();
                let t1 = stack.pop().unwrap();
                match op {
                    TriOp::Put => a0 = t1.put(t0, a0),
                }
            },
            Bytecode::Op2(op) => {
                let t0 = stack.pop().unwrap();
                match op {
                    BinOp::Add => a0 = t0.add(a0),
                    BinOp::Sub => a0 = t0.sub(a0),
                    BinOp::Mul => a0 = t0.mul(a0),
                    BinOp::Div => a0 = t0.div(a0),
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
                    BinOp::Get => a0 = t0.get(a0),
                    BinOp::Cat => a0 = t0.cat(a0),
                }
            },
            Bytecode::Op1(op) => {
                match op {
                    UnOp::Neg => a0 = a0.neg(),
                    UnOp::Not => a0 = a0.not(),
                    UnOp::BNot => a0 = a0.bnot(),
                    UnOp::Len => a0 = a0.len(),
                }
            },
            Bytecode::Call => {
                let ft = a0.as_func();
                let ref fe = program.call_table[ft];
                for _ in 0..(fe.locals - fe.arity) {
                    stack.push(Value::Empty);
                }
                let old_fp = fp;
                fp = stack.len() - fe.locals;
                stack.push(Value::Addr(pc + 1));
                a0 = Value::Addr(old_fp);
                pc = fe.addr;
                continue;
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
            Bytecode::PutLocal(index) => {
                stack[fp + index] = a0.clone();
                //a0 = stack.pop().unwrap();
            },
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
