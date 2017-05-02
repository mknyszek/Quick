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
macro_rules! builtin_call {
    ($fns:ident, $f:ident, $a:expr) => {{
        match $fns.lookup(string_table::insert(stringify!($f))) {
            Some(ft) => $fns.current().func(ft),
            None => return_error!(concat!("Internal error: Undefined function ", stringify!($f))),
        }
        $fns.current().call($a);
    }};
    ($fns:ident, $env:ident, $f:ident, $a:expr, $($e:expr),*) => {{
        $(
        compile_expr($e.borrow(), $fns, $env)?;
        )*
        builtin_call!($fns, $f, $a);
    }};
}

#[macro_export]
macro_rules! builtin_rcall {
    ($fns:ident, $f:ident, $a:expr) => {{
        match $fns.lookup(string_table::insert(stringify!($f))) {
            Some(ft) => $fns.current().func(ft),
            None => return_error!(concat!("Internal error: Undefined function ", stringify!($f))),
        }
        $fns.current().rcall($a);
    }};
    ($fns:ident, $env:ident, $f:ident, $a:expr, $($e:expr),*) => {{
        $(
        compile_rev_expr($e.borrow(), $fns, $env)?;
        )*
        builtin_rcall!($fns, $f, $a);
    }};
}

#[macro_export]
macro_rules! builtin_icall {
    ($fns:ident, $f:ident, $a:expr) => {{
        $fns.current().icall($a);
    }};
    ($fns:ident, $env:ident, $f:ident, $a:expr, $($e:expr),*) => {{
        builtin_icall!($fns, $f, $a);
        $fns.current().discard(); // Discard function at the top
        $(
        compile_inv_expr($e.borrow(), $fns, $env)?;
        )*
    }};
}
