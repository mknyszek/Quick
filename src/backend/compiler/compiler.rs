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

use frontend::ast::*;
use backend::bytecode::Program;
use backend::compiler::function::Functions;
use backend::compiler::environment::LocalEnvironment;
use util::ops::*;
use util::string_table;

use std::borrow::Borrow;
use std::fmt::Write;

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
        Stmt::ForEach(id, ref e, ref b) => {
            compile_expr(e, fns, env)?;
            env.push_scope();
            let id = env.add_id(id)?;
            let counter = env.add_tmp();
            let array = env.add_tmp();
            let start_loop;
            let end_loop;
            {
                let func = fns.current();
                start_loop = func.label();
                end_loop = func.label();

                func.put_local(array);
            }
            builtin_call!(fns, len, 1);   
            {
                let func = fns.current();
                func.put_local(counter);
                func.int(0);
                func.op2(BinOp::Le);
                func.branch(end_loop);

                func.bind(start_loop);
                func.get_local(array);
                func.get_local(counter);
                func.int(1);
                func.op2(BinOp::Sub);
                func.put_local(counter);
            }
            builtin_call!(fns, get, 2);   
            {
                let func = fns.current();
                func.put_local(id);
                func.discard();
            }
            compile_stmt(b.borrow(), fns, env)?;
            {
                let func = fns.current();
                func.get_local(counter);
                func.int(0);
                func.op2(BinOp::Gt);
                func.branch(start_loop);
                func.bind(end_loop);
            }
            env.pop_scope();
        },
        Stmt::ForLoop(id, ref s, ref e, ref b) => {
            compile_expr(s, fns, env)?;
            env.push_scope();
            let id = env.add_id(id)?;
            let end = env.add_tmp();
            let start_loop;
            let end_loop;
            {
                let func = fns.current();
                start_loop = func.label();
                end_loop = func.label();

                func.put_local(id);
            }
            compile_expr(e, fns, env)?;
            {
                let func = fns.current();
                func.put_local(end);
                func.op2(BinOp::Ge);
                func.branch(end_loop);

                func.bind(start_loop);
                func.get_local(id);
                func.int(1);
                func.op2(BinOp::Add);
                func.put_local(id);
                func.discard();
            }
            compile_stmt(b.borrow(), fns, env)?;
            {
                let func = fns.current();
                func.get_local(id);
                func.get_local(end);
                func.op2(BinOp::Lt);
                func.branch(start_loop);
                func.bind(end_loop);
            }
            env.pop_scope();
        },
        Stmt::With(id, ref p, ref b) => {
            compile_rev_expr(p, fns, env)?;
            env.push_scope();
            let id = env.add_id(id)?;
            {
                let func = fns.current();
                func.put_local(id);
                func.discard();
            }
            compile_stmt(b.borrow(), fns, env)?;
            fns.current().get_local(id);
            env.pop_scope();
            compile_inv_expr(p, fns, env)?;
        },
        Stmt::Return(ref e) => {
            compile_expr(e, fns, env)?;
            fns.current().return_(env.locals());
        },
        Stmt::Expr(ref e) => {
            compile_expr(e, fns, env)?;
            fns.current().discard();
        },
        Stmt::Print(lit, ref args) => {
            for a in args.iter() {
                compile_expr(a, fns, env)?;
            }
            fns.current().print(lit, args.len());
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
            None => match fns.lookup(id) {
                Some(ft) => fns.current().func(ft),
                None => return_error!("Identifier '{}' is not defined", string_table::get(id)),
            },
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
        Expr::Move(id) => match env.find(id) {
            Some(offset) => {
                let func = fns.current();
                func.get_local(offset);
                func.null();
                func.put_local(offset);
                func.discard();
            },
            None => return_error!("Identifier '{}' is not defined", string_table::get(id)),
        },
        Expr::Call(ref f, ref args) => {
            for a in args.iter() {
                compile_expr(a, fns, env)?;
            }
            compile_expr(f.borrow(), fns, env)?;
            fns.current().call(args.len());
        },
        Expr::Assign(id, ref e) => {
            compile_expr(e.borrow(), fns, env)?;
            match env.find(id) {
                Some(offset) => fns.current().put_local(offset),
                None => return_error!("Identifier '{}' not found in scope", string_table::get(id)),
            }
        },
        Expr::Array(ref args) => {
            for a in args.iter() {
                compile_expr(a, fns, env)?;
            }
            let func = fns.current();
            func.array(args.len());
        },
        Expr::UnOp(op, ref e) => {
            compile_expr(e.borrow(), fns, env)?;
            fns.current().op1(op);
        },
        Expr::BinOp(ref e1, op, ref e2) => {
            compile_expr(e1.borrow(), fns, env)?;
            compile_expr(e2.borrow(), fns, env)?;
            fns.current().op2(op);
        },
        Expr::Cat(ref e1, ref e2) => builtin_call!(fns, env, cat, 2, e1, e2),
        Expr::Get(ref e1, ref e2) => builtin_call!(fns, env, get, 2, e1, e2),
        Expr::Put(ref e1, ref e2, ref e3) => builtin_call!(fns, env, put, 3, e1, e2, e3),
        Expr::Slice(ref e1, ref e2, ref e3) => builtin_call!(fns, env, slice, 3, e1, e2, e3),
        Expr::Len(ref e) => builtin_call!(fns, env, len, 1, e),
        Expr::QAlloc(ref n, ref i) => builtin_call!(fns, env, qalloc, 2, n, i),
        Expr::Invoke(ref f) => {
            compile_expr(f.borrow(), fns, env)?;
            fns.current().call(0);
        },
        Expr::Apply(ref f, ref a) => {
            compile_expr(a.borrow(), fns, env)?;
            compile_expr(f.borrow(), fns, env)?;
            fns.current().call(1);
        },
    }
    Ok(())
}

