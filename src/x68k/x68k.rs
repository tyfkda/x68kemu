use super::bus::Bus;
use super::super::cpu::Cpu;
use super::super::types::Byte;

pub struct X68k {
    cpu: Cpu<Bus>,
}

impl X68k {
    pub fn new(ipl: Vec<Byte>) -> Self {
        let bus = Bus::new(ipl);
        let mut cpu = Cpu::new(bus);
        cpu.reset();

        Self {
            cpu,
        }
    }

    pub fn update(&mut self, cycles: usize) {
        self.cpu.run_cycles(cycles);
    }
}
