use std::fs;

mod cpu;
mod types;
mod x68k;

use self::cpu::{BusTrait, Cpu};

fn main_loop<BusT: BusTrait>(mut cpu: Cpu<BusT>) {
    cpu.run();
}

fn main() {
    let res = fs::read("X68BIOS/IPLROM.DAT");
    match res {
        Result::Ok(data) => {
            let bus = x68k::Bus::new(data);
            let cpu = Cpu::new(bus);
            main_loop(cpu);
        },
        Result::Err(err) => {
            panic!(err);
        }
    }
}
