use super::bus::{Bus};
use super::disasm::{disasm};
use super::opcode::{Opcode, INST};
use super::types::{Byte, Word, Long, Adr};

const SP: usize = 7;  // Stack pointer = A7 register.

const FLAG_C: Word = 1 << 0;
const FLAG_V: Word = 1 << 1;
const FLAG_Z: Word = 1 << 2;
const FLAG_N: Word = 1 << 3;

const TRAP_VECTOR_START: Adr = 0x0080;

pub struct Cpu {
    bus: Bus,
    a: [Adr; 8],  // Address registers
    d: [Long; 8],  // Data registers
    pc: Adr,
    sr: Word,
}

impl Cpu {
    pub fn new(bus: Bus) -> Cpu {
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

    pub fn reset(&mut self) {
        self.sr = 0;
        self.a[SP] = self.read32(0xff0000);
        self.pc = self.read32(0xff0004);
    }

    pub fn run(&mut self) {
        loop {
            let (sz, mnemonic) = disasm(&self.bus, self.pc);
            println!("{:06x}: {}  {}", self.pc, self.bus.dump_mem(self.pc, sz, 5), mnemonic);
            self.step();
        }
    }

    fn step(&mut self) {
        let startadr = self.pc;
        let op = self.read16(self.pc);
        self.pc += 2;
        let inst = &INST[op as usize];

        match inst.op {
            Opcode::MoveByte => {
                let n = ((op >> 9) & 7) as usize;
                let m = (op & 7) as usize;
                let dt = ((op >> 6) & 7) as usize;
                let src = self.read_source8(((op >> 3) & 7) as usize, m);
                self.write_destination8(dt, n, src);
            },
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
            Opcode::Moveq => {
                let di = (op >> 9) & 7;
                let v = op & 0xff;
                let val = if v < 0x80 { v as i16 } else { -256 + v as i16 };
                self.d[di as usize] = (val as i32) as u32;
            },
            Opcode::MovemFrom => {
                let di = (op & 7) as usize;
                let bits = self.read16(self.pc);
                self.pc += 2;
                let mut p = self.a[di];
                for i in 0..8 {
                    if (bits & (0x0001 << i)) != 0 {
                        p -= 4;
                        self.write32(p, self.a[7 - i]);
                    }
                }
                for i in 0..8 {
                    if (bits & (0x0100 << i)) != 0 {
                        p -= 4;
                        self.write32(p, self.d[7 - i]);
                    }
                }
                self.a[di] = p;
            },
            Opcode::MoveToSrIm => {
                self.sr = self.read16(self.pc);
                self.pc += 2;
            },
            Opcode::LeaDirect => {
                let di = ((op >> 9) & 7) as usize;
                let value = self.read32(self.pc);
                self.pc += 4;
                self.a[di] = value;
            },
            Opcode::Clr => {
                let dt = ((op >> 3) & 7) as usize;
                let n = (op & 7) as usize;
                match op & 0xffc0 {
                    0x4200 => {  // byte
                        self.write_destination8(dt, n, 0);
                    },
                    0x4240 => {  // word
                        self.write_destination16(dt, n, 0);
                    },
                    0x4280 => {  // long
                        self.write_destination32(dt, n, 0);
                    },
                    _ => {
                        panic!("Must not happen");
                    },
                }
            },
            Opcode::CmpmByte => {
                let si = (op & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let v1 = self.read8(self.a[di]);
                let v2 = self.read8(self.a[si]);
                self.a[si] += 1;
                self.a[di] += 1;
                // TODO: Check flag is true.
                let mut c = 0;
                if v1 < v2 {
                    c |= FLAG_C;
                }
                if v1 == v2 {
                    c |= FLAG_Z;
                }
                if ((v1.wrapping_sub(v2)) & 0x80) != 0 {
                    c |= FLAG_N;
                }
                self.sr = (self.sr & 0xff00) | c;
            },
            Opcode::Reset => {
                // TODO: Implement.
            },
            Opcode::AddLong => {
                let di = ((op >> 9) & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let si = (op & 7) as usize;
                let src = self.read_source32(st, si);
                self.d[di] = self.d[di].wrapping_add(src);
            },
            Opcode::AddaLong => {
                let di = ((op >> 9) & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let si = (op & 7) as usize;
                let src = self.read_source32(st, si);
                self.a[di] = self.a[di].wrapping_add(src);
            },
            Opcode::SubaLong => {
                let di = ((op >> 9) & 7) as usize;
                let si = (op & 7) as usize;
                self.a[di] -= self.a[si];
            },
            Opcode::AndLong => {
                let n = ((op >> 9) & 7) as usize;
                let m = (op & 7) as usize;
                let src = self.read_source32(((op >> 3) & 7) as usize, m);
                self.d[n] &= src;
            },
            Opcode::BranchCond => {
                let (ofs, sz) = get_branch_offset(op, &self.bus, self.pc);
                let mut newpc = self.pc + sz;
                if (self.sr & FLAG_Z) == 0 {
                    newpc = ((startadr + 2) as i32 + ofs as i32) as u32;
                }
                self.pc = newpc;
            },
            Opcode::Dbra => {
                let si = (op & 7) as usize;
                let ofs = self.read16(self.pc) as i16;
                self.pc += 2;

                let l = self.d[si];
                let w = (l as u16).wrapping_sub(1);
                self.d[si] = (l & 0xffff0000) | (w as u32);
                if w != 0xffff {
                    self.pc = (self.pc - 2).wrapping_add((ofs as i32) as u32);
                }
            },
            Opcode::Bsr => {
                let (ofs, sz) = get_branch_offset(op, &self.bus, self.pc);
                self.pc += sz;
                self.push32(self.pc);
                self.pc = ((startadr + 2) as i32 + ofs as i32) as u32;
            },
            Opcode::Rts => {
                self.pc = self.pop32();
            },
            Opcode::Trap => {
                let no = op & 0x000f;
                // TODO: Move to super visor mode.
                let adr = self.read32(TRAP_VECTOR_START + (no * 4) as u32);
                self.push32(self.pc);
                self.pc = adr;
            },
            _ => {
                eprintln!("{:08x}: {:04x}  ; Unknown opcode", startadr, op);
                panic!("Not implemented");
            },
        }
    }

    fn push32(&mut self, value: Long) {
        let sp = self.a[SP] - 4;
        self.a[SP] = sp;
        self.write32(sp, value);
    }

    fn pop32(&mut self) -> Long {
        let oldsp = self.a[SP];
        self.a[SP] = oldsp + 4;
        self.read32(oldsp)
    }

    fn read_source8(&mut self, src: usize, m: usize) -> u8 {
        match src {
            0 => {  // move.l Dm, xx
                self.d[m] as u8
            },
            3 => {  // move.b (Am)+, xx
                let adr = self.a[m];
                self.a[m] = adr + 1;
                self.read8(adr)
            },
            7 => {  // Misc.
                match m {
                    1 => {  // move.b $XXXXXXXX.l, xx
                        let adr = self.read32(self.pc);
                        self.pc += 4;
                        self.read8(adr)
                    },
                    4 => {  // move.b #$XXXX, xx
                        let value = self.read16(self.pc);
                        self.pc += 2;
                        (value & 0xff) as u8
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

    fn read_source16(&mut self, src: usize, m: usize) -> u16 {
        match src {
            0 => {  // move.l Dm, xx
                self.d[m] as u16
            },
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
                self.d[m]
            },
            1 => {  // move.l Am, xx
                self.a[m]
            },
            2 => {  // move.l (Am), xx
                let adr = self.a[m];
                self.read32(adr)
            },
            3 => {  // move.l (Am)+, xx
                let adr = self.a[m];
                self.a[m] = adr + 4;
                self.read32(adr)
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

    fn write_destination8(&mut self, dst: usize, n: usize, value: Byte) {
        match dst {
            0 => {
                self.d[n] = (self.d[n] & 0xffffff00) | (value as u32);
            },
            3 => {
                let adr = self.a[n];
                self.write8(adr, value);
                self.a[n] = adr + 1;
            },
            7 => {
                match n {
                    1 => {
                        let d = self.read32(self.pc);
                        self.pc += 4;
                        self.write8(d, value);
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

    fn write_destination16(&mut self, dst: usize, n: usize, value: Word) {
        match dst {
            0 => {
                self.d[n] = (self.d[n] & 0xffff0000) | (value as u32);
            },
            1 => {
                self.a[n] = (self.a[n] & 0xffff0000) | (value as u32);
            },
            3 => {
                let adr = self.a[n];
                self.write16(adr, value);
                self.a[n] = adr + 2;
            },
            7 => {
                match n {
                    1 => {
                        let d = self.read32(self.pc);
                        self.pc += 4;
                        self.write16(d, value);
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

    fn write_destination32(&mut self, dst: usize, n: usize, value: Long) {
        match dst {
            0 => {
                self.d[n] = value;
            },
            1 => {
                self.a[n] = value;
            },
            3 => {
                let adr = self.a[n];
                self.write32(adr, value);
                self.a[n] = adr + 4;
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
        self.bus.read8(adr)
    }

    fn read16(&self, adr: Adr) -> Word {
        self.bus.read16(adr)
    }

    fn read32(&self, adr: Adr) -> Long {
        self.bus.read32(adr)
    }

    fn write8(&mut self, adr: Adr, value: Byte) {
        self.bus.write8(adr, value);
    }

    fn write16(&mut self, adr: Adr, value: Word) {
        self.bus.write16(adr, value);
    }

    fn write32(&mut self, adr: Adr, value: Long) {
        self.bus.write32(adr, value);
    }
}

pub fn get_branch_offset(op: Word, bus: &Bus, adr: Adr) -> (i16, u32) {
    let ofs = ((op & 0x00ff) as i8) as i16;
    if ofs != 0 {
        (ofs, 0)
    } else {
        (bus.read16(adr) as i16, 2)
    }
}
