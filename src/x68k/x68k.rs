use std::cell::RefCell;
use std::rc::Rc;

use super::bus::{Bus};
use super::vram::{Vram};
use super::super::cpu::{Cpu};
use super::super::types::{Byte};

pub struct X68k {
    cpu: Cpu<Bus>,
    #[allow(dead_code)]
    vram: Rc<RefCell<Vram>>,
}

impl X68k {
    pub fn new(ipl: Vec<Byte>) -> X68k {
        let vram = Rc::new(RefCell::new(Vram::new()));
        let bus = Bus::new(ipl, vram.clone());
        let mut cpu = Cpu::new(bus);
        cpu.reset();

        X68k {
            cpu,
            vram,
        }
    }

    pub fn update(&mut self, cycles: usize) {
        self.cpu.run_cycles(cycles);
    }
}
