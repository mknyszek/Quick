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

macro_rules! invalid_call {
    ($f:ident) => { panic!("Attempted to call function {} as reversible.", stringify!($f)) }
}

macro_rules! irt_entry {
    ($f:ident, $s:ident, $rs:ident, { (regular) = $n:block (reverse) = $r:block (inverse) = $i:block }) => {
        IRTEntry {
            irr: &|$s| $n,
            rev: &|$s, $rs| $r,
            inv: &|$s, $rs| $i,
        }
    };
    ($f:ident, $s:ident, $rs:ident, $b:block) => {
        IRTEntry {
            irr: &|$s| $b,
            rev: &|_, _| invalid_call!($f),
            inv: &|_, _| invalid_call!($f),
        }
    };
}

#[macro_export]
macro_rules! irt_table {
    ($(fn[$s:ident, $rs:ident] $i:ident($n:expr) $t:tt)*) => {
        pub const IRT_STRINGS: &'static [&'static str] = &[
            $(stringify!($i)),*
        ];
        pub const IRT_TABLE: &'static [IRTFunction] = &[
            $(IRTFunction { entry: irt_entry!($i, $s, $rs, $t), arity: $n }),* 
        ];
    }
}

#[macro_export]
macro_rules! simple_irt_fn {
    ($stack:ident, $f:ident) => {
        let s_ = $stack.pop().unwrap(); 
        $stack.push(s_.$f());
    };
    ($stack:ident, $f:ident, $($a:ident),*) => {
        let r_ = {
            let l_ = &[$(stringify!($a)),*].len() + 1;
            let sl_ = $stack.len();
            let mut d_ = $stack.drain(sl_-l_..);
            let s_ = d_.next().unwrap(); 
            $(
            let $a = d_.next().unwrap();
            )*
            s_.$f($($a),*)
        };
        $stack.push(r_);
    };
}

#[macro_export]
macro_rules! simple_irt_rev_fn {
    ($stack:ident, $aux:ident, $f:ident) => {
        let s_ = $stack.pop().unwrap(); 
        $stack.push(s_.clone().$f());
        $aux.push(s_);
    };
    ($stack:ident, $aux:ident, $f:ident, $($a:ident),*) => {
        let r_ = {
            let l_ = &[$(stringify!($a)),*].len() + 1;
            let sl_ = $stack.len();
            let mut d_ = $stack.drain(sl_-l_..);
            let s_ = d_.next().unwrap(); 
            $aux.push(s_.clone());
            $(
            let $a = d_.next().unwrap();
            $aux.push($a.clone());
            )*
            s_.$f($($a),*)
        }; 
        $stack.push(r_);
    };
}

#[macro_export]
macro_rules! simple_irt_inv_fn {
    ($stack:ident, $aux:ident, $f:ident) => {
        let _ = $stack.pop().unwrap(); 
        let s_ = $aux.pop().unwrap(); 
        $stack.push(s_);
    };
    ($stack:ident, $aux:ident, $f:ident, $($a:ident),*) => {
        let _ = $stack.pop().unwrap(); 
        let l_ = &[$(stringify!($a)),*].len() + 1;
        let sl_ = $aux.len();
        let mut d_ = $aux.drain(sl_-l_..);
        let s_ = d_.next().unwrap(); 
        $(
        let $a = d_.next().unwrap();
        )*
        $stack.push(s_);
        $(
        $stack.push($a);
        )*
    };
}

#[macro_export]
macro_rules! math_irt_fn {
    ($stack:ident, $f:ident) => {
        let s = $stack.pop().unwrap();
        $stack.push(match s {
            Value::Int(v) => Value::Float((v as f64).$f()),
            Value::Float(v) => Value::Float(v.$f()),
            _ => panic!("sqrt only defined for Int and Float."),
        });
    }
}

#[macro_export]
macro_rules! qureg_fn_t {
    ($f:ident) => {
        pub fn $f(&mut self) {
            let start = self.raw_start();
            let end = self.raw_end();
            let mut qm = self.qureg.borrow_mut();
            for i in start..end {
                qm.$f(i);
            }
        }
    }
}

#[macro_export]
macro_rules! qureg_fn_t_g {
    ($f:ident) => {
        pub fn $f(&mut self, gamma: f64) {
            let start = self.raw_start();
            let end = self.raw_end();
            let mut qm = self.qureg.borrow_mut();
            for i in start..end {
                qm.$f(i, gamma as f32);
            }
        }
    }
}

#[macro_export]
macro_rules! qureg_irt_fn_t {
    ($stack:ident, $f:ident) => {
        let s = $stack.pop().unwrap();
        match s {
            Value::QuReg(mut q) => { q.$f(); $stack.push(Value::QuReg(q)); },
            _ => panic!(concat!(stringify!($f), " only available on quantum registers and bits.")),
        }
    }
}

#[macro_export]
macro_rules! qureg_irt_fn_t_g {
    ($stack:ident, $f:ident) => {
        let g = $stack.pop().unwrap();
        let s = $stack.pop().unwrap();
        match s {
            Value::QuReg(mut q) => { q.$f(g.as_float()); $stack.push(Value::QuReg(q)); },
            _ => panic!(concat!(stringify!($f), " only available on quantum registers and bits.")),
        }
    }
}

#[macro_export]
macro_rules! qureg_irt_rev_fn_t_g {
    ($stack:ident, $aux:ident, $f:ident) => {
        let g = $stack.pop().unwrap().as_float();
        let s = $stack.pop().unwrap();
        match s {
            Value::QuReg(mut q) => {
                q.$f(g);
                $aux.push(Value::Float(g));
                $stack.push(Value::QuReg(q));
            },
            _ => panic!(concat!(stringify!($f), " only available on quantum registers and bits.")),
        }
    }
}

#[macro_export]
macro_rules! qureg_irt_inv_fn_t_g {
    ($stack:ident, $aux:ident, $f:ident) => {
        let g = $aux.pop().unwrap().as_float();
        let s = $stack.pop().unwrap();
        match s {
            Value::QuReg(mut q) => { q.$f(-g); $stack.push(Value::QuReg(q)); },
            _ => panic!(concat!(stringify!($f), " only available on quantum registers and bits.")),
        }
    }
}
