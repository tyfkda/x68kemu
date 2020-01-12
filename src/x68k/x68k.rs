use super::bus::{Bus};
use super::super::cpu::{Cpu};
use super::super::types::{Byte};

pub struct X68k {
    cpu: Cpu<Bus>,
}

impl X68k {
    pub fn new(ipl: Vec<Byte>) -> X68k {
        let bus = Bus::new(ipl);
        let mut cpu = Cpu::new(bus);
        cpu.reset();

        let x68k = X68k {
            cpu,
        };
        x68k
    }

    pub fn update(&mut self, cycles: usize) {
        self.cpu.run_cycles(cycles);
    }
}
