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
use std::rc::Rc;
use std::vec::Vec;

use backend::runtime::value::Value;

#[derive(Debug, Clone)]
pub struct ArrayObject {
    start: usize,
    end: usize,
    array: Rc<RefCell<Vec<Value>>>, 
}

impl ArrayObject {
    pub fn from_vec(v: Vec<Value>) -> ArrayObject {
        ArrayObject {
            start: 0,
            end: v.len(),
            array: Rc::new(RefCell::new(v)),
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn get(&self, idx: usize) -> Value {
        if idx > self.end - self.start {
            panic!("Invalid index '{}' into Array.", idx);
        }
        let start = self.start;
        self.array.borrow()[start + idx].clone()
    }

    pub fn put(&mut self, idx: usize, value: Value) {
        if idx > self.end - self.start {
            panic!("Invalid index '{}' into Array.", idx);
        }
        let start = self.start;
        self.array.borrow_mut()[start + idx] = value;
    }

    pub fn push_front(&mut self, value: Value) {
        if Rc::strong_count(&self.array) != 1 {
            panic!("Must only change array size when no views exist.");
        }
        self.array.borrow_mut().insert(self.start, value);
        self.end += 1;
    }

    pub fn push_back(&mut self, value: Value) {
        if Rc::strong_count(&self.array) != 1 {
            panic!("Must only change array size when no views exist.");
        }
        self.array.borrow_mut().insert(self.end, value);
        self.end += 1;
    }

    pub fn slice(&self, lb: usize, ub: usize) -> Value {
        if lb >= ub || ub > self.end - self.start {
            panic!("Invalid slice indicies '{}:{}' into Array.", lb, ub);
        }
        Value::Array(ArrayObject {
            start: self.start + lb,
            end: self.start + ub,
            array: self.array.clone(),
        })
    }

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        out.push('[');
        out.push(' ');
        let start = self.start;
        let end = self.end;
        for i in self.array.borrow()[start..end].iter() {
            out.push_str(&(i.clone().as_string())[..]);
            out.push(' ');
        }
        out.push(']');
        out
    }
}
