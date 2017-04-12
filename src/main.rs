#![recursion_limit = "200"]
#[macro_use]
extern crate pest;
extern crate libquantum;

mod string_table;
mod ast;
mod parser;
mod backend;

use parser::Rdp;

use pest::prelude::*;

use std::io::{self, Read};

fn print_line_from_pos(program: &String, pos: usize) {
    let mut line_start = pos;
    let mut line_end = pos;
    let chars: Vec<_> = program.chars().collect();
    println!("{:?}", chars);
    while line_start > 0 && chars[line_start] != '\n' {
        line_start -= 1;
    }
    while line_end < chars.len()-1 && chars[line_end] != '\n' {
        line_end += 1;
    }
    println!("{}", &chars[line_start..line_end].iter().cloned().collect::<String>());
    println!("{marker:>width$}", marker = '^', width = pos - line_start);
}

fn main() {
    let mut buffer = String::new();
    let _ = io::stdin().read_to_string(&mut buffer).unwrap();
    let mut parser = Rdp::new(StringInput::new(buffer.as_str()));
    if !parser.program() || !parser.end() {
        let (rules, pos) = parser.expected();
        print_line_from_pos(&buffer, pos);
        print!("[Parsing Error] Expected one of: ");
        for rule in rules.iter() {
            print!("{:?} ", rule);
        }
        println!();
        return;
    }
    let ast = parser.parse();
    match backend::compiler::compile(&ast) {
        Ok(program) => backend::interpreter::interpret(program),
        Err(err) => println!("[Compile Error] {}", err),
    }
}
