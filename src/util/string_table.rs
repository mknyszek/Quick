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

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::vec::Vec;

thread_local!(static STRING_TABLE: RefCell<StringTable> = RefCell::new(StringTable::new()));

pub fn insert<'h>(value: &'h str) -> StringToken {
    STRING_TABLE.with(|st| -> StringToken {
        (st.borrow_mut()).insert(value)
    })
}

pub fn get(i: StringToken) -> Rc<String> {
    STRING_TABLE.with(|st| -> Rc<String> {
        (st.borrow()).get(i)
    })
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct StringToken {
    pub id: usize,
}

pub struct StringTable {
    map: HashMap<Rc<String>, StringToken>,
    table: Vec<Rc<String>>,
}

impl StringTable {
    pub fn new() -> StringTable {
        StringTable {
            map: HashMap::new(),
            table: Vec::new(),
        }
    }

    pub fn insert<'h>(&mut self, value: &'h str) -> StringToken {
        let boxed_value = Rc::new(value.to_string());
        match self.map.get(&boxed_value) {
            Some(token) => return *token,
            None => self.table.push(boxed_value.clone()),
        }
        let next_token = StringToken { id: self.table.len() };
        self.map.insert(boxed_value, next_token);
        next_token
    }

    pub fn get(&self, i: StringToken) -> Rc<String> {
        self.table[i.id-1].clone()
    }
}

impl fmt::Debug for StringTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.map)
    }
}

