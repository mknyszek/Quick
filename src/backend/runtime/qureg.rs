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

use libquantum::QuReg;

#[derive(Debug, Clone)]
pub struct QuRegObject {
    start: usize,
    end: usize,
    scratch: bool,
    qureg: Rc<RefCell<QuReg>>, 
}

impl QuRegObject {
    pub fn new(s: usize, init: i64) -> QuRegObject {
        QuRegObject {
            start: 0,
            end: s,
            scratch: false,
            qureg: Rc::new(RefCell::new(QuReg::new(s, init as u64))),
        }
    }

    fn raw_start(&self) -> usize {
        if self.scratch {
            self.scratch() - self.start - 1
        } else {
            self.start + self.scratch()
        }
    }

    fn raw_end(&self) -> usize {
        if self.scratch {
            self.scratch() - self.end + 1
        } else {
            self.end + self.scratch()
        }
    }

    fn qubit(&self) -> bool {
        self.len() == 1
    }

    fn scratch(&self) -> usize {
        self.qureg.borrow().scratch()
    }

    fn add_scratch(&mut self) -> QuRegObject {
        assert!(self.qureg.borrow().width() < 64);
        self.qureg.borrow_mut().add_scratch(1);
        QuRegObject {
            start: self.scratch() - 1,
            end: self.scratch(),
            scratch: true,
            qureg: self.qureg.clone(),
        }
    }

