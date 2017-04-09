
use ast::*;
use pest::prelude::*;

use std::collections::LinkedList;

impl_rdp! {
    grammar! {
        // Program is made up of statements
        program = { soi ~ stmt* ~ eoi }

        // Statements end with semi-colon
        stmt = { func_stmt | while_stmt | var_stmt | expr_stmt }

        // Types of statements
        func_stmt  = { ["func"] ~ iden ~ iden_list ~ (expr ~ [";"] | block_expr) }
        var_stmt   = { ["var"] ~ iden ~ ["="] ~ expr ~ [";"] }
        while_stmt = { ["while"] ~ ["("] ~ expr ~ [")"] ~ (expr ~ [";"] | block_expr) }
        expr_stmt  = { expr ~ [";"] }

        // Most everything else is an expression
        expr = _{
            { ["("] ~ expr ~ [")"] | if_expr | block_expr | call_expr | assign_expr | unary_expr | float | snum | blit | iden }
            //chng = { cat }
            lgc  = { and | or }
            cond = { lt | gt | eq | ne }
            sum  = { plus  | minus }
            prod = { times | slash }
            bit  = { band | bor | bxor }
        }

        // Operators for matching later
        plus  =  { ["+"] }
        minus =  { ["-"] }
        times =  { ["*"] }
        slash =  { ["/"] }
        lt    =  { ["<"] }
        gt    =  { [">"] }
        eq    =  { ["=="] }
        ne    =  { ["!="] }
        and   =  { ["and"] }
        or    =  { ["or"] }
        not   =  { ["not"] }
        band  =  { ["&"] }
        bor   =  { ["|"] }
        bxor  =  { ["^"] }
        bnot  =  { ["~"] }
        //cat   =  { ["><"] }

        // Expressions to match
        if_expr     = { ["if"] ~ ["("] ~ expr ~ [")"] ~ expr ~ ["else"] ~ expr }
        block_expr  = { ["{"] ~ stmt* ~ expr ~ ["}"] }
        call_expr   = { iden ~ ["("] ~ (arg ~ ([","] ~ arg)*)? ~ [")"] }
        assign_expr = { iden ~ ["="] ~ expr }
        //take_expr   = { iden ~ ["["] ~ expr ~ ["]"] }
        //slice_expr  = { iden ~ ["["] ~ expr ~ [":"] ~ expr ~ ["]"] }
        //alloc_expr  = { ["|"] ~ ["["] ~ expr ~ ["]"] ~ [">"] }
        unary_expr  = { (not | bnot | minus) ~ expr }

        // Helper rules
        arg       = { expr }
        iden_list = _{ ["("] ~ (iden ~ ([","] ~ iden)*)? ~ [")"] }

        // Literals and identifiers
        iden  = @{ (['a'..'z'] | ['A'..'Z'] | ["_"]) ~ (['a'..'z'] | ['A'..'Z'] | ["_"] | ['0'..'9'])* } 
        snum  = @{ ["0"] | (["-"]? ~ ['1'..'9'] ~ ['0'..'9']*) }
        float = { ["-"]? ~ ['0'..'9']+ ~ (["."] ~ ['0'..'9']+)? ~ ["f"] }
        blit  = { ["true"] | ["false"] }

        // Ignore whitespace
        whitespace = _{ [" "] | ["\n"] | ["\r"] | ["\t"] }

        // Ignore comment
        comment = _{
              ["/*"] ~ (!(["/"] | ["*"]) ~ any)* ~ ["*/"] 
            | ["//"] ~ (!(["\r"] | ["\n"]) ~ any)* ~ (["\n"] | ["\r\n"] | ["\r"] | eoi) 
        }
    }

    process! {
        parse(&self) -> Program {
            (_: program, stmts: _stmt_list()) => stmts
        }
        _iden_list(&self) -> LinkedList<Iden> {
            (&head: iden, mut rest: _iden_list()) => {
                rest.push_front(head.parse::<String>().unwrap());
                rest
            },
            () => LinkedList::new()
        }
        _stmt(&self) -> Stmt {
            (_: func_stmt, &name: iden, params: _iden_list(), body: _expr()) => {
                Stmt::DefFunc(name.parse::<String>().unwrap(), params, body)
            },
            (_: var_stmt, &i: iden, e: _expr()) => Stmt::DefVar(i.parse::<String>().unwrap(), e),
            (_: while_stmt, pred: _expr(), body: _expr()) => Stmt::While(pred, body),
            (_: expr_stmt, e: _expr()) => Stmt::Expr(e),
        }
        _stmt_list(&self) -> LinkedList<Stmt> {
            (_: stmt, head: _stmt(), mut rest: _stmt_list()) => {
                rest.push_front(head);
                rest
            },
            () => LinkedList::new()
        }
        _expr(&self) -> Expr {
            (&i: iden) => Expr::Ref(i.parse::<String>().unwrap()),
            (&blit: blit) => Expr::Bool(blit.parse::<bool>().unwrap()),
            (&num: snum) => Expr::Int(num.parse::<i64>().unwrap()),
            (&num: float) => Expr::Float(num.parse::<f64>().unwrap()),
            (_: if_expr, pred: _expr(), then: _expr(), other: _expr()) => {
                Expr::If(Box::new(pred), Box::new(then), Box::new(other))
            },
            (_: block_expr, stmts: _stmt_list(), result: _expr()) => {
                Expr::Block(stmts, Box::new(result))
            },
            (_: call_expr, &func: iden, args: _arg_list()) => {
                Expr::Call(func.parse::<String>().unwrap(), args)
            },
            (_: assign_expr, &var: iden, value: _expr()) => {
                Expr::Assign(var.parse::<String>().unwrap(), Box::new(value))
            },
            (_: unary_expr, op, e: _expr()) => {
                Expr::UnOp(match op.rule {
                    Rule::minus => UnOp::Neg,
                    Rule::not => UnOp::Not,
                    Rule::bnot => UnOp::BNot,
                    _ => unreachable!(),
                }, Box::new(e))
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
