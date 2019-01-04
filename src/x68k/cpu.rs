use super::disasm::{disasm};
use super::opcode::{Opcode, INST};
use super::types::{Byte, Word, Long, Adr};

const DREG: usize = 0;
const AREG: usize = 8;
const SP: usize = 7 + AREG;  // Stack pointer = A7 register.

pub struct Cpu {
    pub(crate) mem: Vec<Byte>,
    pub(crate) ipl: Vec<Byte>,
    pub(crate) regs: Vec<Long>,
    pub(crate) pc: Adr,
    pub(crate) sr: Word,
}

impl Cpu {
    pub fn reset(&mut self) {
        self.sr = 0;
        self.regs[SP] = self.read32(0xff0000);
        self.pc = self.read32(0xff0004);
    }

    pub fn run(&mut self) {
        loop {
            let (sz, mnemonic) = disasm(&self, self.pc);
            println!("{:06x}: {}  {}", self.pc, self.dump_mem(self.pc, sz, 5), mnemonic);
            self.step();
        }
    }

    fn step(&mut self) {
        let startadr = self.pc;
        let op = self.read16(self.pc);
        self.pc += 2;
        let inst = &INST[op as usize];

        match inst.op {
            Opcode::MoveLong => {
                let n = ((op >> 9) & 7) as usize;
                let m = (op & 7) as usize;
                let dt = ((op >> 6) & 7) as usize;
                let src = self.read_source32(((op >> 3) & 7) as usize, m);
                self.write_destination32(dt, n, src);
            },
            Opcode::MoveWord => {
                let n = ((op >> 9) & 7) as usize;
                let m = (op & 7) as usize;
                let dt = ((op >> 6) & 7) as usize;
                let src = self.read_source16(((op >> 3) & 7) as usize, m);
                self.write_destination16(dt, n, src);
            },
            Opcode::MoveToSrIm => {
                self.sr = self.read16(self.pc);
                self.pc += 2;
            },
            Opcode::LeaDirect => {
                let di = ((op >> 9) & 7) as usize;
                let value = self.read32(self.pc);
                self.pc += 4;
                self.regs[di + AREG] = value;
            },
            Opcode::Reset => {
                // TODO: Implement.
            },
            Opcode::AddLong => {
                let di = ((op >> 9) & 7) as usize;
                let si = (op & 7) as usize;
                self.regs[di + DREG] = self.regs[di + DREG].wrapping_add(self.regs[si + DREG]);
            },
            Opcode::SubaLong => {
                let di = ((op >> 9) & 7) as usize;
                let si = (op & 7) as usize;
                self.regs[di + AREG] -= self.regs[si + AREG];
            },
            Opcode::Dbra => {
                let si = (op & 7) as usize;
                let ofs = self.read16(self.pc) as i16;
                self.pc += 2;

                let l = self.regs[si + DREG];
                let w = (l as u16).wrapping_sub(1);
                self.regs[si + DREG] = (l & 0xffff0000) | (w as u32);
                if w != 0xffff {
                    self.pc = (self.pc - 2).wrapping_add((ofs as i32) as u32);
                }
            },
            Opcode::Bsr => {
                let mut ofs = ((op & 0x00ff) as i8) as i16;
                if ofs == 0 {
                    ofs = self.read16(self.pc) as i16;
                    self.pc += 2;
                }
                self.push32(self.pc);
                self.pc = ((startadr + 2) as i32 + ofs as i32) as u32;
            },
            _ => {
                eprintln!("{:08x}: {:04x}  ; Unknown opcode", startadr, op);
                panic!("Not implemented");
            },
        }
    }

    fn push32(&mut self, value: Long) {
        let sp = self.regs[SP] - 4;
        self.regs[SP] = sp;
        self.write32(sp, value);
    }

    fn read_source16(&mut self, src: usize, m: usize) -> u16 {
        match src {
            7 => {  // Misc.
                match m {
                    4 => {  // move.w #$XXXX, xx
                        let value = self.read16(self.pc);
                        self.pc += 2;
                        value
                    },
                    _ => {
                        panic!("Not implemented, m={}", m);
                    },
                }
            },
            _ => {
                panic!("Not implemented, src={}", src);
            },
        }
    }

    fn read_source32(&mut self, src: usize, m: usize) -> u32 {
        match src {
            0 => {  // move.l Dm, xx
                self.regs[m + DREG]
            },
            7 => {  // Misc.
                match m {
                    4 => {  // move.l #$XXXX, xx
                        let value = self.read32(self.pc);
                        self.pc += 4;
                        value
                    },
                    _ => {
                        panic!("Not implemented, m={}", m);
                    },
                }
            },
            _ => {
                panic!("Not implemented, src={}", src);
            },
        }
    }

    fn write_destination16(&mut self, dst: usize, n: usize, value: Word) {
        match dst {
            0 => {
                self.regs[n + DREG] = (self.regs[n + DREG] & 0xffff0000) | (value as u32);
            },
            _ => {
                panic!("Not implemented, dst={}", dst);
            },
        }
    }

    fn write_destination32(&mut self, dst: usize, n: usize, value: Long) {
        match dst {
            0 => {
                self.regs[n + DREG] = value;
            },
            3 => {
                let adr = self.regs[n + AREG];
                self.write32(adr, value);
                self.regs[n + AREG] = adr + 4;
            },
            7 => {
                match n {
                    1 => {
                        let d = self.read32(self.pc);
                        self.pc += 4;
                        self.write32(d, value);
                    },
                    _ => {
                        panic!("Not implemented, n={}", n);
                    },
                }
            },
            _ => {
                panic!("Not implemented, dst={}", dst);
            },
        }
    }

    fn read8(&self, adr: Adr) -> Byte {
        if 0xfe0000 <= adr && adr <= 0xffffff {
            self.ipl[(adr - 0xfe0000) as usize]
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }

    pub(crate) fn read16(&self, adr: Adr) -> Word {
        let d0 = self.read8(adr) as Word;
        let d1 = self.read8(adr + 1) as Word;
        (d0 << 8) | d1
    }

    pub(crate) fn read32(&self, adr: Adr) -> Long {
        let d0 = self.read8(adr) as Long;
        let d1 = self.read8(adr + 1) as Long;
        let d2 = self.read8(adr + 2) as Long;
        let d3 = self.read8(adr + 3) as Long;
        (d0 << 24) | (d1 << 16) | (d2 << 8) | d3
    }

    fn write8(&mut self, adr: Adr, value: Byte) {
        if /*0x000000 <= adr &&*/ adr <= 0xffff {
            self.mem[adr as usize] = value;
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }

    fn write32(&mut self, adr: Adr, value: Long) {
        self.write8(adr,     (value >> 24) as Byte);
        self.write8(adr + 1, (value >> 16) as Byte);
        self.write8(adr + 2, (value >>  8) as Byte);
        self.write8(adr + 3,  value        as Byte);
    }

    fn dump_mem(&self, adr: Adr, sz: usize, max: usize) -> String {
        let arr = (0..max).map(|i| {
            if i * 2 < sz {
                format!("{:04x}", self.read16(adr + (i as u32) * 2))
            } else {
                String::from("    ")
            }
        });
        arr.collect::<Vec<String>>().join(" ")
    }
}
