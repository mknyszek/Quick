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
                func.op1(UnOp::Len);
                func.put_local(counter);
                func.int(0);
                func.op2(BinOp::Ge);
                func.branch(end_loop);

                func.bind(start_loop);
                func.int(1);
                func.get_local(counter);
                func.op2(BinOp::Sub);
                func.put_local(counter);
                func.get_local(array);
                func.op2(BinOp::Get);
                func.put_local(id);
                func.discard();
            }
            compile_stmt(b.borrow(), fns, env)?;
            {
                let func = fns.current();
                func.int(0);
                func.get_local(counter);
                func.op2(BinOp::Gt);
                func.branch(start_loop);
                func.bind(end_loop);
            }
        },
        Stmt::Return(ref e) => {
            compile_expr(e, fns, env)?;
            let func = fns.current();
            func.return_(env.locals());
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
        Expr::Get(id, ref idx) => {
            compile_expr(idx.borrow(), fns, env)?;
            match env.find(id) {
                Some(offset) => fns.current().get_local(offset),
                None => return_error!("Identifier '{}' not found in scope", string_table::get(id)),
            }
            fns.current().op2(BinOp::Get);
        },
        Expr::Put(id, ref idx, ref e) => {
            compile_expr(e.borrow(), fns, env)?;
            compile_expr(idx.borrow(), fns, env)?;
            match env.find(id) {
                Some(offset) => fns.current().get_local(offset),
                None => return_error!("Identifier '{}' not found in scope", string_table::get(id)),
            }
            fns.current().op3(TriOp::Put);
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
            if let UnOp::Invoke = op {
                fns.current().call(0);
            } else {
                fns.current().op1(op);
            }
        },
        Expr::BinOp(ref e1, op, ref e2) => {
            compile_expr(e2.borrow(), fns, env)?;
            compile_expr(e1.borrow(), fns, env)?;
            if let BinOp::Apply = op {
                fns.current().call(1);
            } else {
                fns.current().op2(op);
            }
        }
    }
    Ok(())
}
