use std::fs;

mod x68k;

use self::x68k::bus_trait::{BusTrait};

fn main_loop<BusT: BusTrait>(mut cpu: x68k::cpu::Cpu<BusT>) {
    cpu.run();
}

fn main() {
    let res = fs::read("X68BIOS/IPLROM.DAT");
    match res {
        Result::Ok(data) => {
            let bus = x68k::bus::Bus::new(data);
            let cpu = x68k::cpu::Cpu::new(bus);
            main_loop(cpu);
        },
        Result::Err(err) => {
            panic!(err);
        }
    }
}
