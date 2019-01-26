use super::bus::Bus;
use super::fdc::Fdc;
use super::vram::Vram;
use super::super::cpu::Cpu;
use super::super::types::Byte;

pub struct X68k {
    cpu: Cpu<Bus>,
}

impl X68k {
    pub fn new(ipl: Vec<Byte>) -> Self {
        let vram = Vram::new();
        let fdc = Fdc::new();
        let bus = Bus::new(ipl, vram, fdc);
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
