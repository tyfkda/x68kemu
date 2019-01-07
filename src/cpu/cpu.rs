use super::bus_trait::{BusTrait};
use super::disasm::{disasm};
use super::opcode::{Opcode, INST};
use super::super::types::{Byte, Word, Long, SByte, SWord, SLong, Adr};

const SP: usize = 7;  // Stack pointer = A7 register.

const FLAG_C: Word = 1 << 0;
const FLAG_V: Word = 1 << 1;
const FLAG_Z: Word = 1 << 2;
const FLAG_N: Word = 1 << 3;

const TRAP_VECTOR_START: Adr = 0x0080;

pub struct Cpu<BusT> {
    bus: BusT,
    a: [Adr; 8],  // Address registers
    d: [Long; 8],  // Data registers
    pc: Adr,
    sr: Word,
}

impl <BusT: BusTrait> Cpu<BusT> {
    pub fn new(bus: BusT) -> Cpu<BusT> {
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
            println!("{:06x}: {}  {}", self.pc, dump_mem(&self.bus, self.pc, sz, 5), mnemonic);
            self.step();
        }
    }

    fn step(&mut self) {
        let startadr = self.pc;
        let op = self.read16(self.pc);
        self.pc += 2;
        let inst = &INST[op as usize];

        match inst.op {
            Opcode::Nop => {
                // Waste cycles.
            },
            Opcode::MoveByte => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let dt = ((op >> 6) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source8(st, si);
                self.write_destination8(dt, di, src);
            },
            Opcode::MoveLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let dt = ((op >> 6) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                self.write_destination32(dt, di, src);
            },
            Opcode::MoveWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let dt = ((op >> 6) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source16(st, si);
                self.write_destination16(dt, di, src);
            },
            Opcode::Moveq => {
                let v = op & 0xff;
                let di = (op >> 9) & 7;
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
            Opcode::MovemTo => {
                let si = (op & 7) as usize;
                let bits = self.read16(self.pc);
                self.pc += 2;
                let mut p = self.a[si];
                for i in 0..8 {
                    if (bits & (0x8000 >> i)) != 0 {
                        self.d[i] = self.read32(p);
                        p += 4;
                    }
                }
                for i in 0..8 {
                    if (bits & (0x0080 << i)) != 0 {
                        self.a[i] = self.read32(p);
                        p += 4;
                    }
                }
                self.a[si] = p;
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
            Opcode::LeaOffset => {
                let si = (op & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let ofs = self.read16(self.pc) as SWord;
                self.pc += 2;
                self.a[di] = (self.a[si] as SLong + ofs as SLong) as Long;
            },
            Opcode::LeaOffsetD => {
                let si = (op & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let next = self.read16(self.pc);
                self.pc += 2;
                if (next & 0x8f00) == 0x0000 {
                    let ofs = next as SByte;
                    let ii = ((next >> 12) & 0x07) as usize;
                    self.a[di] = (self.a[si] as SLong).wrapping_add(self.d[ii] as SWord as SLong).wrapping_add(ofs as SLong) as Adr
                } else {
                    panic!("Not implemented");
                }
            },
            Opcode::LeaOffsetPc => {
                let di = ((op >> 9) & 7) as usize;
                let ofs = self.read16(self.pc) as SWord;
                self.pc += 2;
                self.a[di] = (self.pc as SLong + ofs as SLong) as Long;
            },
            Opcode::Clr => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                match op & 0xffc0 {
                    0x4200 => {  // byte
                        self.write_destination8(dt, di, 0);
                    },
                    0x4240 => {  // word
                        self.write_destination16(dt, di, 0);
                    },
                    0x4280 => {  // long
                        self.write_destination32(dt, di, 0);
                    },
                    _ => {
                        panic!("Must not happen");
                    },
                }
            },
            Opcode::CmpByte => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source8(st, si);
                let dst = self.read_source8(0, di);
                self.set_cmp_sr(dst < src, dst == src, (dst.wrapping_sub(src) & 0x80) != 0)
            },
            Opcode::CmpWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source16(st, si);
                let dst = self.read_source16(0, di);
                self.set_cmp_sr(dst < src, dst == src, (dst.wrapping_sub(src) & 0x8000) != 0)
            },
            Opcode::CmpaLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                let dst = self.read_source32(1, di);
                self.set_cmp_sr(dst < src, dst == src, (dst.wrapping_sub(src) & 0x80000000) != 0)
            },
            Opcode::CmpmByte => {
                let si = (op & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let dst = self.read8(self.a[di]);
                let src = self.read8(self.a[si]);
                self.a[si] += 1;
                self.a[di] += 1;
                self.set_cmp_sr(dst < src, dst == src, (dst.wrapping_sub(src) & 0x80) != 0)
            },
            Opcode::TstByte => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let val = self.read_source8(st, si) as SByte;
                self.set_tst_sr(val == 0, val < 0)
            },
            Opcode::TstWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let val = self.read_source16(st, si) as SWord;
                self.set_tst_sr(val == 0, val < 0)
            },
            Opcode::TstLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let val = self.read_source32(st, si) as SLong;
                self.set_tst_sr(val == 0, val < 0)
            },
            Opcode::Reset => {
                // TODO: Implement.
            },
            Opcode::AddLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                self.d[di] = self.d[di].wrapping_add(src);
            },
            Opcode::AddaLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                self.a[di] = self.a[di].wrapping_add(src);
            },
            Opcode::AddqLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let v = conv07to18(op >> 9);
                let src = self.read_source32(st, si);
                self.write_destination32(st, si, src.wrapping_add(v as u32));
            },
            Opcode::SubaLong => {
                let si = (op & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                self.a[di] -= self.a[si];
            },
            Opcode::SubqWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let v = conv07to18(op >> 9);
                let src = self.read_source16(st, si);
                self.write_destination16(st, si, src.wrapping_sub(v));
            },
            Opcode::AndWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source16(st, si);
                self.d[di] = replace_word(self.d[di], (self.d[di] as Word) & src);
            },
            Opcode::AndLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                self.d[di] &= src;
            },
            Opcode::AndiWord => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let v = self.read16(self.pc);
                self.pc += 2;
                let src = self.read_source16(dt, di);
                self.write_destination16(dt, di, src & v);
            },
            Opcode::AslImWord => {
                let di = (op & 7) as usize;
                let shift = conv07to18(op >> 9);
                self.d[di] = replace_word(self.d[di], (self.d[di] as Word) << shift);
                // TODO: Set SR.
            },
            Opcode::AslImLong => {
                let di = (op & 7) as usize;
                let shift = conv07to18(op >> 9);
                self.d[di] <<= shift;
                // TODO: Set SR.
            },
            Opcode::RorWord => {
                let di = (op & 7) as usize;
                let si = conv07to18(op >> 9);
                let w = self.d[di] as Word;
                self.d[di] = replace_word(self.d[di], (w >> si) | (w << (8 - si)));
                // TODO: Set SR.
            },
            Opcode::RolByte => {
                let di = (op & 7) as usize;
                let si = conv07to18(op >> 9);
                let b = self.d[di] as Byte;
                self.d[di] = replace_byte(self.d[di], (b << si) | (b >> (8 - si)));
                // TODO: Set SR.
            },
            Opcode::Bcc => { self.bcond(op, (self.sr & FLAG_C) == 0); },
            Opcode::Bcs => { self.bcond(op, (self.sr & FLAG_C) != 0); },
            Opcode::Bne => { self.bcond(op, (self.sr & FLAG_Z) == 0); },
            Opcode::Beq => { self.bcond(op, (self.sr & FLAG_Z) != 0); },
            Opcode::Dbra => {
                let si = (op & 7) as usize;
                let ofs = self.read16(self.pc) as SWord;

                let l = self.d[si];
                let w = (l as u16).wrapping_sub(1);
                self.d[si] = replace_word(l, w);
                self.pc = if w != 0xffff { (self.pc as SLong).wrapping_add(ofs as SLong) as Adr } else { self.pc + 2 }
            },
            Opcode::Bsr => {
                let (ofs, sz) = get_branch_offset(op, &self.bus, self.pc);
                self.pc += sz;
                self.push32(self.pc);
                self.pc = ((startadr + 2) as i32 + ofs as i32) as u32;
            },
            Opcode::JsrA => {
                let si = (op & 7) as usize;
                let adr = if (op & 15) < 8 {
                    self.a[si]
                } else {
                    let offset = self.read16(self.pc);
                    self.pc += 2;
                    panic!("Not implemented: JSR (${:04x}, A{})", offset, si);
                };
                self.push32(self.pc);
                self.pc = adr;
            },
            Opcode::Rts => {
                self.pc = self.pop32();
            },
            Opcode::Rte => {
                self.pc = self.pop32();
                // TODO: Switch to user mode.
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

    fn bcond(&mut self, op: Word, cond: bool) {
        let (ofs, sz) = get_branch_offset(op, &self.bus, self.pc);
        self.pc = if cond { (self.pc as SLong).wrapping_add(ofs) as Adr } else { self.pc + sz };
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

    fn read_source8(&mut self, src: usize, m: usize) -> Byte {
        match src {
            0 => {  // move.l Dm, xx
                self.d[m] as u8
            },
            2 => {  // move.b (Am), xx
                let adr = self.a[m];
                self.read8(adr)
            },
            3 => {  // move.b (Am)+, xx
                let adr = self.a[m];
                self.a[m] = adr + 1;
                self.read8(adr)
            },
            5 => {  // move.b (123, Am), xx
                let ofs = self.read16(self.pc) as SWord;
                self.pc += 2;
                self.read8((self.a[m] as SLong + ofs as SLong) as Adr)
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

    fn read_source16(&mut self, src: usize, m: usize) -> Word {
        match src {
            0 => {  // move.w Dm, xx
                self.d[m] as u16
            },
            2 => {  // move.w (Am), xx
                let adr = self.a[m];
                self.read16(adr)
            },
            3 => {  // move.w (Am)+, xx
                let adr = self.a[m];
                self.a[m] = adr + 2;
                self.read16(adr)
            },
            5 => {  // move.w (123, Am), xx
                let ofs = self.read16(self.pc) as SWord;
                self.pc += 2;
                self.read16((self.a[m] as SLong + ofs as SLong) as Adr)
            },
            7 => {  // Misc.
                match m {
                    1 => {  // move.b $XXXXXXXX.l, xx
                        let adr = self.read32(self.pc);
                        self.pc += 4;
                        self.read16(adr)
                    },
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

    fn read_source32(&mut self, src: usize, m: usize) -> Long {
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
            5 => {  // move.l (123, Am), xx
                let ofs = self.read16(self.pc) as SWord;
                self.pc += 2;
                self.read32((self.a[m] as SLong + ofs as SLong) as Adr)
            },
            7 => {  // Misc.
                match m {
                    1 => {  // move.b $XXXXXXXX.l, xx
                        let adr = self.read32(self.pc);
                        self.pc += 4;
                        self.read32(adr)
                    },
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
                self.d[n] = replace_byte(self.d[n], value);
            },
            3 => {
                let adr = self.a[n];
                self.write8(adr, value);
                self.a[n] = adr + 1;
            },
            5 => {  // move.b xx, (123, An)
                let ofs = self.read16(self.pc) as SWord;
                self.pc += 2;
                self.write8((self.a[n] as SLong + ofs as SLong) as Adr, value);
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
                self.d[n] = replace_word(self.d[n], value);
            },
            1 => {
                self.a[n] = replace_word(self.a[n], value);
            },
            2 => {  // move.w xx, (An)
                self.write16(self.a[n], value);
            },
            3 => {
                let adr = self.a[n];
                self.write16(adr, value);
                self.a[n] = adr + 2;
            },
            5 => {  // move.w xx, (123, An)
                let ofs = self.read16(self.pc) as SWord;
                self.pc += 2;
                self.write16((self.a[n] as SLong + ofs as SLong) as Adr, value);
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
            4 => {
                let adr = self.a[n] - 4;
                self.a[n] = adr;
                self.write32(adr, value);
            },
            5 => {  // move.l xx, (123, An)
                let ofs = self.read16(self.pc) as SWord;
                self.pc += 2;
                self.write32((self.a[n] as SLong + ofs as SLong) as Adr, value);
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

    fn set_cmp_sr(&mut self, less: bool, eq: bool, neg: bool) {
        // TODO: Check flag is true.
        let mut c = 0;
        if less {
            c |= FLAG_C;
        }
        if eq {
            c |= FLAG_Z;
        }
        if neg {
            c |= FLAG_N;
        }
        self.sr = (self.sr & 0xff00) | c;
    }

    fn set_tst_sr(&mut self, zero: bool, neg: bool) {
        let mut sr = self.sr;
        sr &= !(FLAG_V | FLAG_C | FLAG_Z | FLAG_N);
        if zero {
            sr |= FLAG_Z;
        }
        if neg {
            sr |= FLAG_N;
        }
        self.sr = sr;
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

pub fn get_branch_offset<BusT: BusTrait>(op: Word, bus: &BusT, adr: Adr) -> (SLong, u32) {
    let ofs = op & 0x00ff;
    match ofs {
        0 => {
            (bus.read16(adr) as SWord as SLong, 2)
        },
        0xff => {
            (bus.read32(adr) as SLong, 4)
        },
        _ => {
            (ofs as SByte as SWord as SLong , 0)
        },
    }
}

// Return 0~7 => 8,1~7
pub fn conv07to18(x: Word) -> Word {
    ((x & 7).wrapping_sub(1) & 7) + 1
}

#[test]
fn test_conv07to18() {
    assert_eq!(8, conv07to18(0));
    assert_eq!(1, conv07to18(1));
    assert_eq!(7, conv07to18(7));
}

#[test]
fn test_shift_byte() {
    let b: Byte = 0xa5;  // 0b10100101
    assert_eq!(0x28 as Byte, b << 3);
    assert_eq!(0x29 as Byte, b >> 2);
}

fn replace_byte(x: Long, b: Byte) -> Long {
    (x & 0xffffff00) | (b as Long)
}

#[test]
fn test_replace_byte() {
    assert_eq!(0x123456ab, replace_byte(0x12345678, 0xab));
}

fn replace_word(x: Long, w: Word) -> Long {
    (x & 0xffff0000) | (w as Long)
}

#[test]
fn test_replace_word() {
    assert_eq!(0x1234abcd, replace_word(0x12345678, 0xabcd));
}

fn dump_mem<BusT: BusTrait>(bus: &BusT, adr: Adr, sz: usize, max: usize) -> String {
    let arr = (0..max).map(|i| {
        if i * 2 < sz {
            format!("{:04x}", bus.read16(adr + (i as u32) * 2))
        } else {
            String::from("    ")
        }
    });
    arr.collect::<Vec<String>>().join(" ")
}