fn compile_rev_expr(expr: &Expr, fns: &mut Functions, env: &mut LocalEnvironment) -> Result<(), String> {
    match *expr {
        Expr::Int(i) => fns.current().int(i),
        Expr::Bool(b) => fns.current().bool(b),
        Expr::Ref(id) => match env.find(id) {
            Some(offset) => fns.current().get_local(offset),
            None => match fns.lookup(id) {
                Some(ft) => fns.current().func(ft),
                None => return_error!("Identifier '{}' is not defined", string_table::get(id)),
            },
        },
        Expr::Call(ref f, ref args) => {
            for a in args.iter() {
                compile_rev_expr(a, fns, env)?;
            }
            compile_rev_expr(f.borrow(), fns, env)?;
            fns.current().rcall(args.len());
        },
        Expr::UnOp(op, ref e) => {
            compile_rev_expr(e.borrow(), fns, env)?;
            fns.current().rop1(op);
        },
        Expr::BinOp(ref e1, op, ref e2) => {
            compile_rev_expr(e1.borrow(), fns, env)?;
            compile_rev_expr(e2.borrow(), fns, env)?;
            fns.current().rop2(op);
        },
        Expr::Cat(ref e1, ref e2) => builtin_rcall!(fns, env, cat, 2, e1, e2),
        Expr::Get(ref e1, ref e2) => builtin_rcall!(fns, env, get, 2, e1, e2),
        Expr::Put(ref e1, ref e2, ref e3) => builtin_rcall!(fns, env, put, 3, e1, e2, e3),
        Expr::Slice(ref e1, ref e2, ref e3) => builtin_rcall!(fns, env, slice, 3, e1, e2, e3),
        Expr::Len(ref e) => builtin_rcall!(fns, env, len, 1, e),
        Expr::QAlloc(ref n, ref i) => builtin_rcall!(fns, env, qalloc, 2, n, i),
        Expr::Invoke(ref f) => {
            compile_rev_expr(f.borrow(), fns, env)?;
            fns.current().rcall(0);
        },
        Expr::Apply(ref f, ref a) => {
            compile_rev_expr(a.borrow(), fns, env)?;
            compile_rev_expr(f.borrow(), fns, env)?;
            fns.current().rcall(1);
        },
        _ => panic!("Feature {:?} is not reversible.", expr),
    }
    Ok(())
}

fn compile_inv_expr(expr: &Expr, fns: &mut Functions, env: &mut LocalEnvironment) -> Result<(), String> {
    match *expr {
        Expr::Int(_) => fns.current().discard(),
        Expr::Bool(_) => fns.current().discard(),
        Expr::Ref(_) => fns.current().discard(),
        Expr::Call(ref f, ref args) => {
            fns.current().icall(args.len());
            compile_inv_expr(f.borrow(), fns, env)?;
            for a in args.iter().rev() {
                compile_inv_expr(a, fns, env)?;
            }
        },
        Expr::UnOp(op, ref e) => {
            fns.current().iop1(op);
            compile_inv_expr(e.borrow(), fns, env)?;
        },
        Expr::BinOp(ref e1, op, ref e2) => {
            fns.current().iop2(op);
            compile_inv_expr(e2.borrow(), fns, env)?;
            compile_inv_expr(e1.borrow(), fns, env)?;
        },
        Expr::Cat(ref e1, ref e2) => builtin_icall!(fns, env, cat, 2, e2, e1),
        Expr::Get(ref e1, ref e2) => builtin_icall!(fns, env, get, 2, e2, e1),
        Expr::Put(ref e1, ref e2, ref e3) => builtin_icall!(fns, env, put, 3, e3, e2, e1),
        Expr::Slice(ref e1, ref e2, ref e3) => builtin_icall!(fns, env, slice, 3, e3, e2, e1),
        Expr::Len(ref e) => builtin_icall!(fns, env, len, 1, e),
        Expr::QAlloc(ref n, ref i) => builtin_icall!(fns, env, qalloc, 2, n, i),
        Expr::Invoke(ref f) => {
            fns.current().icall(0);
            compile_inv_expr(f.borrow(), fns, env)?;
        },
        Expr::Apply(ref f, ref a) => {
            fns.current().icall(1);
            compile_inv_expr(f.borrow(), fns, env)?;
            compile_inv_expr(a.borrow(), fns, env)?;
        },
        _ => panic!("Feature {:?} is not reversible.", expr),
    }
    Ok(())
}
