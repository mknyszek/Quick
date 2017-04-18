use backend::value::Value;

use std::vec::Vec;

pub struct IRTFunction {
    pub entry: fn(&mut Vec<Value>),
    pub arity: usize,
}

irt_table! {
    fn[stack] len(1) {
        let s = stack.pop().unwrap();
        stack.push(s.len());
    }

    fn[stack] get(2) {
        let i = stack.pop().unwrap();
        let s = stack.pop().unwrap();
        stack.push(s.get(i));
    }

    fn[stack] put(3) {
        let e = stack.pop().unwrap();
        let i = stack.pop().unwrap();
        let s = stack.pop().unwrap();
        stack.push(s.put(i, e));
    }

    fn[stack] cat(2) {
        let s2 = stack.pop().unwrap();
        let s1 = stack.pop().unwrap();
        stack.push(s1.cat(s2));
    }

    fn[stack] hadamard(1) {
        let s = stack.pop().unwrap();
        match s {
            Value::QuReg(ref q) => {
                let l = q.borrow().width();
                q.borrow_mut().walsh(l);
            },
            Value::Qubit(i, ref q) => q.borrow_mut().hadamard(i),
            _ => panic!("Hadamard only available on quantum registers and bits."),
        }
        stack.push(s);
    }

    fn[stack] sigx(1) { qubit_irt_fn_t!(stack, sigma_x); }
    fn[stack] sigy(1) { qubit_irt_fn_t!(stack, sigma_y); }
    fn[stack] sigz(1) { qubit_irt_fn_t!(stack, sigma_z); }

    fn[stack] rx(2) { qubit_irt_fn_t_g!(stack, rotate_x); }
    fn[stack] ry(2) { qubit_irt_fn_t_g!(stack, rotate_y); }
    fn[stack] rz(2) { qubit_irt_fn_t_g!(stack, rotate_z); }

    fn[stack] phase(2)   { qubit_irt_fn_t_g!(stack, phase); }
    fn[stack] phaseby(2) { qubit_irt_fn_t_g!(stack, phaseby); }

    fn[stack] qft(1) {
        let s = stack.pop().unwrap();
        match s {
            Value::QuReg(ref q) => {
                let l = q.borrow().width();
                q.borrow_mut().qft(l);
            },
            _ => panic!("QFT only available on quantum registers."),
        }
        stack.push(s);
    }

    fn[stack] iqft(1) {
        let s = stack.pop().unwrap();
        match s {
            Value::QuReg(ref q) => {
                let l = q.borrow().width();
                q.borrow_mut().qft_inv(l);
            },
            _ => panic!("QFT only available on quantum registers."),
        }
        stack.push(s);
    }

    fn[stack] qftw(2) {
        let w = stack.pop().unwrap();
        let s = stack.pop().unwrap();
        match s {
            Value::QuReg(ref q) => q.borrow_mut().qft(w.as_int() as usize),
            _ => panic!("QFT only available on quantum registers."),
        }
        stack.push(s);
    }

    fn[stack] iqftw(2) {
        let w = stack.pop().unwrap();
        let s = stack.pop().unwrap();
        match s {
            Value::QuReg(ref q) => q.borrow_mut().qft_inv(w.as_int() as usize),
            _ => panic!("QFT only available on quantum registers."),
        }
        stack.push(s);
    }

    fn[stack] measure(1) {
        let s = stack.pop().unwrap();
        let value = match s {
            Value::QuReg(ref q) => {
                let width = q.borrow().width();
                Value::Int(q.borrow_mut().measure_partial(0..width) as i64)
            },
            Value::Qubit(i, ref q) => {
                Value::Int(q.borrow_mut().measure_bit_preserve(i) as i64)
            },
            _ => panic!("Measurement only available for quantum registers and bits."),
        };
        stack.push(value);
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
