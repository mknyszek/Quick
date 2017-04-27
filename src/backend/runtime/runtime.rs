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

use backend::runtime::value::Value;

use std::f64;
use std::vec::Vec;

pub struct IRTEntry {
    pub irr: &'static Fn(&mut Vec<Value>),
    pub rev: &'static Fn(&mut Vec<Value>, &mut Vec<Value>),
    pub inv: &'static Fn(&mut Vec<Value>, &mut Vec<Value>)
}

pub struct IRTFunction {
    pub entry: IRTEntry,
    pub arity: usize,
}

irt_table! {
    fn[stack] len(1) {
        let s = stack.pop().unwrap();
        stack.push(s.len());
    }

    fn[stack] get(2) {
        let i = stack.pop().unwrap();
        let s = stack.pop().unwrap();
        stack.push(s.get(i));
    }

    fn[stack] put(3) {
        let e = stack.pop().unwrap();
        let i = stack.pop().unwrap();
        let s = stack.pop().unwrap();
        stack.push(s.put(i, e));
    }

    fn[stack] cat(2) {
        let s2 = stack.pop().unwrap();
        let s1 = stack.pop().unwrap();
        stack.push(s1.cat(s2));
    }

    fn[stack] ceil(1) { math_irt_fn!(stack, ceil); }
    fn[stack] floor(1) { math_irt_fn!(stack, floor); }
    fn[stack] round(1) { math_irt_fn!(stack, round); }
    fn[stack] abs(1) { math_irt_fn!(stack, abs); }
    fn[stack] ln(1) { math_irt_fn!(stack, ln); }
    fn[stack] log2(1) { math_irt_fn!(stack, log2); }
    fn[stack] log10(1) { math_irt_fn!(stack, log10); }
    fn[stack] sqrt(1) { math_irt_fn!(stack, sqrt); }
    fn[stack] cos(1) { math_irt_fn!(stack, cos); }
    fn[stack] sin(1) { math_irt_fn!(stack, sin); }
    fn[stack] tan(1) { math_irt_fn!(stack, tan); }
    fn[stack] acos(1) { math_irt_fn!(stack, acos); }
    fn[stack] asin(1) { math_irt_fn!(stack, asin); }
    fn[stack] atan(1) { math_irt_fn!(stack, atan); }

    fn[stack] pow(2) {
        let e = stack.pop().unwrap();
        let s = stack.pop().unwrap();
        stack.push(s.pow(e));
    }

    fn[stack] pi(0) {
        stack.push(Value::Float(f64::consts::PI));
    }

    fn[stack] e(0) {
        stack.push(Value::Float(f64::consts::E));
    } 

    fn[stack] hadamard(1) { qureg_irt_fn_t!(stack, hadamard); }

    fn[stack] sigx(1) { qureg_irt_fn_t!(stack, sigma_x); }
    fn[stack] sigy(1) { qureg_irt_fn_t!(stack, sigma_y); }
    fn[stack] sigz(1) { qureg_irt_fn_t!(stack, sigma_z); }

    fn[stack] rx(2) { qureg_irt_fn_t_g!(stack, rotate_x); }
    fn[stack] ry(2) { qureg_irt_fn_t_g!(stack, rotate_y); }
    fn[stack] rz(2) { qureg_irt_fn_t_g!(stack, rotate_z); }

    fn[stack] phase(2)   { qureg_irt_fn_t_g!(stack, phase); }
    fn[stack] phaseby(2) { qureg_irt_fn_t_g!(stack, phaseby); }

    fn[stack] cnot(2) {
        let mut t = stack.pop().unwrap().as_qureg();
        let mut c = stack.pop().unwrap().as_qureg();
        t.cnot(&mut c);
        stack.push(Value::QuReg(t));
    }

    fn[stack] cphase(2) {
        let mut t = stack.pop().unwrap().as_qureg();
        let mut c = stack.pop().unwrap().as_qureg();
        t.cphase(&mut c);
        stack.push(Value::QuReg(t));
    }

    fn[stack] cphaseby(3) {
        let g = stack.pop().unwrap().as_float();
        let mut t = stack.pop().unwrap().as_qureg();
        let mut c = stack.pop().unwrap().as_qureg();
        t.cphaseby(&mut c, g);
        stack.push(Value::QuReg(t));
    }

    fn[stack] measure(1) {
        let s = stack.pop().unwrap();
        let value = match s {
            Value::QuReg(mut q) => Value::Int(q.measure()),
            _ => panic!("Measurement only available for QuReg."),
        };
        stack.push(value);
    }
}

pub fn printf(fmt: &String, args: &[Value]) {
    let mut out: Vec<char> = Vec::with_capacity(fmt.len());
    let mut arg = 0;
    let mut escaping = false;
    for c in fmt.chars() {
        if escaping {
            match c {
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                '\"' => out.push('\"'),
                _ => out.push(c),
            }
            escaping = false;
        } else {
            match c {
                // TODO: Verify that not too few args
                '@' => {
                    if arg >= args.len() {
                        out.push('@');
                    } else {
                        for c in args[arg].clone().as_string().chars() {
                            out.push(c);
                        }
                        arg += 1;
                    }
                },
                '\\' => escaping = true,
                _ => out.push(c),
            }
        }
    }
    let s: String = out.into_iter().collect();
    print!("{}", s);
}
