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
    fn[stack, aux] len(1) {
        (regular) = { simple_irt_fn!(stack, len);  }
        (reverse) = { simple_irt_rev_fn!(stack, aux, len); }
        (inverse) = { simple_irt_inv_fn!(stack, aux, len); }
    }

    fn[stack, aux] get(2) {
        (regular) = { simple_irt_fn!(stack, get, i);  }
        (reverse) = { simple_irt_rev_fn!(stack, aux, get, i); }
        (inverse) = { simple_irt_inv_fn!(stack, aux, get, i); }
    }

    fn[stack, aux] slice(3) {
        (regular) = { simple_irt_fn!(stack, slice, i1, i2);  }
        (reverse) = { simple_irt_rev_fn!(stack, aux, slice, i1, i2); }
        (inverse) = { simple_irt_inv_fn!(stack, aux, slice, i1, i2); }
    }

    fn[stack, aux] put(3) {
        (regular) = { simple_irt_fn!(stack, put, i, e);  }
        (reverse) = {
            let e = stack.pop().unwrap();
            let i = stack.pop().unwrap();
            let s = stack.pop().unwrap();
            let old_e = s.clone().get(i.clone());
            aux.push(e.clone());
            aux.push(i.clone());
            aux.push(s.clone());
            aux.push(old_e);
            stack.push(s.put(i, e));
        }
        (inverse) = {
            let _ = stack.pop().unwrap(); 
            let old_e = aux.pop().unwrap();
            let e = aux.pop().unwrap();
            let i = aux.pop().unwrap();
            let s = aux.pop().unwrap();
            s.clone().put(i.clone(), old_e);
            stack.push(s);
            stack.push(i);
            stack.push(e);
        }
    }

    fn[stack, aux] cat(2) {
        (regular) = { simple_irt_fn!(stack, cat, s2);  }
        (reverse) = { simple_irt_rev_fn!(stack, aux, cat, s2); }
        (inverse) = { simple_irt_inv_fn!(stack, aux, cat, s2); }
    }

    fn[stack, aux] qalloc(2) {
        (regular) = { simple_irt_fn!(stack, qalloc, i);  }
        (reverse) = { simple_irt_rev_fn!(stack, aux, qalloc, i); }
        (inverse) = { simple_irt_inv_fn!(stack, aux, qalloc, i); }
    }

    fn[stack, _aux] ceil(1) { math_irt_fn!(stack, ceil); }
    fn[stack, _aux] floor(1) { math_irt_fn!(stack, floor); }
    fn[stack, _aux] round(1) { math_irt_fn!(stack, round); }
    fn[stack, _aux] abs(1) { math_irt_fn!(stack, abs); }
    fn[stack, _aux] ln(1) { math_irt_fn!(stack, ln); }
    fn[stack, _aux] log2(1) { math_irt_fn!(stack, log2); }
    fn[stack, _aux] log10(1) { math_irt_fn!(stack, log10); }
    fn[stack, _aux] sqrt(1) { math_irt_fn!(stack, sqrt); }
    fn[stack, _aux] cos(1) { math_irt_fn!(stack, cos); }
    fn[stack, _aux] sin(1) { math_irt_fn!(stack, sin); }
    fn[stack, _aux] tan(1) { math_irt_fn!(stack, tan); }
    fn[stack, _aux] acos(1) { math_irt_fn!(stack, acos); }
    fn[stack, _aux] asin(1) { math_irt_fn!(stack, asin); }
    fn[stack, _aux] atan(1) { math_irt_fn!(stack, atan); }

    fn[stack, _aux] pow(2) {
        let e = stack.pop().unwrap();
        let s = stack.pop().unwrap();
        stack.push(s.pow(e));
    }

    fn[stack, _aux] pi(0) {
        stack.push(Value::Float(f64::consts::PI));
    }

    fn[stack, _aux] e(0) {
        stack.push(Value::Float(f64::consts::E));
    } 

    fn[stack, aux] hadamard(1) {
        (regular) = { qureg_irt_fn_t!(stack, hadamard); }
        (reverse) = { qureg_irt_rev_fn_t!(stack, aux, hadamard); }
        (inverse) = { qureg_irt_inv_fn_t!(stack, aux, hadamard); }
    }

    fn[stack, aux] sigx(1) {
        (regular) = { qureg_irt_fn_t!(stack, sigma_x); }
        (reverse) = { qureg_irt_rev_fn_t!(stack, aux, sigma_x); }
        (inverse) = { qureg_irt_inv_fn_t!(stack, aux, sigma_x); }
    }

    fn[stack, aux] sigy(1) {
        (regular) = { qureg_irt_fn_t!(stack, sigma_y); }
        (reverse) = { qureg_irt_rev_fn_t!(stack, aux, sigma_y); }
        (inverse) = { qureg_irt_inv_fn_t!(stack, aux, sigma_y); }
    }

    fn[stack, aux] sigz(1) {
        (regular) = { qureg_irt_fn_t!(stack, sigma_z); }
        (reverse) = { qureg_irt_rev_fn_t!(stack, aux, sigma_z); }
        (inverse) = { qureg_irt_inv_fn_t!(stack, aux, sigma_z); }
    }

    fn[stack, aux] rx(2) { 
        (regular) = { qureg_irt_fn_t_g!(stack, rotate_x); }
        (reverse) = { qureg_irt_rev_fn_t_g!(stack, aux, rotate_x); }
        (inverse) = { qureg_irt_inv_fn_t_g!(stack, aux, rotate_x); }
    }

    fn[stack, aux] ry(2) { 
        (regular) = { qureg_irt_fn_t_g!(stack, rotate_y); }
        (reverse) = { qureg_irt_rev_fn_t_g!(stack, aux, rotate_y); }
        (inverse) = { qureg_irt_inv_fn_t_g!(stack, aux, rotate_y); }
    }

    fn[stack, aux] rz(2) { 
        (regular) = { qureg_irt_fn_t_g!(stack, rotate_z); }
        (reverse) = { qureg_irt_rev_fn_t_g!(stack, aux, rotate_z); }
        (inverse) = { qureg_irt_inv_fn_t_g!(stack, aux, rotate_z); }
    }

    fn[stack, aux] phase(2) {
        (regular) = { qureg_irt_fn_t_g!(stack, phase); }
        (reverse) = { qureg_irt_rev_fn_t_g!(stack, aux, phase); }
        (inverse) = { qureg_irt_inv_fn_t_g!(stack, aux, phase); }
    }

    fn[stack, aux] phaseby(2) {
        (regular) = { qureg_irt_fn_t_g!(stack, phaseby); }
        (reverse) = { qureg_irt_rev_fn_t_g!(stack, aux, phaseby); }
        (inverse) = { qureg_irt_inv_fn_t_g!(stack, aux, phaseby); }
    } 

    fn[stack, aux] all(1) {
        (reverse) = {
            let mut t = stack.pop().unwrap().as_qureg();
            let s = t.all();
            aux.push(Value::QuReg(t));
            stack.push(Value::QuReg(s));
        }
        (inverse) = {
            let s = stack.pop().unwrap().as_qureg();
            let mut t = aux.pop().unwrap().as_qureg();
            s.iall(&mut t);
            stack.push(Value::QuReg(t));
        }
    }

    fn[stack, _aux] cnot(2) {
        let mut t = stack.pop().unwrap().as_qureg();
        let mut c = stack.pop().unwrap().as_qureg();
        t.cnot(&mut c);
        stack.push(Value::QuReg(t));
    }

    fn[stack, _aux] cflip(2) {
        let mut t = stack.pop().unwrap().as_qureg();
        let mut c = stack.pop().unwrap().as_qureg();
        t.cflip(&mut c);
        stack.push(Value::QuReg(t));
    }

    fn[stack, _aux] toffoli(3) {
        let mut t = stack.pop().unwrap().as_qureg();
        let mut c1 = stack.pop().unwrap().as_qureg();
        let mut c2 = stack.pop().unwrap().as_qureg();
        t.toffoli(&mut c1, &mut c2);
        stack.push(Value::QuReg(t));
    }

    fn[stack, _aux] cphase(2) {
        let mut t = stack.pop().unwrap().as_qureg();
        let mut c = stack.pop().unwrap().as_qureg();
        t.cphase(&mut c);
        stack.push(Value::QuReg(t));
    }

    fn[stack, _aux] cphaseby(3) {
        let g = stack.pop().unwrap().as_float();
        let mut t = stack.pop().unwrap().as_qureg();
        let mut c = stack.pop().unwrap().as_qureg();
        t.cphaseby(&mut c, g);
        stack.push(Value::QuReg(t));
    }

    fn[stack, _aux] measure(1) {
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
