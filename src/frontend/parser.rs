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
use util::ops::*;
use util::string_table::{self, StringToken};

use pest::prelude::*;

use std::collections::LinkedList;

impl_rdp! {
    grammar! {
        // Program is made up of statements
        program = { soi ~ stmt* ~ eoi }

        // Statements end with semi-colon
        stmt = {
            func_stmt |
            fore_stmt |
            forl_stmt |
            while_stmt |
            var_stmt |
            print_stmt |
            block_stmt |
            ret_stmt |
            uif_stmt |
            expr_stmt
        }

        // Types of statements
        func_stmt  = { ["func"] ~ iden ~ iden_list ~ (expr ~ [";"] | block_expr) }
        var_stmt   = { ["var"] ~ iden ~ ["="] ~ expr ~ [";"] }
        block_stmt = { blk_s ~ stmt* ~ blk_e }
        while_stmt = { ["while"] ~ ["("] ~ expr ~ [")"] ~ stmt }
        fore_stmt  = { ["foreach"] ~ ["("] ~ iden ~ ["in"] ~ expr ~ [")"] ~ stmt }
        forl_stmt  = { ["for"] ~ ["("] ~ iden ~ ["in"] ~ expr ~ [".."] ~ expr ~ [")"] ~ stmt }
        expr_stmt  = { expr ~ [";"] }
        uif_stmt   = { ["if*"] ~ ["("] ~ expr ~ [")"] ~ stmt }
        print_stmt = { ["print"] ~ lst_s ~ string ~ ([","] ~ arg)* ~ lst_e ~ [";"] }
        ret_stmt   = { ["ret"] ~ expr ~ [";"] }

        // Most everything else is an expression
        expr = _{
            { rexpr }
            func = {< apply }
            chng = { cat }
            lgc  = { and | or }
            cond = { le | ge | lt | gt | eq | ne }
            sum  = { plus  | minus }
            prod = { times | slash | perc }
            exp  = {< pow }
            bit  = { band | bor | bxor }
        }
        
        rexpr = _{ sexpr | ["("] ~ expr ~ [")"] | lit | iden }

        sexpr = _{
            if_expr | 
            call_expr |
            move_expr |
            unary_expr |
            alloc_expr |
            block_expr |
            array_expr |
            assign_expr |
            put_expr |
            get_expr
        }

        lit   = _{ float | snum | blit }

        // Operators for matching later
        plus  =  { ["+"] }
        minus =  { ["-"] }
        times =  { ["*"] }
        slash =  { ["/"] }
        perc  =  { ["%"] }
        pow   =  { ["^^"] }
        lt    =  { ["<"] }
        gt    =  { [">"] }
        le    =  { ["<="] }
        ge    =  { [">="] }
        eq    =  { ["=="] }
        ne    =  { ["!="] }
        and   =  { ["and"] }
        or    =  { ["or"] }
        not   =  { ["not"] }
        band  =  { ["&"] }
        bor   =  { ["|"] }
        bxor  =  { ["^"] }
        bnot  =  { ["~"] }
        cat   =  { ["><"] }
        len   =  { ["#"] }
        apply =  { ["$"] }
        blk_s =  { ["{"] }
        blk_e =  { ["}"] }
        lst_s =  { ["("] }
        lst_e =  { [")"] }
        arr_s =  { ["["] }
        arr_e =  { ["]"] }

        // Expressions to match
        if_expr     = { ["if"] ~ ["("] ~ expr ~ [")"] ~ expr ~ ["else"] ~ expr }
        block_expr  = { blk_s ~ stmt* ~ expr ~ blk_e }
        call_expr   = { caller ~ arg_list }
        assign_expr = { iden ~ ["="] ~ expr }
        get_expr    = { iden ~ ["["] ~ expr ~ ["]"] }
        put_expr    = { iden ~ ["["] ~ expr ~ ["]"] ~ ["="] ~ expr }
        array_expr  = { arr_s ~ (arg ~ ([","] ~ arg)*)? ~ arr_e } 
        alloc_expr  = { ["|"] ~ expr ~ [">"] }
        unary_expr  = { (apply | not | bnot | minus | len) ~ rexpr }
        move_expr   = { ["`"] ~ iden }

        // Helper rules
        arg       = { expr }
        iden_list = _{ ["("] ~ (iden ~ ([","] ~ iden)*)? ~ [")"] }
        arg_list  = _{ lst_s ~ (arg ~ ([","] ~ arg)*)? ~ lst_e }
        caller    = _{ iden | ["("] ~ expr ~ [")"] }

        // Literals and identifiers
        iden   = @{ (['a'..'z'] | ['A'..'Z'] | ["_"] | ["@"]) ~ (['a'..'z'] | ['A'..'Z'] | ["_"] | ['0'..'9'])* } 
        snum   = @{ ["0"] | (["-"]? ~ ['1'..'9'] ~ ['0'..'9']*) }
        float  = @{ ["-"]? ~ ['0'..'9']+ ~ (["."] ~ ['0'..'9']+)? ~ ["f"] }
        blit   = { ["true"] | ["false"] }
        chr    = _{ !(["\""]) ~ any }
        string = @{ ["\""] ~ (["\\\""] | chr)* ~ ["\""] }

        // Ignore whitespace
        whitespace = _{ [" "] | ["\n"] | ["\r"] | ["\t"] }

        // Ignore comment
        comment = _{
              ["/*"] ~ (!(["/"] | ["*"]) ~ any)* ~ ["*/"] 
            | ["//"] ~ (!(["\r"] | ["\n"]) ~ any)* ~ (["\n"] | ["\r\n"] | ["\r"] | eoi) 
        }
    }

    process! {
        parse(&self) -> Ast {
            (_: program, stmts: _stmt_list()) => stmts
        }
        _iden_list(&self) -> LinkedList<StringToken> {
            (&head: iden, mut rest: _iden_list()) => {
                rest.push_front(string_table::insert(head));
                rest
            },
            () => LinkedList::new()
        }
        _stmt(&self) -> Stmt {
            (_: func_stmt, &name: iden, params: _iden_list(), body: _expr()) => {
                Stmt::DefFunc(string_table::insert(name), params, body)
            },
            (_: var_stmt, &i: iden, e: _expr()) => Stmt::DefVar(string_table::insert(i), e),
            (_: block_stmt, _: blk_s, stmts: _stmt_list(), _: blk_e) => {
                Stmt::Block(stmts)
            },
            (_: while_stmt, pred: _expr(), _: stmt, body: _stmt()) => Stmt::While(pred, Box::new(body)),
            (_: fore_stmt, &name: iden, iter: _expr(), _: stmt, body: _stmt()) => {
                Stmt::ForEach(string_table::insert(name), iter, Box::new(body))
            },
            (_: forl_stmt, &name: iden, start: _expr(), end: _expr(), _: stmt, body: _stmt()) => {
                Stmt::ForLoop(string_table::insert(name), start, end, Box::new(body))
            },
            (_: ret_stmt, value: _expr()) => Stmt::Return(value),
            (_: uif_stmt, pred: _expr(), _: stmt, body: _stmt()) => Stmt::UnIf(pred, Box::new(body)),
            (_: expr_stmt, e: _expr()) => Stmt::Expr(e),
            (_: print_stmt, _: lst_s, &s: string, args: _arg_list(), _: lst_e) => {
                let s_len = s.len();
                Stmt::Print(string_table::insert(&s[1..s_len-1]), args) 
            },
        }
        _stmt_list(&self) -> LinkedList<Stmt> {
            (_: stmt, head: _stmt(), mut rest: _stmt_list()) => {
                rest.push_front(head);
                rest
            },
            () => LinkedList::new()
        }
        _expr(&self) -> Expr {
            (&i: iden) => Expr::Ref(string_table::insert(i)),
            (&blit: blit) => Expr::Bool(blit.parse::<bool>().unwrap()),
            (&num: snum) => Expr::Int(num.parse::<i64>().unwrap()),
            (&num: float) => {
                // Truncate suffix "f" before parse
                let num_len = num.len();
                Expr::Float(num[0..num_len-1].parse::<f64>().unwrap())
            },
            (_: if_expr, pred: _expr(), then: _expr(), other: _expr()) => {
                Expr::If(Box::new(pred), Box::new(then), Box::new(other))
            },
            (_: alloc_expr, n: _expr()) => {
                Expr::UnOp(UnOp::QAlloc, Box::new(n))
            },
            (_: block_expr, _: blk_s, stmts: _stmt_list(), result: _expr(), _: blk_e) => {
                Expr::Block(stmts, Box::new(result))
            },
            (_: call_expr, func: _expr(), _: lst_s, args: _arg_list(), _: lst_e) => {
                Expr::Call(Box::new(func), args)
            },
            (_: move_expr, &i: iden) => {
                Expr::Move(string_table::insert(i))
            },
            (_: assign_expr, &var: iden, value: _expr()) => {
                Expr::Assign(string_table::insert(var), Box::new(value))
            },
            (_: get_expr, &var: iden, index: _expr()) => {
                Expr::Get(string_table::insert(var), Box::new(index))
            },
            (_: put_expr, &var: iden, index: _expr(), value: _expr()) => {
                Expr::Put(string_table::insert(var), Box::new(index), Box::new(value))
            },
            (_: array_expr, _: arr_s, args: _arg_list(), _: arr_e) => {
                Expr::Array(args)
            },
            (_: unary_expr, op, e: _expr()) => {
                Expr::UnOp(match op.rule {
                    Rule::minus => UnOp::Neg,
                    Rule::not => UnOp::Not,
                    Rule::bnot => UnOp::BNot,
                    Rule::len => UnOp::Len,
                    Rule::apply => UnOp::Invoke,
                    _ => unreachable!(),
                }, Box::new(e))
            },
            (_: func, e1: _expr(), op, e2: _expr()) => {
                Expr::BinOp(Box::new(e1), match op.rule {
                    Rule::apply => BinOp::Apply,
                    _ => unreachable!(),
                }, Box::new(e2))
            },
            (_: chng, e1: _expr(), op, e2: _expr()) => {
                Expr::BinOp(Box::new(e1), match op.rule {
                    Rule::cat => BinOp::Cat,
                    _ => unreachable!(),
                }, Box::new(e2))
            },
            (_: lgc, e1: _expr(), op, e2: _expr()) => {
                Expr::BinOp(Box::new(e1), match op.rule {
                    Rule::and => BinOp::And,
                    Rule::or => BinOp::Or,
                    _ => unreachable!(),
                }, Box::new(e2))
            },
            (_: cond, e1: _expr(), op, e2: _expr()) => {
                Expr::BinOp(Box::new(e1), match op.rule {
                    Rule::lt => BinOp::Lt,
                    Rule::gt => BinOp::Gt,
                    Rule::le => BinOp::Le,
                    Rule::ge => BinOp::Ge,
                    Rule::eq => BinOp::Eq,
                    Rule::ne => BinOp::Ne,
                    _ => unreachable!(),
                }, Box::new(e2))
            },
            (_: sum, e1: _expr(), op, e2: _expr()) => {
                Expr::BinOp(Box::new(e1), match op.rule {
                    Rule::plus => BinOp::Add,
                    Rule::minus => BinOp::Sub,
                    _ => unreachable!(),
                }, Box::new(e2))
            },
            (_: prod, e1: _expr(), op, e2: _expr()) => {
                Expr::BinOp(Box::new(e1), match op.rule {
                    Rule::times => BinOp::Mul,
                    Rule::slash => BinOp::Div,
                    Rule::perc => BinOp::Rem,
                    _ => unreachable!(),
                }, Box::new(e2))
            },
            (_: exp, e1: _expr(), op, e2: _expr()) => {
                Expr::BinOp(Box::new(e1), match op.rule {
                    Rule::pow => BinOp::Pow,
                    _ => unreachable!(),
                }, Box::new(e2))
            },
            (_: bit, e1: _expr(), op, e2: _expr()) => {
                Expr::BinOp(Box::new(e1), match op.rule {
                    Rule::band => BinOp::BAnd,
                    Rule::bor => BinOp::BOr,
                    Rule::bxor => BinOp::BXor,
                    _ => unreachable!(),
                }, Box::new(e2))
            },
        }
        _arg_list(&self) -> LinkedList<Expr> {
            (_: arg, head: _expr(), mut rest: _arg_list()) => {
                rest.push_front(head);
                rest
            },
            () => LinkedList::new()
        }
    }
}
