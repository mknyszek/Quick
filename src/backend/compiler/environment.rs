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

use util::string_table::{self, StringToken};

use std::collections::HashMap;
use std::fmt::Write;
use std::vec::Vec;

pub struct Environment<T> {
    ids: Vec<HashMap<StringToken, T>>,
}

impl<T: Copy> Environment<T> {
    pub fn new() -> Environment<T> {
        let mut ids = Vec::new();
        ids.push(HashMap::new());
        Environment {
            ids: ids,
        }
    }

    pub fn push_scope(&mut self) {
        self.ids.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        let _ = self.ids.pop().unwrap();
    }

    pub fn add(&mut self, id: StringToken, data: T) -> Result<(), String> {
        let l = self.ids.len();
        if self.ids[l-1].contains_key(&id) {
            return_error!("Illegal redefinition of identifier '{}'", string_table::get(id));
        }
        self.ids[l-1].insert(id, data);
        Ok(())
    }

    pub fn find(&self, id: StringToken) -> Option<T> {
        for hm in self.ids.iter().rev() {
            if let Some(v) = hm.get(&id) {
                return Some(*v);
            }
        }
        None
    }
}

pub struct LocalEnvironment {
    ids: Environment<usize>,
    id_count: Vec<usize>,
    id_total: usize,
    id_max: usize,
}

impl LocalEnvironment {
    pub fn new() -> LocalEnvironment {
        let mut id_count = Vec::new();
        id_count.push(0);
        LocalEnvironment {
            ids: Environment::new(),
            id_count: id_count,
            id_total: 0,
            id_max: 0
        }
    }

    pub fn push_scope(&mut self) {
        self.ids.push_scope();
        self.id_count.push(0);
    }

    pub fn pop_scope(&mut self) {
        self.ids.pop_scope();
        let h = self.id_count.pop().unwrap();
        self.id_total -= h;
    }

    pub fn add_id(&mut self, id: StringToken) -> Result<usize, String> {
        self.ids.add(id, self.id_total)?;
        Ok(self.add_tmp())
    }

    pub fn add_tmp(&mut self) -> usize {
        let pos = self.id_total;
        let l = self.id_count.len();
        self.id_count[l-1] += 1;
        self.id_total += 1;
        if self.id_total > self.id_max {
            self.id_max = self.id_total;
        }
        pos
    }

    pub fn locals(&self) -> usize {
        self.id_max
    }

    pub fn find(&self, id: StringToken) -> Option<usize> {
        self.ids.find(id)
    }
}
