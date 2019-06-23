use std::fs;

mod cpu;
mod types;
mod x68k;

use self::cpu::{BusTrait, Cpu};

fn main_loop<BusT: BusTrait>(regs: &mut cpu::Registers, bus: &mut BusT) {
    let mut cpu = Cpu::new(regs, bus);
    cpu.run();
}

fn main() {
    match fs::read("X68BIOSE/IPLROM.DAT") {
        Result::Ok(ipl) => {
            let mut bus = x68k::Bus::new(ipl);
            let mut regs = cpu::Registers::new();
            main_loop(&mut regs, &mut bus);
        },
        Result::Err(err) => {
            panic!(err);
        }
    }
}
