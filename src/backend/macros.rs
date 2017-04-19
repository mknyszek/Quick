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
macro_rules! irt_table {
    ($(fn[$s:ident] $i:ident($n:expr) $b:block)*) => {
        $(
        pub fn $i($s: &mut Vec<Value>) {
            $b
        }
        )*
        pub static IRT_STRINGS: &'static [&'static str] = &[
            $(stringify!($i)),*
        ];
        pub static IRT_TABLE: &'static [IRTFunction] = &[
            $(IRTFunction { entry: $i, arity: $n }),* 
        ];
    }
}

#[macro_export]
macro_rules! qubit_irt_fn_t {
    ($stack:ident, $f:ident) => {
        let s = $stack.pop().unwrap();
        match s {
            Value::QuReg(ref q) => {
                let l = q.borrow().width();
                let mut qm = q.borrow_mut();
                for i in 0..l {
                    qm.$f(i);
                }
            },
            Value::Qubit(i, ref q) => q.borrow_mut().$f(i),
            _ => panic!(concat!(stringify!($f), " only available on quantum registers and bits.")),
        }
        $stack.push(s);
    }
}

#[macro_export]
macro_rules! qubit_irt_fn_t_g {
    ($stack:ident, $f:ident) => {
        let g = $stack.pop().unwrap();
        let s = $stack.pop().unwrap();
        match s {
            Value::QuReg(ref q) => {
                let l = q.borrow().width();
                let mut qm = q.borrow_mut();
                let gamma = g.as_float();
                for i in 0..l {
                    qm.$f(i, gamma as f32);
                }
            },
            Value::Qubit(i, ref q) => q.borrow_mut().$f(i, g.as_float() as f32),
            _ => panic!(concat!(stringify!($f), " only available on quantum registers and bits.")),
        }
        $stack.push(s);
    }
}
