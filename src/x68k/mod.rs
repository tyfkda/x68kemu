pub(crate) mod bus;
pub mod cpu;
pub(crate) mod disasm;
pub(crate) mod opcode;
pub(crate) mod types;

use self::bus::{Bus};
use self::cpu::{Cpu};
use self::types::{Byte};

pub fn new_cpu(ipl: Vec<Byte>) -> Cpu {
    let bus = Bus {
        mem: vec![0; 0x10000],
        sram: vec![0; 0x4000],
        ipl: ipl,
    };
    let mut cpu = Cpu {
        bus: bus,
        a: [0; 8],
        d: [0; 8],
        pc: 0,
        sr: 0,
    };
    cpu.reset();
    cpu
}
