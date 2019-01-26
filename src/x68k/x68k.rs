use super::bus::{Bus};
use super::super::cpu;
use super::super::cpu::{Cpu};
use super::super::types::{Byte};

pub struct X68k {
    bus: Bus,
    cpu_regs: cpu::Registers,
}

impl X68k {
    pub fn new(ipl: Vec<Byte>) -> X68k {
        let bus = Bus::new(ipl);
        let cpu_regs = cpu::Registers::new();

        let x68k = X68k {
            bus,
            cpu_regs,
        };
        x68k
    }

    pub fn main_loop(&mut self) {
        let mut cpu = Cpu::new(&mut self.cpu_regs, &mut self.bus);
        cpu.run();
    }
}
