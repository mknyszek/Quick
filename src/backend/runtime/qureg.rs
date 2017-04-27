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
    qureg: Rc<RefCell<QuReg>>, 
}

impl QuRegObject {
    pub fn new(s: usize) -> QuRegObject {
        QuRegObject {
            start: 0,
            end: s,
            qureg: Rc::new(RefCell::new(QuReg::new(s, 0))),
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
        if lb >= ub || ub > self.end - self.start {
            panic!("Invalid slice indicies '{}:{}' into Array.", lb, ub);
        }
        Value::QuReg(QuRegObject {
            start: self.start + lb,
            end: self.start + ub,
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
        let mut qm = self.qureg.borrow_mut();
        for i in control.start..control.end {
            for j in self.start..self.end {
                assert!(i != j);
                qm.cnot(i, j);
            }
        }
    }

    pub fn cphase(&mut self, control: &mut QuRegObject) {
        //assert!(Rc::ptr_eq(&self.qureg, &control.qureg));
        let mut qm = self.qureg.borrow_mut();
        for i in control.start..control.end {
            for j in self.start..self.end {
                assert!(i != j);
                qm.cond_phase(i, j);
            }
        }
    }

    pub fn cphaseby(&mut self, control: &mut QuRegObject, gamma: f64) {
        //assert!(Rc::ptr_eq(&self.qureg, &control.qureg));
        let mut qm = self.qureg.borrow_mut();
        for i in control.start..control.end {
            for j in self.start..self.end {
                assert!(i != j);
                qm.cond_phaseby(i, j, gamma as f32);
            }
        }
    }

    pub fn measure(&mut self) -> i64 {
        self.qureg.borrow_mut().measure_partial(self.start..self.end) as i64
    }
}
