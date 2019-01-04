pub mod cpu;
pub(crate) mod disasm;
pub(crate) mod opcode;
pub(crate) mod types;

use self::cpu::{Cpu};
use self::types::{Byte};

pub fn new_cpu(ipl: Vec<Byte>) -> Cpu {
    let mut cpu = Cpu{
        mem: vec![0; 0x10000],
        sram: vec![0; 0x4000],
        ipl: ipl,
        a: [0; 8],
        d: [0; 8],
        pc: 0,
        sr: 0,
    };
    cpu.reset();
    cpu
}
