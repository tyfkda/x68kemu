use std::fs;

mod cpu;
mod types;
mod x68k;

use self::cpu::{BusTrait, Cpu};

fn main_loop<BusT: BusTrait>(cpu: &mut Cpu<BusT>) {
    cpu.run();
}

fn main() {
    match fs::read("X68BIOSE/IPLROM.DAT") {
        Result::Ok(ipl) => {
            let mut bus = x68k::Bus::new(ipl);
            let mut cpu = Cpu::new(&mut bus);
            main_loop(&mut cpu);
        },
        Result::Err(err) => {
            panic!(err);
        }
    }
}
