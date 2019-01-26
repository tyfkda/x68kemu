use std::cell::RefCell;
use std::rc::Rc;

use super::bus::{Bus};
use super::fdc::{Fdc};
use super::super::cpu::{Cpu};
use super::super::types::{Byte};

pub struct X68k {
    cpu: Cpu<Bus>,
    //fdc: Rc<RefCell<Fdc>>,
}

impl X68k {
    pub fn new(ipl: Vec<Byte>) -> X68k {
        let fdc = Rc::new(RefCell::new(Fdc::new()));
        let bus = Bus::new(ipl, fdc.clone());
        let mut cpu = Cpu::new(bus);
        cpu.reset();

        let x68k = X68k {
            cpu,
            //fdc,
        };
        x68k
    }

    pub fn update(&mut self, cycles: usize) {
        self.cpu.run_cycles(cycles);
    }
}
