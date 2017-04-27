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

#[macro_export]
macro_rules! return_error {
    ($($i:expr),*) => {{
        let mut s = String::new();
        write!(s, $($i),*).ok().unwrap();
        return Err(s);
    }}
}

#[macro_export]
macro_rules! unreachable {
    () => { panic!("Broken logic; unreachable point!"); }
}

#[macro_export]
macro_rules! unimplemented {
    () => { panic!("Not yet implemented."); }
}

#[macro_export]
macro_rules! invalid_call {
    ($f:ident) => { panic!("Attempted to call function {} as reversible.", stringify!($f)) }
}

macro_rules! irt_entry {
    ($f:ident, $s:ident, $b:block) => {
        IRTEntry {
            irr: &|$s| $b,
            rev: &|_, _| invalid_call!($f),
            inv: &|_, _| invalid_call!($f),
        }
    };
    ($f:ident, $s:ident, [$rs:ident] { normal => $n:stmt; reverse => $r:stmt; inverse => $i:stmt; }) => {
        IRTEntry {
            irr: &|$s| $n,
            rev: &|$s, $rs| $r,
            inv: &|$s, $rs| $i,
        }
    }
}

#[macro_export]
macro_rules! irt_table {
    ($(fn[$s:ident] $i:ident($n:expr) $t:tt )*) => {
        pub const IRT_STRINGS: &'static [&'static str] = &[
            $(stringify!($i)),*
        ];
        pub const IRT_TABLE: &'static [IRTFunction] = &[
            $(IRTFunction { entry: irt_entry!($i, $s, $t), arity: $n }),* 
        ];
    }
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
            let mut qm = self.qureg.borrow_mut();
            for i in self.start..self.end {
                qm.$f(i);
            }
        }
    }
}

#[macro_export]
macro_rules! qureg_fn_t_g {
    ($f:ident) => {
        pub fn $f(&mut self, gamma: f64) {
            let mut qm = self.qureg.borrow_mut();
            for i in self.start..self.end {
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
