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

use backend::bytecode::FunctionToken;

use std::cell::RefCell;
use std::ops::{Add, Sub, Mul, Div, Rem};
use std::rc::Rc;
use std::vec::Vec;

use libquantum::QuReg;

type ArrayObject = Rc<RefCell<Vec<Value>>>;
type QuRegObject = Rc<RefCell<QuReg>>;

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
    Qubit(usize, QuRegObject),
}

enum CatOperation {
    MergeArrays,
    PushBack,
    PushFront,
    NewArray,
    Tensor
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
        Value::Bool(!self.as_bool())
    }

    pub fn bnot(self) -> Value {
        Value::Int(!self.as_int())
    }

    pub fn len(self) -> Value {
        match self {
            Value::Array(a) => Value::Int(a.borrow().len() as i64),
            Value::QuReg(q) => Value::Int(q.borrow().width() as i64),
            _ => panic!("Length operation only available for Array"),
        }
    }

    pub fn qalloc(self) -> Value {
        match self {
            Value::Int(v) => {
                assert!(v > 0);
                Value::QuReg(Rc::new(RefCell::new(QuReg::new(v as usize, 0))))
            },
            _ => panic!("Must use an integer to allocate a quantum register!"),
        }
    }

    pub fn get(self, index: Value) -> Value {
        let idx = index.as_int() as usize;
        match self {
            Value::Int(v) => Value::Int((v & (1 << idx)) >> idx),
            Value::Array(v) => (v.borrow())[idx].clone(),
            Value::QuReg(q) => {
                assert!(idx < q.borrow().width());
                Value::Qubit(idx, q)
            },
            _ => panic!("Index operation only available for Array, QuReg, and Int"),
        }
    }

    pub fn put(self, index: Value, value: Value) -> Value {
        match self {
            Value::Array(v) => (v.borrow_mut())[index.as_int() as usize] = value.clone(),
            _ => panic!("Index operation only available for Array"),
        }
        value
    }

    pub fn cat(self, other: Value) -> Value {
        let logic = match self {
            Value::Array(_) => match other {
                Value::Array(_) => CatOperation::MergeArrays,
                Value::Int(_) => CatOperation::PushBack,
                Value::Float(_) => CatOperation::PushBack,
                Value::Bool(_) => CatOperation::PushBack,
                Value::QuReg(_) => CatOperation::PushBack,
                _ => panic!("Cat operation applied to non-user type"),
            },
            Value::Int(_) 
            | Value::Float(_) 
            | Value::Bool(_) 
            | Value::Func(_) => match other {
                Value::Array(_) => CatOperation::PushFront,
                Value::Addr(_) => panic!("Shouldn't operate on Addr"),
                Value::Null => panic!("Shouldn't operate on Null"),
                _ => CatOperation::NewArray,
            },
            Value::QuReg(_) => match other {
                Value::QuReg(_) => CatOperation::Tensor,
                Value::Array(_) => CatOperation::PushFront,
                Value::Addr(_) => panic!("Shouldn't operate on Addr"),
                Value::Null => panic!("Shouldn't operate on Null"),
                _ => CatOperation::NewArray,
            },
            _ => panic!("Cat operation applied to non-user type"),
        };
        match logic {
            CatOperation::MergeArrays => {
                if let Value::Array(ref a1) = self {
                    let mut a1_inner = a1.borrow_mut();
                    if let Value::Array(ref a2) = other {
                        for v in a2.borrow().iter() {
                            a1_inner.push(v.clone());
                        }
                    } else { unreachable!(); }
                } else { unreachable!(); }
                self
            },
            CatOperation::PushBack => {
                match self {
                    Value::Array(ref a) => a.borrow_mut().push(other),
                    _ => unreachable!(),
                }
                self
            },
            CatOperation::PushFront => {
                match other {
                    Value::Array(ref a) => a.borrow_mut().push(self),
                    _ => unreachable!(),
                }
                other
            },
            CatOperation::NewArray => {
                Value::Array(Rc::new(RefCell::new(vec![self, other])))
            },
            CatOperation::Tensor => {
                // TODO: Improve error message when more than one strong reference
                // is found on self or other.
                let q = Rc::try_unwrap(self.as_qureg()).unwrap().into_inner();
                let o = Rc::try_unwrap(other.as_qureg()).unwrap().into_inner();
                Value::QuReg(Rc::new(RefCell::new(q.tensor(o))))
            },
        }
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
            Value::Array(v) => {
                let mut out = String::new();
                out.push('[');
                out.push(' ');
                for i in v.borrow().iter() {
                    out.push_str(&(i.clone().as_string())[..]);
                    out.push(' ');
                }
                out.push(']');
                out
            },
            _ => panic!("String representation not available for other types"),
        }
    }
}
