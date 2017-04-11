use ast::*;
use backend::bytecodes::*;
use string_table::{self, StringToken};

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Write;
use std::vec::Vec;

macro_rules! return_error {
    ($($i:expr),*) => {{
        let mut s = String::new();
        write!(s, $($i),*).ok().unwrap();
        return Err(s);
    }}
}

#[derive(Debug)]
pub struct FunctionEntry {
    pub addr: usize,
    pub arity: usize,
    pub locals: usize
}

#[derive(Debug)]
pub struct Program {
    pub instructions: Vec<Bytecode>,
    pub call_table: Vec<FunctionEntry>
}

type FunctionToken = usize;
type LabelToken = usize;

#[derive(Debug)]
struct Function {
    bc: Vec<Bytecode>,
    labels: Vec<Option<usize>>,
    arity: usize,
    locals: usize,
}

impl Function {
    fn new(arity: usize) -> Function {
        Function {
            bc: Vec::new(),
            labels: Vec::new(),
            arity: arity,
            locals: 0,
        }
    }

    fn pos(&self) -> usize { self.bc.len() }

    fn int(&mut self, v: i64)            { self.bc.push(Bytecode::Int(v));             } 
    fn float(&mut self, v: f64)          { self.bc.push(Bytecode::Float(v));           } 
    fn bool(&mut self, v: bool)          { self.bc.push(Bytecode::Bool(v));            } 
    fn op2(&mut self, op: BinOp)         { self.bc.push(Bytecode::Op2(op));            } 
    fn op1(&mut self, op: UnOp)          { self.bc.push(Bytecode::Op1(op));            } 
    fn call(&mut self, f: FunctionToken) { self.bc.push(Bytecode::Call(f));            } 
    fn return_(&mut self, o: usize)      { self.bc.push(Bytecode::Return(o));          }
    fn discard(&mut self)                { self.bc.push(Bytecode::Discard);            }
    fn put_local(&mut self, o: usize)    { self.bc.push(Bytecode::PutLocal(o));        }
    fn get_local(&mut self, o: usize)    { self.bc.push(Bytecode::GetLocal(o));        }
    fn jump(&mut self, o: usize)         { self.bc.push(Bytecode::Jump(o as isize));   }
    fn branch(&mut self, o: usize)       { self.bc.push(Bytecode::Branch(o as isize)); }
    fn print(&mut self, st:StringToken, n: usize) { self.bc.push(Bytecode::Print(st, n)); }

    fn label(&mut self) -> LabelToken { 
        self.labels.push(None);
        self.labels.len()-1
    }

    fn bind(&mut self, l: LabelToken) {
        let pos = self.pos()-1;
        self.labels[l] = Some(pos);
    }

    fn set_locals(&mut self, l: usize) {
        self.locals = l;
    }

    fn locals(&self) -> usize {
        self.locals
    }

    fn arity(&self) -> usize {
        self.arity
    }

