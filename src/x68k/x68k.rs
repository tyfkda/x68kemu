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
        let mut bus = Bus::new(ipl);
        let mut cpu_regs = cpu::Registers::new();

        let mut cpu = Cpu::new(&mut cpu_regs, &mut bus);
        cpu.reset();

        let x68k = X68k {
            bus,
            cpu_regs,
        };
        x68k
    }

    pub fn update(&mut self, cycles: usize) {
        let mut cpu = Cpu::new(&mut self.cpu_regs, &mut self.bus);
        cpu.run_cycles(cycles);
    }
}
