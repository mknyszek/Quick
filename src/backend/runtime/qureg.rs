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

use backend::runtime::value::Value;

use libquantum::QuReg;

#[derive(Debug, Clone)]
pub struct QuRegObject {
    start: usize,
    end: usize,
    scratch: bool,
    qureg: Rc<RefCell<QuReg>>, 
}

impl QuRegObject {
    pub fn new(s: usize) -> QuRegObject {
        QuRegObject {
            start: 0,
            end: s,
            scratch: false,
            qureg: Rc::new(RefCell::new(QuReg::new(s, 0))),
        }
    }

    fn raw_start(&self) -> usize {
        if self.scratch {
            self.start
        } else {
            self.start + self.scratch()
        }
    }

    fn raw_end(&self) -> usize {
        if self.scratch {
            self.end
        } else {
            self.end + self.scratch()
        }
    }

    fn scratch(&self) -> usize {
        self.qureg.borrow().scratch()
    }

    fn add_scratch(&mut self) -> QuRegObject {
        self.qureg.borrow_mut().add_scratch(1);
        QuRegObject {
            start: self.scratch() - 1,
            end: self.scratch(),
            scratch: true,
            qureg: self.qureg.clone(),
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn get(&self, idx: usize) -> Value {
        if idx > self.end - self.start {
            panic!("Invalid index '{}' into QuReg.", idx);
        }
        self.slice(idx, idx+1)
    }

    pub fn slice(&self, lb: usize, ub: usize) -> Value {
        if self.scratch {
            panic!("Invalid slice of scratch qubit.");
        }
        if lb >= ub || ub > self.end - self.start {
            panic!("Invalid slice indicies '{}:{}' into QuReg.", lb, ub);
        }
        Value::QuReg(QuRegObject {
            start: self.start + lb,
            end: self.start + ub,
            scratch: false,
            qureg: self.qureg.clone(),
        })
    } 

    qureg_fn_t!(hadamard);
    qureg_fn_t!(sigma_x);
    qureg_fn_t!(sigma_y);
    qureg_fn_t!(sigma_z);

    qureg_fn_t_g!(rotate_x);
    qureg_fn_t_g!(rotate_y);
    qureg_fn_t_g!(rotate_z);
    qureg_fn_t_g!(phase);
    qureg_fn_t_g!(phaseby);

    pub fn cnot(&mut self, control: &mut QuRegObject) {
        //assert!(Rc::ptr_eq(&self.qureg, &control.qureg));
        let start = self.raw_start();
        self.qureg.borrow_mut().cnot(control.raw_start(), start);
    }

    pub fn toffoli(&mut self, control1: &mut QuRegObject, control2: &mut QuRegObject) {
        //assert!(Rc::ptr_eq(&self.qureg, &control.qureg));
        let start = self.raw_start();
        self.qureg.borrow_mut().toffoli(control1.raw_start(), control2.raw_start(), start);
    }

    pub fn cphase(&mut self, control: &mut QuRegObject) {
        //assert!(Rc::ptr_eq(&self.qureg, &control.qureg));
        let start = self.raw_start();
        self.qureg.borrow_mut().cond_phase(control.raw_start(), start);
    }

    pub fn cphaseby(&mut self, control: &mut QuRegObject, gamma: f64) {
        //assert!(Rc::ptr_eq(&self.qureg, &control.qureg));
        let start = self.raw_start();
        self.qureg.borrow_mut().cond_phaseby(control.raw_start(), start, gamma as f32);
    }

    pub fn not(&mut self) {
        self.sigma_x();
    }

    pub fn and(&mut self, other: &mut QuRegObject) -> QuRegObject {
        let mut scratch = self.add_scratch();
        scratch.toffoli(self, other);
        scratch
    }

    pub fn or(&mut self, other: &mut QuRegObject) -> QuRegObject {
        let mut scratch = self.add_scratch();
        // Use De Morgan's Law to implement reversible OR gate with
        // Toffoli, which closely resembles "AND"
        self.not();
        other.not();
        scratch.toffoli(self, other);
        self.not(); // Reverse NOT on self
        other.not(); // Reverse NOT on other 
        scratch.not();
        scratch
    }

    pub fn measure(&mut self) -> i64 {
        let start = self.raw_start();
        let end = self.raw_end();
        self.qureg.borrow_mut().measure_partial(start..end) as i64
    }
}

impl Drop for QuRegObject {
    fn drop(&mut self) {
        if self.scratch {
            // Scratch reference must be a bit
            assert!(self.end - self.start == 1);
            if self.end != self.scratch() {
                panic!("Scratch qubit {} deleted out of order!", self.start);
            }
            if self.qureg.borrow_mut().measure_bit(self.start) {
                panic!("Scratch qubit {} not properly cleared!", self.start);
            }
        }
    }
}
