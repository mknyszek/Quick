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

use backend::bytecode::*;
use backend::compiler::environment::Environment;
use backend::runtime::{IRT_STRINGS, IRT_TABLE};
use util::ops::*;
use util::string_table::{self, StringToken};

use std::vec::Vec;

type LabelToken = usize;

#[derive(Debug)]
pub struct Function {
    bc: Vec<Bytecode>,
    labels: Vec<Option<usize>>,
    arity: usize,
    locals: usize,
}

impl Function {
    pub fn new(arity: usize) -> Function {
        Function {
            bc: Vec::new(),
            labels: Vec::new(),
            arity: arity,
            locals: 0,
        }
    }

    pub fn pos(&self) -> usize { self.bc.len() }

    pub fn null(&mut self)                   { self.bc.push(Bytecode::Null);               }
    pub fn int(&mut self, v: i64)            { self.bc.push(Bytecode::Int(v));             } 
    pub fn float(&mut self, v: f64)          { self.bc.push(Bytecode::Float(v));           } 
    pub fn bool(&mut self, v: bool)          { self.bc.push(Bytecode::Bool(v));            } 
    pub fn func(&mut self, f: FunctionToken) { self.bc.push(Bytecode::Func(f));            }
    pub fn array(&mut self, len: usize)      { self.bc.push(Bytecode::Array(len));         }
    pub fn op3(&mut self, op: TriOp)         { self.bc.push(Bytecode::Op3(op));            } 
    pub fn op2(&mut self, op: BinOp)         { self.bc.push(Bytecode::Op2(op));            } 
    pub fn op1(&mut self, op: UnOp)          { self.bc.push(Bytecode::Op1(op));            } 
    pub fn call(&mut self, arity: usize)     { self.bc.push(Bytecode::Call(Call::Regular, arity)); } 
    pub fn rcall(&mut self, arity: usize)    { self.bc.push(Bytecode::Call(Call::Reverse, arity)); } 
    pub fn icall(&mut self, arity: usize)    { self.bc.push(Bytecode::Call(Call::Inverse, arity)); } 
    pub fn return_(&mut self, o: usize)      { self.bc.push(Bytecode::Return(o));          }
    pub fn discard(&mut self)                { self.bc.push(Bytecode::Discard);            }
    pub fn put_local(&mut self, o: usize)    { self.bc.push(Bytecode::PutLocal(o));        }
    pub fn get_local(&mut self, o: usize)    { self.bc.push(Bytecode::GetLocal(o));        }
    pub fn jump(&mut self, o: usize)         { self.bc.push(Bytecode::Jump(o as isize));   }
    pub fn branch(&mut self, o: usize)       { self.bc.push(Bytecode::Branch(o as isize)); }
    pub fn print(&mut self, st: StringToken, n: usize) { self.bc.push(Bytecode::Print(st, n));   }

    pub fn label(&mut self) -> LabelToken { 
        self.labels.push(None);
        self.labels.len()-1
    }

    pub fn bind(&mut self, l: LabelToken) {
        let pos = self.pos()-1;
        self.labels[l] = Some(pos);
    }

    pub fn set_locals(&mut self, l: usize) {
        self.locals = l;
    }

    pub fn locals(&self) -> usize {
        self.locals
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn resolve(mut self) -> Vec<Bytecode> { 
        let mut i: isize = 0;
        for b in self.bc.iter_mut() {
            let bcopy = (*b).clone();
            match bcopy {
                Bytecode::Branch(o) => *b = match self.labels[o as usize] {
                    Some(o) => Bytecode::Branch((o as isize) - i + 1),
                    None => panic!("Internal Error: Found unbound label!"),
                },
                Bytecode::Jump(o) => *b = match self.labels[o as usize] {
                    Some(o) => Bytecode::Jump((o as isize) - i + 1),
                    None => panic!("Internal Error: Found unbound label!"),
                },
                _ => (),
            }
            i += 1;
        }
        self.bc
    }
}

pub struct Functions {
    ftg: FunctionToken,
    ctx: Vec<FunctionToken>,
    fns: Vec<Function>,
    env: Environment<FunctionToken>,
}

impl Functions {
    pub fn new() -> Functions {
        let ftg = FunctionToken::from_index(IRT_TABLE.len());
        let mut fns = Vec::new();
        let mut ctx = Vec::new();
        let mut env = Environment::new();
        let top = Function::new(0);
        fns.push(top);
        ctx.push(ftg);
        let mut i = 0;
        for s in IRT_STRINGS.iter() {
            let ft = FunctionToken::from_index(i);
            env.add(string_table::insert(s), ft).unwrap();
            i += 1;
        }
        Functions {
            ftg: ftg.inc(),
            ctx: ctx,
            fns: fns,
            env: env,
        }
    }

    pub fn current(&mut self) -> &mut Function {
        let l = self.ctx.len();
        let ft = self.ctx[l-1];
        &mut self.fns[ft.to_call_index()]
    }

    pub fn push_func(&mut self, name: StringToken, arity: usize) -> Result<(), String> {
        self.env.add(name, self.ftg)?;
        self.ctx.push(self.ftg);
        let f = Function::new(arity);
        self.fns.push(f);
        self.env.push_scope();
        self.ftg = self.ftg.inc();
        Ok(())
    }

    pub fn pop_func(&mut self) {
        let _ = self.ctx.pop();
        let _ = self.env.pop_scope();
    }

    pub fn lookup(&self, name: StringToken) -> Option<FunctionToken> {
        self.env.find(name)
    }

    pub fn to_program(self) -> Program {
        let Functions { ftg: _, ctx: _, fns, env: _ } = self; 
        let mut instructions = Vec::new();
        let mut call_table = Vec::new();
        for func in fns.into_iter() {
            let start = instructions.len();
            let arity = func.arity();
            let locals = func.locals();
            instructions.extend(func.resolve().into_iter());
            call_table.push(FunctionEntry { addr: start, arity: arity, locals: locals });
        }
        Program { instructions: instructions, call_table: call_table }
    }
}
