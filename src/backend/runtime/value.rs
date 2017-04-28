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

use backend::runtime::array::ArrayObject;
use backend::runtime::qureg::QuRegObject;
use backend::bytecode::FunctionToken;

use std::ops::{Add, Sub, Mul, Div, Rem};

// TODO: This value representation is grossly suboptimal, but is good for
// quickly iterating. Try to replace this with something more general in
// the future. 
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Addr(usize),
    Int(i64),
    Bool(bool),
    Float(f64),
    Func(FunctionToken),
    Array(ArrayObject),
    QuReg(QuRegObject),
}

macro_rules! arith_method {
    ($i:ident) => {
        pub fn $i(self, other: Value) -> Value {
            if let Value::Float(_) = self {
                Value::Float(self.as_float().$i(&other.as_float()))
            } else if let Value::Float(_) = other {
                Value::Float(self.as_float().$i(&other.as_float()))
            } else {
                Value::Int(self.as_int().$i(&other.as_int()))
            }
        }
    }
}

macro_rules! cmp_method {
    ($i:ident) => {
        pub fn $i(self, other: Value) -> Value {
            if let Value::Float(_) = self {
                Value::Bool(self.as_float().$i(&other.as_float()))
            } else if let Value::Float(_) = other {
                Value::Bool(self.as_float().$i(&other.as_float()))
            } else {
                Value::Bool(self.as_int().$i(&other.as_int()))
            }
        }
    }
}

impl Value {

    pub fn new_array(v: Vec<Value>) -> Value {
        Value::Array(ArrayObject::from_vec(v))
    }

    arith_method!(add);
    arith_method!(sub);
    arith_method!(mul);
    arith_method!(div);
    arith_method!(rem);

    pub fn pow(self, other: Value) -> Value {
        Value::Float(self.as_float().powf(other.as_float()))
    }

    cmp_method!(lt);
    cmp_method!(gt);
    cmp_method!(le);
    cmp_method!(ge);
    cmp_method!(eq);
    cmp_method!(ne);

    pub fn and(self, other: Value) -> Value {
        Value::Bool(self.as_bool() && other.as_bool())
    }

    pub fn or(self, other: Value) -> Value {
        Value::Bool(self.as_bool() || other.as_bool())
    }

    pub fn band(self, other: Value) -> Value {
        Value::Int(self.as_int() & other.as_int())
    }

    pub fn bor(self, other: Value) -> Value {
        Value::Int(self.as_int() | other.as_int())
    }

    pub fn bxor(self, other: Value) -> Value {
        Value::Int(self.as_int() ^ other.as_int())
    }

    pub fn neg(self) -> Value {
        match self {
            Value::Int(v) => Value::Int(-v),
            Value::Float(v) => Value::Float(-v),
            _ => panic!("Negation only available for Int and Float"),
        }
    }

    pub fn not(self) -> Value {
        match self {
            Value::QuReg(mut q) => {
                q.not();
                Value::QuReg(q)
            },
            _ => Value::Bool(!self.as_bool()),
        }
    }

    pub fn bnot(self) -> Value {
        Value::Int(!self.as_int())
    }

    pub fn len(self) -> Value {
        match self {
            Value::Array(a) => Value::Int(a.len() as i64),
            Value::QuReg(q) => Value::Int(q.len() as i64),
            _ => panic!("Length operation not available for {:?}", &self),
        }
    }

    pub fn qalloc(self) -> Value {
        match self {
            Value::Int(v) => {
                assert!(v > 0);
                Value::QuReg(QuRegObject::new(v as usize))
            },
            _ => panic!("Must use an integer to allocate a quantum register!"),
        }
    }

    pub fn get(self, index: Value) -> Value {
        let idx = index.as_int() as usize;
        match self {
            Value::Int(v) => Value::Int((v & (1 << idx)) >> idx),
            Value::Array(v) => v.get(idx),
            Value::QuReg(q) => q.get(idx),
            _ => panic!("Get operation not available for {:?}", &self),
        }
    }

    pub fn put(self, index: Value, value: Value) -> Value {
        let idx = index.as_int() as usize;
        match self {
            Value::Array(mut v) => v.put(idx, value.clone()),
            _ => panic!("Put operation not available for {:?}", &self),
        }
        value
    }
    
    pub fn slice(self, index1: Value, index2: Value) -> Value {
        let idx1 = index1.as_int() as usize;
        let idx2 = index2.as_int() as usize;
        match self {
            Value::Array(v) => v.slice(idx1, idx2),
            Value::QuReg(q) => q.slice(idx1, idx2),
            _ => panic!("Slice operation not available for {:?}", &self),
        }
    }

    // TODO: Improve error message when more than one strong reference
    // is found on self or other.
    pub fn cat(self, other: Value) -> Value {
        if let Value::Array(mut v) = self {
            v.push_back(other);
            return Value::Array(v);
        } else if let Value::Array(mut v) = other {
            v.push_front(self);
            return Value::Array(v);
        }
        Value::Array(ArrayObject::from_vec(vec![self, other]))
    }

    pub fn as_int(self) -> i64 {
        match self {
            Value::Int(v) => v,
            Value::Float(v) => v as i64,
            Value::Bool(v) => v as i64, 
            _ => panic!("Invalid cast of {:?} to Int", self),
        }
    }

    pub fn as_float(self) -> f64 {
        match self {
            Value::Int(v) => v as f64,
            Value::Float(v) => v,
            _ => panic!("Invalid cast of {:?} to Float", self),
        }
    }

    pub fn as_bool(self) -> bool {
        match self {
            Value::Bool(v) => v,
            Value::Int(v) => if v != 0 { true } else { false },
            _ => panic!("Invalid cast of {:?} to Bool", self),
        }
    }

    pub fn as_func(self) -> FunctionToken {
        match self {
            Value::Func(ft) => ft,
            _ => panic!("Invalid cast of {:?} to Func", self),
        }
    }

    pub fn as_addr(self) -> usize {
        match self {
            Value::Addr(v) => v,
            _ => panic!("Invalid cast of {:?} to Addr", self),
        }
    }

    pub fn as_qureg(self) -> QuRegObject {
        match self {
            Value::QuReg(v) => v,
            _ => panic!("Invalid cast of {:?} to QuReg", self),
        }
    }

    pub fn as_string(self) -> String {
        match self {
            Value::Bool(v) => v.to_string(),
            Value::Int(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
            Value::Array(v) => v.to_string(),
            _ => panic!("String representation not available for {:?}", &self),
        }
    }
}
