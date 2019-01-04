use std::fs;

mod x68k;

fn main_loop(mut cpu: x68k::cpu::Cpu) {
    cpu.run();
}

fn main() {
    let res = fs::read("X68BIOS/IPLROM.DAT");
    match res {
        Result::Ok(data) => {
            let cpu = x68k::new_cpu(data);
            main_loop(cpu);
        },
        Result::Err(err) => {
            panic!(err);
        }
    }
}