    fn resolve(mut self) -> Vec<Bytecode> { 
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

struct Environment<T> {
    ids: Vec<HashMap<StringToken, T>>,
}

impl<T: Copy> Environment<T> {
    fn new() -> Environment<T> {
        let mut ids = Vec::new();
        ids.push(HashMap::new());
        Environment {
            ids: ids,
        }
    }

    fn push_scope(&mut self) {
        self.ids.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        let _ = self.ids.pop().unwrap();
    }

    fn add(&mut self, id: StringToken, data: T) -> Result<(), String> {
        let l = self.ids.len();
        if self.ids[l-1].contains_key(&id) {
            return_error!("Illegal redefinition of identifier '{}'", string_table::get(id));
        }
        self.ids[l-1].insert(id, data);
        Ok(())
    }

    fn find(&self, id: StringToken) -> Option<T> {
        for hm in self.ids.iter().rev() {
            if let Some(v) = hm.get(&id) {
                return Some(*v);
            }
        }
        None
    }
}

struct LocalEnvironment {
    ids: Environment<usize>,
    id_count: Vec<usize>,
    id_total: usize,
    id_max: usize,
}

impl LocalEnvironment {
    fn new() -> LocalEnvironment {
        let mut id_count = Vec::new();
        id_count.push(0);
        LocalEnvironment {
            ids: Environment::new(),
            id_count: id_count,
            id_total: 0,
            id_max: 0
        }
    }

    fn push_scope(&mut self) {
        self.ids.push_scope();
        self.id_count.push(0);
    }

    fn pop_scope(&mut self) {
        self.ids.pop_scope();
        let h = self.id_count.pop().unwrap();
        self.id_total -= h;
    }

    fn add_id(&mut self, id: StringToken) -> Result<usize, String> {
        let pos = self.id_total;
        self.ids.add(id, pos)?;
        let l = self.id_count.len();
        self.id_count[l-1] += 1;
        self.id_total += 1;
        if self.id_total > self.id_max {
            self.id_max = self.id_total;
        }
        Ok(pos)
    }

    fn locals(&self) -> usize {
        self.id_max
    }

    fn find(&self, id: StringToken) -> Option<usize> {
        self.ids.find(id)
    }
}

struct Functions {
    ctx: Vec<FunctionToken>,
    fns: Vec<Function>,
    env: Environment<FunctionToken>,
}

impl Functions {
    fn new() -> Functions {
        let mut fns = Vec::new();
        let top = Function::new(0);
        fns.push(top);
        let mut ctx = Vec::new();
        ctx.push(0);
        Functions {
            ctx: ctx,
            fns: fns,
            env: Environment::new(),
        }
    }

    fn current(&mut self) -> &mut Function {
        let l = self.ctx.len();
        let ft = self.ctx[l-1];
        &mut self.fns[ft]
    }

    fn push_func(&mut self, name: StringToken, arity: usize) -> Result<(), String> {
        let ft = self.fns.len();
        self.env.add(name, ft)?;
        self.ctx.push(ft);
        let f = Function::new(arity);
        self.fns.push(f);
        self.env.push_scope();
        Ok(())
    }

    fn pop_func(&mut self) {
        let _ = self.ctx.pop();
        let _ = self.env.pop_scope();
    }

    fn lookup(&self, name: StringToken) -> Option<FunctionToken> {
        self.env.find(name)
    }

    fn to_program(self) -> Program {
        let Functions { ctx: _, fns, env: _ } = self; 
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

pub fn compile(ast: &Ast) -> Result<Program, String> {
    let mut env = LocalEnvironment::new();
    let mut fns = Functions::new();
    for stmt in ast.iter() {
        compile_stmt(stmt, &mut fns, &mut env)?;
    }
    {
        let top_func = fns.current();
        top_func.return_(env.locals());
        top_func.set_locals(env.locals());
    }
    Ok(fns.to_program())
}

fn compile_stmt(stmt: &Stmt, fns: &mut Functions, env: &mut LocalEnvironment) -> Result<(), String> {
    match *stmt {
        Stmt::DefFunc(name, ref params, ref b) => {
            fns.push_func(name, params.len())?;
            let mut new_env = LocalEnvironment::new();
            for p in params.iter() {
                new_env.add_id(*p)?;
            }
            compile_expr(b, fns, &mut new_env)?;
            {
                let new_func = fns.current();
                new_func.return_(new_env.locals());
                new_func.set_locals(new_env.locals());
            }
            fns.pop_func();
        },
        Stmt::DefVar(name, ref e) => {
            compile_expr(e, fns, env)?;
            let offset = env.add_id(name)?;
            let func = fns.current();
            func.put_local(offset);
            func.discard();
        },
        Stmt::Block(ref stmts) => {
            env.push_scope();
            for s in stmts.iter() {
                compile_stmt(s, fns, env)?;
            }
            env.pop_scope();
        },
        Stmt::While(ref p, ref b) => {
            let start_loop;
            let end_loop;
            {
                let func = fns.current();
                start_loop = func.label();
                end_loop = func.label();
                func.bind(start_loop);
            }
            compile_expr(p, fns, env)?;
            {
                let func = fns.current();
                func.op1(UnOp::Not);
                func.branch(end_loop);
            }
            compile_stmt(b.borrow(), fns, env)?;
            {
                let func = fns.current();
                func.jump(start_loop);
                func.bind(end_loop);
            }
        },
        Stmt::Expr(ref e) => {
            compile_expr(e, fns, env)?;
            let func = fns.current();
            func.discard();
        },
        Stmt::Print(lit, ref args) => {
            for a in args.iter() {
                compile_expr(a, fns, env)?;
            }
            let func = fns.current();
            func.print(lit, args.len());
        }
    }
    Ok(())
}

fn compile_expr(expr: &Expr, fns: &mut Functions, env: &mut LocalEnvironment) -> Result<(), String> {
    match *expr {
        Expr::Int(i) => fns.current().int(i),
        Expr::Float(f) => fns.current().float(f),
        Expr::Bool(b) => fns.current().bool(b),
        Expr::Ref(id) => match env.find(id) {
            Some(offset) => fns.current().get_local(offset),
            None => return_error!("Identifier '{}' not found in scope", string_table::get(id)),
        },
        Expr::If(ref p, ref t, ref e) => {
            let then;
            let done;
            {
                let func = fns.current();
                then = func.label();
                done = func.label();
            }
            compile_expr(p.borrow(), fns, env)?;
            {
                let func = fns.current();
                func.branch(then);
            }
            compile_expr(e.borrow(), fns, env)?;
            {
                let func = fns.current();
                func.jump(done);
                func.bind(then);
            }
            compile_expr(t.borrow(), fns, env)?;
            {
                let func = fns.current();
                func.bind(done);
            }
        },
        Expr::Block(ref stmts, ref e) => {
            env.push_scope();
            for s in stmts.iter() {
                compile_stmt(s, fns, env)?;
            }
            compile_expr(e.borrow(), fns, env)?;
            env.pop_scope();
        },
        Expr::Call(id, ref args) => {
            for a in args.iter() {
                compile_expr(a, fns, env)?;
            }
            match fns.lookup(id) {
                Some(ft) => fns.current().call(ft),
                None => return_error!("Function '{}' is not defined", string_table::get(id)),
            }
        },
        Expr::Assign(id, ref e) => {
            compile_expr(e.borrow(), fns, env)?;
            match env.find(id) {
                Some(offset) => fns.current().put_local(offset),
                None => return_error!("Identifier '{}' not found in scope", string_table::get(id)),
            }
        },
        Expr::UnOp(op, ref e) => {
            compile_expr(e.borrow(), fns, env)?;
            fns.current().op1(op);
        },
        Expr::BinOp(ref e1, op, ref e2) => {
            compile_expr(e1.borrow(), fns, env)?;
            compile_expr(e2.borrow(), fns, env)?;
            fns.current().op2(op);
        }
    }
    Ok(())
}
