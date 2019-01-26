use std::fs;

mod cpu;
mod types;
mod x68k;

use self::cpu::{BusTrait, Cpu};

fn main_loop<BusT: BusTrait>(cpu: &mut Cpu<BusT>) {
    cpu.run();
}

fn main() {
    let res = fs::read("X68BIOSE/IPLROM.DAT");
    match res {
        Result::Ok(data) => {
            let mut bus = x68k::Bus::new(data);
            let mut cpu = Cpu::new(&mut bus);
            main_loop(&mut cpu);
        },
        Result::Err(err) => {
            panic!(err);
        }
    }
}