    fn remove_scratch(self) {
        if self.scratch {
            // Scratch reference must be a bit
            assert!(self.qubit());
            if self.end != self.scratch() {
                panic!("Scratch qubit {} deleted out of order!", self.start);
            }
            let bit = self.raw_start();
            if self.qureg.borrow_mut().measure_bit(bit) {
                panic!("Scratch qubit {} not properly cleared!", self.start);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn get(&self, idx: usize) -> QuRegObject {
        if idx > self.end - self.start {
            panic!("Invalid index '{}' into QuReg.", idx);
        }
        self.slice(idx, idx+1)
    }

    pub fn slice(&self, lb: usize, ub: usize) -> QuRegObject {
        if lb >= ub || ub > self.end - self.start {
            panic!("Invalid slice indicies '{}:{}' into QuReg.", lb, ub);
        }
        QuRegObject {
            start: self.start + lb,
            end: self.start + ub,
            scratch: self.scratch,
            qureg: self.qureg.clone(),
        }
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

    fn to_vec(&self) -> Vec<QuRegObject> {
        let mut v = Vec::with_capacity(self.len() + 2);
        for i in 0..self.len() {
            v.push(self.get(i));
        }
        v
    }

    fn overlaps(&self, other: &QuRegObject) -> bool {
        self.raw_start() < other.raw_end() && other.raw_start() < self.raw_end()
    }

    fn cnot_half(target: &mut QuRegObject,
                 dummy: &mut [QuRegObject],
                 control: &mut [QuRegObject]) {

        debug_assert!(control.len() > 0);
        debug_assert!(control.len() <= 1 + ((dummy.len() + control.len() + 1) / 2));
        if control.len() == 1 {
            target.cnot(&mut control[0]);
            return;
        } else if control.len() == 2 {
            let (l, u) = control.split_at_mut(1);
            target.toffoli(&mut l[0], &mut u[0]);
            return;
        }
        let control_len = control.len();
        // Perform O(n) (C^m)NOT algorithm, to avoid allocating bits
        // See https://arxiv.org/abs/quant-ph/9503016
        // In this case, m = control_len
        target.toffoli(&mut dummy[0], &mut control[0]);
        for i in 0..control_len-3 {
            let (l, u) = dummy.split_at_mut(i+1);
            l[i].toffoli(&mut u[0], &mut control[i]);
        }
        {
            let (l, u) = control.split_at_mut(control_len-1);
            dummy[control_len-3].toffoli(&mut u[0], &mut l[control_len-2]);
        }
        for i in (0..control_len-3).rev() {
            let (l, u) = dummy.split_at_mut(i+1);
            l[i].toffoli(&mut u[0], &mut control[i]);
        }
        target.toffoli(&mut dummy[0], &mut control[0]);
        // Clean up messed up dummy qubits
        for i in 0..control_len-3 {
            let (l, u) = dummy.split_at_mut(i+1);
            l[i].toffoli(&mut u[0], &mut control[i]);
        }
        {
            let (l, u) = control.split_at_mut(control_len-1);
            dummy[control_len-3].toffoli(&mut u[0], &mut l[control_len-2]);
        }
        for i in (0..control_len-3).rev() {
            let (l, u) = dummy.split_at_mut(i+1);
            l[i].toffoli(&mut u[0], &mut control[i]);
        }
    }

    pub fn cnot(&mut self, control: &mut QuRegObject) {
        assert!(Rc::ptr_eq(&self.qureg, &control.qureg));
        assert!(!self.overlaps(control));
        if control.qubit() {
            let startc = control.raw_start();
            let start = self.raw_start();
            let end = self.raw_end();
            let mut qm = self.qureg.borrow_mut();
            for i in start..end {
                qm.cnot(startc, i);
            }
            return;
        } else if control.len() == 2 {
            let startc = control.raw_start();
            let start = self.raw_start();
            let end = self.raw_end();
            let mut qm = self.qureg.borrow_mut();
            for i in start..end {
                qm.toffoli(startc, startc+1, i);
            }
            return;
        } else if control.len() < 1 {
            panic!("Found zero-length quantum register!");
        }
        let mut work = self.add_scratch();
        let k = control.len();
        let m = (2 + k) / 2;
        let mut half1 = control.slice(0, k-m).to_vec();
        let mut half2 = control.slice(k-m, k).to_vec();
        for i in 0..self.len() {
            let mut bit = self.get(i);
            half1.push(bit);
            QuRegObject::cnot_half(&mut work, &mut half1[..], &mut half2[..]);
            bit = half1.pop().unwrap();
            half1.push(work);
            QuRegObject::cnot_half(&mut bit, &mut half2[..], &mut half1[..]);
            work = half1.pop().unwrap();
            half1.push(bit);
            QuRegObject::cnot_half(&mut work, &mut half1[..], &mut half2[..]);
            bit = half1.pop().unwrap();
            half1.push(work);
            QuRegObject::cnot_half(&mut bit, &mut half2[..], &mut half1[..]);
            work = half1.pop().unwrap();
        }
        work.remove_scratch();
    }

    pub fn toffoli(&mut self, control1: &mut QuRegObject, control2: &mut QuRegObject) {
        assert!(Rc::ptr_eq(&self.qureg, &control1.qureg));
        assert!(Rc::ptr_eq(&self.qureg, &control2.qureg));
        assert!(self.qubit());
        assert!(control1.qubit());
        assert!(!self.overlaps(control1));
        assert!(control2.qubit());
        assert!(!self.overlaps(control2));
        let start = self.raw_start();
        let start1 = control1.raw_start();
        let start2 = control2.raw_start();
        self.qureg.borrow_mut().toffoli(start1, start2, start);
    }

    pub fn cphase(&mut self, control: &mut QuRegObject) {
        assert!(Rc::ptr_eq(&self.qureg, &control.qureg));
        assert!(self.qubit());
        assert!(control.qubit());
        assert!(!self.overlaps(control));
        let start = self.raw_start();
        let startc = control.raw_start();
        self.qureg.borrow_mut().cond_phase(startc, start);
    }

    pub fn cphaseby(&mut self, control: &mut QuRegObject, gamma: f64) {
        assert!(Rc::ptr_eq(&self.qureg, &control.qureg));
        assert!(self.qubit());
        assert!(control.qubit());
        assert!(!self.overlaps(control));
        let start = self.raw_start();
        let startc = control.raw_start();
        self.qureg.borrow_mut().cond_phaseby(startc, start, gamma as f32);
    }

    pub fn cflip(&mut self, control: &mut QuRegObject) {
        self.hadamard();
        self.cnot(control);
        self.hadamard();
    }

    pub fn swap(&mut self, control: &mut QuRegObject) {
        assert!(self.qubit());
        assert!(control.qubit());
        self.cnot(control);
        control.cnot(self);
        self.cnot(control);
    }

    pub fn all(&mut self) -> QuRegObject {
        let mut scratch = self.add_scratch();
        scratch.cnot(self);
        scratch
    }

    pub fn iall(mut self, orig: &mut QuRegObject) {
        self.cnot(orig);
        self.remove_scratch();
    }

    pub fn any(&mut self) -> QuRegObject {
        let mut scratch = self.add_scratch();
        self.sigma_x();
        scratch.cnot(self);
        self.sigma_x();
        scratch.sigma_x();
        scratch
    }

    pub fn iany(mut self, orig: &mut QuRegObject) {
        self.sigma_x();
        orig.sigma_x();
        self.cnot(orig);
        orig.sigma_x();
        self.remove_scratch();
    }

    pub fn not(&mut self) -> QuRegObject {
        let mut scratch = self.add_scratch();
        scratch.cnot(self);
        scratch.sigma_x();
        scratch
    }

    pub fn inot(mut self, orig: &mut QuRegObject) {
        self.sigma_x();
        self.cnot(orig);
        self.remove_scratch();
    }

    pub fn and(&mut self, other: &mut QuRegObject) -> QuRegObject {
        let mut scratch = self.add_scratch();
        scratch.toffoli(self, other);
        scratch
    }

    pub fn iand(mut self, control1: &mut QuRegObject, control2: &mut QuRegObject) {
        self.toffoli(control1, control2);
        self.remove_scratch();
    } 

    pub fn or(&mut self, other: &mut QuRegObject) -> QuRegObject {
        let mut scratch = self.add_scratch();
        scratch.cnot(self);
        scratch.cnot(other);
        scratch.toffoli(self, other);
        scratch
    }

    pub fn ior(mut self, control1: &mut QuRegObject, control2: &mut QuRegObject) {
        self.toffoli(control1, control2);
        self.cnot(control2);
        self.cnot(control1);
        self.remove_scratch();
    }

    pub fn measure(&mut self) -> i64 {
        let start = self.raw_start();
        let end = self.raw_end();
        self.qureg.borrow_mut().measure_partial(start..end) as i64
    }

    pub fn to_string(&self) -> String {
        self.qureg.borrow().to_string().unwrap()
    }
}

