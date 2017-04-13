use std::cell::RefCell;
use std::ops::{Add, Sub, Mul, Div};
use std::rc::Rc;
use std::vec::Vec;

type ArrayObject = Rc<RefCell<Vec<Value>>>;

#[derive(Debug, Clone)]
pub enum Value {
    Empty,
    Addr(usize),
    Int(i64),
    Bool(bool),
    Float(f64),
    Array(ArrayObject),
    //QReg(),
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

    pub fn get(self, index: Value) -> Value {
        match self {
            Value::Array(v) => (v.borrow())[index.as_int() as usize].clone(),
            _ => panic!("Index operation only available for Array"),
        }
    }

    pub fn put(self, index: Value, value: Value) -> Value {
        match self {
            Value::Array(v) => (v.borrow_mut())[index.as_int() as usize] = value.clone(),
            _ => panic!("Index operation only available for Array"),
        }
        value
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

    pub fn as_addr(self) -> usize {
        match self {
            Value::Addr(v) => v,
            _ => panic!("Invalid case of {:?} to Addr", self),
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
                _ => out.push(c),
            }
            escaping = false;
        } else {
            match c {
                // TODO: Verify that not too few args
                '@' => {
                    for c in args[arg].clone().as_string().chars() {
                        out.push(c);
                    }
                    arg += 1;
                },
                '\\' => escaping = true,
                _ => out.push(c),
            }
        }
    }
    let s: String = out.into_iter().collect();
    print!("{}", s);
}
