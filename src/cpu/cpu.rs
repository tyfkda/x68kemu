use std::panic;

use super::bus_trait::BusTrait;
use super::registers::Registers;
use super::disasm::disasm;
use super::opcode::{Opcode, INST};
use super::util::{get_branch_offset, conv07to18};
use super::super::types::{Byte, Word, Long, SByte, SWord, SLong, Adr};

const SP: usize = 7;  // Stack pointer = A7 register.

const FLAG_C: Word = 1 << 0;
const FLAG_V: Word = 1 << 1;
const FLAG_Z: Word = 1 << 2;
const FLAG_N: Word = 1 << 3;
const FLAG_X: Word = 1 << 4;

const TRAP_VECTOR_START: Adr = 0x0080;

pub struct Cpu<BusT> {
    regs: Registers,
    bus: BusT,
}

impl<BusT: BusTrait> Cpu<BusT> {
    pub fn new(bus: BusT) -> Self {
        let regs = Registers::new();
        Self {
            regs,
            bus,
        }
    }

    pub fn reset(&mut self) {
        self.bus.reset();
        self.regs.sr = 0;
        self.regs.a[SP] = self.read32(0x000000);
        self.regs.pc = self.read32(0x000004);
    }

    #[allow(dead_code)]
    pub fn set_pc(&mut self, pc: Adr) {
        self.regs.pc = pc;
    }

    pub fn run_cycles(&mut self, cycles: usize) {
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            for _ in 0..cycles {
                let (sz, mnemonic) = disasm(&mut self.bus, self.regs.pc);
                println!("{:06x}: {}  {}", self.regs.pc, dump_mem(&mut self.bus, self.regs.pc, sz, 5), mnemonic);
                self.step();
            }
        }));
        if result.is_err() {
            eprintln!("panic catched: pc={:06x}, op={:04x}", self.regs.pc, self.bus.read16(self.regs.pc));
            result.unwrap_or_else(|e| panic::resume_unwind(e));
        }
    }

    fn step(&mut self) {
        let startadr = self.regs.pc;
        let op = self.read16(self.regs.pc);
        self.regs.pc += 2;
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

                let mut ccr = 0;
                if src == 0          { ccr |= FLAG_Z; }
                if (src & 0x80) != 0 { ccr |= FLAG_N; }
                self.regs.sr = (self.regs.sr & !(FLAG_C | FLAG_V | FLAG_Z | FLAG_N)) | ccr;
            },
            Opcode::MoveWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let dt = ((op >> 6) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source16(st, si);
                self.write_destination16(dt, di, src);

                let mut ccr = 0;
                if src == 0            { ccr |= FLAG_Z; }
                if (src & 0x8000) != 0 { ccr |= FLAG_N; }
                self.regs.sr = (self.regs.sr & !(FLAG_C | FLAG_V | FLAG_Z | FLAG_N)) | ccr;
            },
            Opcode::MoveLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let dt = ((op >> 6) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                self.write_destination32(dt, di, src);

                let mut ccr = 0;
                if src == 0                { ccr |= FLAG_Z; }
                if (src & 0x80000000) != 0 { ccr |= FLAG_N; }
                self.regs.sr = (self.regs.sr & !(FLAG_C | FLAG_V | FLAG_Z | FLAG_N)) | ccr;
            },
            Opcode::Moveq => {
                let v = op & 0xff;
                let di = (op >> 9) & 7;
                let src = if v < 0x80 { v as i16 } else { -256 + v as i16 };
                self.regs.d[di as usize] = (src as i32) as u32;

                let mut ccr = 0;
                if src == 0 { ccr |= FLAG_Z; }
                if src < 0  { ccr |= FLAG_N; }
                self.regs.sr = (self.regs.sr & !(FLAG_C | FLAG_V | FLAG_Z | FLAG_N)) | ccr;
            },
            Opcode::MovemFrom => {
                let di = (op & 7) as usize;
                let bits = self.read16(self.regs.pc);
                self.regs.pc += 2;
                let mut p = self.regs.a[di];
                for i in 0..8 {
                    if (bits & (0x0001 << i)) != 0 {
                        p -= 4;
                        self.write32(p, self.regs.a[7 - i]);
                    }
                }
                for i in 0..8 {
                    if (bits & (0x0100 << i)) != 0 {
                        p -= 4;
                        self.write32(p, self.regs.d[7 - i]);
                    }
                }
                self.regs.a[di] = p;
            },
            Opcode::MovemTo => {
                let di = (op & 7) as usize;
                let bits = self.read16(self.regs.pc);
                self.regs.pc += 2;
                let mut p = self.regs.a[di];
                for i in 0..8 {
                    if (bits & (0x0001 << i)) != 0 {
                        self.regs.d[i] = self.read32(p);
                        p += 4;
                    }
                }
                for i in 0..8 {
                    if (bits & (0x0100 << i)) != 0 {
                        self.regs.a[i] = self.read32(p);
                        p += 4;
                    }
                }
                self.regs.a[di] = p;
            },
            Opcode::MoveToSrIm => {
                self.regs.sr = self.read16(self.regs.pc);
                self.regs.pc += 2;
            },
            Opcode::MoveToSr => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                self.regs.sr = self.read_source16(st, si);
            },
            Opcode::MoveFromSr => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                self.write_destination16(dt, di, self.regs.sr);
            },
            Opcode::LeaDirect => {
                let di = ((op >> 9) & 7) as usize;
                let value = self.read32(self.regs.pc);
                self.regs.pc += 4;
                self.regs.a[di] = value;
            },
            Opcode::LeaOffset => {
                let si = (op & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let ofs = self.read16(self.regs.pc) as SWord;
                self.regs.pc += 2;
                self.regs.a[di] = (self.regs.a[si] as SLong + ofs as SLong) as Long;
            },
            Opcode::LeaOffsetD => {
                let si = (op & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let next = self.read16(self.regs.pc);
                self.regs.pc += 2;
                if (next & 0x8f00) == 0x0000 {
                    let ofs = next as SByte;
                    let ii = ((next >> 12) & 0x07) as usize;
                    self.regs.a[di] = (self.regs.a[si] as SLong).wrapping_add(self.regs.d[ii] as SWord as SLong).wrapping_add(ofs as SLong) as Adr
                } else {
                    panic!("Not implemented");
                }
            },
            Opcode::LeaOffsetPc => {
                let di = ((op >> 9) & 7) as usize;
                let ofs = self.read16(self.regs.pc) as SWord;
                self.regs.pc += 2;
                self.regs.a[di] = (self.regs.pc as SLong + ofs as SLong) as Long;
            },
            Opcode::ClrByte => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                self.write_destination8(dt, di, 0);
            },
            Opcode::ClrWord => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                self.write_destination16(dt, di, 0);
            },
            Opcode::ClrLong => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                self.write_destination32(dt, di, 0);
            },
            Opcode::Swap => {
                let di = (op & 7) as usize;
                let v = self.regs.d[di];
                self.regs.d[di] = v.rotate_right(16);
            },
            Opcode::CmpByte => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source8(st, si);
                let dst = self.read_source8(0, di);
                let res = dst.wrapping_sub(src);
                self.set_cmp_sr(dst < src, dst == src, (((src ^ dst) & (res ^ dst)) & 0x80) != 0, (res & 0x80) != 0);
            },
            Opcode::CmpWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source16(st, si);
                let dst = self.read_source16(0, di);
                let res = dst.wrapping_sub(src);
                self.set_cmp_sr(dst < src, dst == src, (((src ^ dst) & (res ^ dst)) & 0x8000) != 0, (res & 0x8000) != 0);
            },
            Opcode::CmpLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                let dst = self.read_source32(0, di);
                let res = dst.wrapping_sub(src);
                self.set_cmp_sr(dst < src, dst == src, (((src ^ dst) & (res ^ dst)) & 0x80000000) != 0, (res & 0x80000000) != 0);
            },
            Opcode::CmpiByte => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let src = self.read16(self.regs.pc) as Byte;
                self.regs.pc += 2;
                let dst = self.read_source8(dt, di);
                let res = dst.wrapping_sub(src);
                self.set_cmp_sr(dst < src, dst == src, (((src ^ dst) & (res ^ dst)) & 0x80) != 0, (res & 0x80) != 0);
            },
            Opcode::CmpiWord => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let src = self.read16(self.regs.pc);
                self.regs.pc += 2;
                let dst = self.read_source16(dt, di);
                let res = dst.wrapping_sub(src);
                self.set_cmp_sr(dst < src, dst == src, (((src ^ dst) & (res ^ dst)) & 0x8000) != 0, (res & 0x8000) != 0);
            },
            Opcode::CmpaLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                let dst = self.read_source32(1, di);
                let res = dst.wrapping_sub(src);
                self.set_cmp_sr(dst < src, dst == src, (((src ^ dst) & (res ^ dst)) & 0x80000000) != 0, (res & 0x80000000) != 0);
            },
            Opcode::CmpmByte => {
                let si = (op & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let dst = self.read8(self.regs.a[di]);
                let src = self.read8(self.regs.a[si]);
                self.regs.a[si] += 1;
                self.regs.a[di] += 1;
                let res = dst.wrapping_sub(src);
                self.set_cmp_sr(dst < src, dst == src, (((src ^ dst) & (res ^ dst)) & 0x80) != 0, (res & 0x80) != 0);
            },
            Opcode::TstByte => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let val = self.read_source8(st, si) as SByte;
                self.set_tst_sr(val == 0, val < 0);
            },
            Opcode::TstWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let val = self.read_source16(st, si) as SWord;
                self.set_tst_sr(val == 0, val < 0);
            },
            Opcode::TstLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let val = self.read_source32(st, si) as SLong;
                self.set_tst_sr(val == 0, val < 0);
            },
            Opcode::BtstIm => {
                let bit = self.read16(self.regs.pc);
                self.regs.pc += 2;
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                if st < 2 {  // Data or address register: 32bit.
                    let val = self.read_source32(st, si);
                    let zero = (val & (1 << (bit & 31))) == 0;
                    self.regs.sr = (self.regs.sr & !FLAG_Z) | (if zero {FLAG_Z} else {0});
                } else {  // Memory: 8bit.
                    let val = self.read_source8(st, si);
                    let zero = (val & (1 << (bit & 7))) == 0;
                    self.regs.sr = (self.regs.sr & !FLAG_Z) | (if zero {FLAG_Z} else {0});
                }
            },
            Opcode::BclrIm => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let bit = self.read16(self.regs.pc);
                self.regs.pc += 2;
                if dt < 2 {
                    let dst = self.read_source32_incpc(dt, di, false);
                    self.write_destination32(dt, di, dst & !(1 << (bit & 31)));
                } else {
                    let dst = self.read_source8_incpc(dt, di, false);
                    self.write_destination8(dt, di, dst & !(1 << (bit & 7)));
                }
            },
            Opcode::Bset => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let si = ((op >> 9) & 7) as usize;
                if dt < 2 {  // Register: 32bit
                    let dst = self.read_source32_incpc(dt, di, false);
                    self.write_destination32(dt, di, dst | (1 << (self.regs.d[si] & 31)));
                } else {  // Memory: 8bit
                    let dst = self.read_source8_incpc(dt, di, false);
                    self.write_destination8(dt, di, dst | (1 << (self.regs.d[si] & 7)));
                }
                // TODO: Update status.
            },
            Opcode::BsetIm => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let bit = self.read16(self.regs.pc);
                self.regs.pc += 2;
                if dt < 2 {  // Register: 32bit
                    let dst = self.read_source32_incpc(dt, di, false);
                    self.write_destination32(dt, di, dst | (1 << (bit & 31)));
                } else {  // Memory: 8bit
                    let dst = self.read_source8_incpc(dt, di, false);
                    self.write_destination8(dt, di, dst | (1 << (bit & 7)));
                }
            },
            Opcode::AddByte => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source8(st, si);
                let val = self.regs.d[di];
                self.regs.d[di] = replace_byte(val, (val as Byte).wrapping_add(src));
            },
            Opcode::AddWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source16(st, si);
                let val = self.regs.d[di];
                self.regs.d[di] = replace_word(val, (val as Word).wrapping_add(src));
            },
            Opcode::AddLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                self.regs.d[di] = self.regs.d[di].wrapping_add(src);
            },
            Opcode::AddiByte => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let v = self.read16(self.regs.pc) as Byte;
                self.regs.pc += 2;
                let src = self.read_source8_incpc(dt, di, false);
                self.write_destination8(dt, di, src.wrapping_add(v));
                // TODO: Update all flags
            },
            Opcode::AddiWord => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let v = self.read16(self.regs.pc);
                self.regs.pc += 2;
                let src = self.read_source16_incpc(dt, di, false);
                self.write_destination16(dt, di, src.wrapping_add(v));
                // TODO: Update all flags
            },
            Opcode::AddaLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                self.regs.a[di] = self.regs.a[di].wrapping_add(src);
            },
            Opcode::AddqByte => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let v = conv07to18(op >> 9);
                let src = self.read_source8_incpc(st, si, false);
                self.write_destination8(st, si, (v as Byte).wrapping_add(src));
            },
            Opcode::AddqWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let v = conv07to18(op >> 9);
                let src = self.read_source16_incpc(st, si, false);
                self.write_destination16(st, si, (v as Word).wrapping_add(src));
            },
            Opcode::AddqLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let v = conv07to18(op >> 9);
                let src = self.read_source32_incpc(st, si, false);
                self.write_destination32(st, si, (v as Long).wrapping_add(src));
            },
            Opcode::SubByte => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source8(st, si);
                let val = self.regs.d[di];
                self.regs.d[di] = replace_byte(val, (val as Byte).wrapping_sub(src));
            },
            Opcode::SubWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source16(st, si);
                let val = self.regs.d[di];
                self.regs.d[di] = replace_word(val, (val as Word).wrapping_sub(src));
            },
            Opcode::SubiByte => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let v = self.read16(self.regs.pc) as Byte;
                self.regs.pc += 2;
                let src = self.read_source8_incpc(dt, di, false);
                self.write_destination8(dt, di, src.wrapping_sub(v));
                // TODO: Update all flags
            },
            Opcode::SubaLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                self.regs.a[di] = self.regs.a[di].wrapping_sub(src);
            },
            Opcode::SubqWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let v = conv07to18(op >> 9);
                let src = self.read_source16_incpc(st, si, false);
                let val = src.wrapping_sub(v);
                self.write_destination16(st, si, val);

                // TODO: Update all flags
                let mut sr = self.regs.sr & !FLAG_Z;
                if val == 0 { sr |= FLAG_Z; }
                self.regs.sr = sr;
            },
            Opcode::SubqLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let v = conv07to18(op >> 9);
                let src = self.read_source32_incpc(st, si, false);
                let val = src.wrapping_sub(v as u32);
                self.write_destination32(st, si, val);

                // TODO: Update all flags
                let mut sr = self.regs.sr & !FLAG_Z;
                if val == 0 { sr |= FLAG_Z; }
                self.regs.sr = sr;
            },
            Opcode::MuluWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source16(st, si);
                self.regs.d[di] = ((self.regs.d[di] as Word) as Long).wrapping_mul(src as Long);
            },
            Opcode::AndByte => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source8(st, si);
                let dst = self.regs.d[di];
                let res = (dst as Byte) & src;
                self.regs.d[di] = replace_byte(dst, res);
                self.set_and_sr(res == 0, (res & 0x80) != 0);
            },
            Opcode::AndWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source16(st, si);
                let dst = self.regs.d[di];
                let res = (dst as Word) & src;
                self.regs.d[di] = replace_word(dst, res);
                self.set_and_sr(res == 0, (res & 0x8000) != 0);
            },
            Opcode::AndLong => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source32(st, si);
                let dst = self.regs.d[di];
                let res = dst & src;
                self.regs.d[di] = res;
                self.set_and_sr(res == 0, (res & 0x80000000) != 0);
            },
            Opcode::AndiWord => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let v = self.read16(self.regs.pc);
                self.regs.pc += 2;
                let dst = self.read_source16_incpc(dt, di, false);
                let res = dst & v;
                self.write_destination16(dt, di, res);
                self.set_and_sr(res == 0, (res & 0x8000) != 0);
            },
            Opcode::OrByte => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source8(st, si);
                let dst = self.regs.d[di];
                self.regs.d[di] = replace_byte(dst, (dst as Byte) | src);
                // TODO: Update all flags
            },
            Opcode::OrWord => {
                let si = (op & 7) as usize;
                let st = ((op >> 3) & 7) as usize;
                let di = ((op >> 9) & 7) as usize;
                let src = self.read_source16(st, si);
                let dst = self.regs.d[di];
                self.regs.d[di] = replace_word(dst, (dst as Word) | src);
                // TODO: Update all flags
            },
            Opcode::OriByte => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let v = self.read16(self.regs.pc) as Byte;
                self.regs.pc += 2;
                let src = self.read_source8_incpc(dt, di, false);
                self.write_destination8(dt, di, src | v);
                // TODO: Update all flags
            },
            Opcode::OriWord => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let v = self.read16(self.regs.pc);
                self.regs.pc += 2;
                let src = self.read_source16_incpc(dt, di, false);
                self.write_destination16(dt, di, src | v);
                // TODO: Update all flags
            },
            Opcode::EorByte => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let si = ((op >> 9) & 7) as usize;
                let dst = self.read_source8_incpc(dt, di, false);
                self.write_destination8(dt, di, (self.regs.d[si] as Byte) ^ dst);
                // TODO: Update all flags
            },
            Opcode::EoriByte => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let v = self.read16(self.regs.pc) as Byte;
                self.regs.pc += 2;
                let src = self.read_source8_incpc(dt, di, false);
                self.write_destination8(dt, di, src ^ v);
                // TODO: Update all flags
            },
            Opcode::EoriWord => {
                let di = (op & 7) as usize;
                let dt = ((op >> 3) & 7) as usize;
                let v = self.read16(self.regs.pc);
                self.regs.pc += 2;
                let src = self.read_source16_incpc(dt, di, false);
                self.write_destination16(dt, di, src ^ v);
                // TODO: Update all flags
            },
            Opcode::AslImByte => {
                let di = (op & 7) as usize;
                let shift = conv07to18(op >> 9);
                self.regs.d[di] = replace_byte(self.regs.d[di], (self.regs.d[di] as Byte) << shift);
                // TODO: Set SR.
            },
            Opcode::AslImWord => {
                let di = (op & 7) as usize;
                let shift = conv07to18(op >> 9);
                self.regs.d[di] = replace_word(self.regs.d[di], (self.regs.d[di] as Word) << shift);
                // TODO: Set SR.
            },
            Opcode::AslImLong => {
                let di = (op & 7) as usize;
                let shift = conv07to18(op >> 9);
                self.regs.d[di] <<= shift;
                // TODO: Set SR.
            },
            Opcode::LsrImByte => {
                let di = (op & 7) as usize;
                let shift = conv07to18(op >> 9);
                let val = self.regs.d[di];
                let newval = (val as Byte) >> shift;
                self.regs.d[di] = replace_byte(val, newval);

                let mut sr = self.regs.sr & !(FLAG_X | FLAG_N | FLAG_Z | FLAG_V | FLAG_C);
                if val & (1 << (shift - 1)) != 0 { sr |= FLAG_X | FLAG_C; }
                if newval == 0 { sr |= FLAG_Z; }
                self.regs.sr = sr;
            },
            Opcode::LsrImWord => {
                let di = (op & 7) as usize;
                let shift = conv07to18(op >> 9);
                let val = self.regs.d[di];
                let newval = (val as Word) >> shift;
                self.regs.d[di] = replace_word(val, newval);

                let mut sr = self.regs.sr & !(FLAG_X | FLAG_N | FLAG_Z | FLAG_V | FLAG_C);
                if val & (1 << (shift - 1)) != 0 { sr |= FLAG_X | FLAG_C; }
                if newval == 0 { sr |= FLAG_Z; }
                self.regs.sr = sr;
            },
            Opcode::LslImWord => {
                let di = (op & 7) as usize;
                let shift = conv07to18(op >> 9);
                let val = self.regs.d[di];
                self.regs.d[di] = replace_word(val, (val as Word) << shift);
                // TODO: Set SR.
            },
            Opcode::RorImWord => {
                let di = (op & 7) as usize;
                let si = conv07to18(op >> 9);
                let dst = self.regs.d[di];
                let w = dst as Word;
                self.regs.d[di] = replace_word(dst, (w >> si) | (w << (8 - si)));
                // TODO: Set SR.
            },
            Opcode::RorImLong => {
                let di = (op & 7) as usize;
                let si = conv07to18(op >> 9);
                let dst = self.regs.d[di];
                self.regs.d[di] = (dst >> si) | (dst << (8 - si));
                // TODO: Set SR.
            },
            Opcode::RolWord => {
                let di = (op & 7) as usize;
                let si = ((op >> 9) & 7) as usize;
                let val = self.regs.d[di] as Word;
                let shift = self.regs.d[si] & 15;
                self.regs.d[di] = replace_word(self.regs.d[di], (val << shift) | (val >> (16 - shift)));
                // TODO: Set SR.
            },
            Opcode::RolImByte => {
                let di = (op & 7) as usize;
                let si = conv07to18(op >> 9);
                let val = self.regs.d[di] as Byte;
                self.regs.d[di] = replace_byte(self.regs.d[di], (val << si) | (val >> (8 - si)));
                // TODO: Set SR.
            },
            Opcode::ExtWord => {
                let di = (op & 7) as usize;
                let src = self.regs.d[di];
                self.regs.d[di] = replace_word(src, src as SByte as SWord as Word);
            },
            Opcode::Bra => { self.bcond(op, true); },
            Opcode::Bcc => { self.bcond(op, (self.regs.sr & FLAG_C) == 0); },
            Opcode::Bcs => { self.bcond(op, (self.regs.sr & FLAG_C) != 0); },
            Opcode::Bne => { self.bcond(op, (self.regs.sr & FLAG_Z) == 0); },
            Opcode::Beq => { self.bcond(op, (self.regs.sr & FLAG_Z) != 0); },
            Opcode::Bpl => { self.bcond(op, (self.regs.sr & FLAG_N) == 0); },
            Opcode::Bmi => { self.bcond(op, (self.regs.sr & FLAG_N) != 0); },
            Opcode::Bge => { let nv = self.regs.sr & (FLAG_N | FLAG_V); self.bcond(op, nv == 0 || nv == (FLAG_N | FLAG_V)); },
            Opcode::Blt => { let nv = self.regs.sr & (FLAG_N | FLAG_V); self.bcond(op, nv == FLAG_N || nv == FLAG_V); },
            Opcode::Bgt => { let nv = self.regs.sr & (FLAG_N | FLAG_V); self.bcond(op, (self.regs.sr & FLAG_Z) == 0 && (nv == 0 || nv == (FLAG_N | FLAG_V))); },
            Opcode::Ble => { let nv = self.regs.sr & (FLAG_N | FLAG_V); self.bcond(op, (self.regs.sr & FLAG_Z) != 0 || nv == FLAG_N || nv == FLAG_V); },
            Opcode::Dbra => {
                let si = (op & 7) as usize;
                let ofs = self.read16(self.regs.pc) as SWord;

                let l = self.regs.d[si];
                let w = (l as u16).wrapping_sub(1);
                self.regs.d[si] = replace_word(l, w);
                self.regs.pc = if w != 0xffff { (self.regs.pc as SLong).wrapping_add(ofs as SLong) as Adr } else { self.regs.pc + 2 }
            },
            Opcode::Bsr => {
                let (ofs, sz) = get_branch_offset(op, &mut self.bus, self.regs.pc);
                self.regs.pc += sz;
                self.push32(self.regs.pc);
                self.regs.pc = ((startadr + 2) as i32 + ofs) as u32;
            },
            Opcode::JsrA => {
                let si = (op & 7) as usize;
                let adr = if (op & 15) < 8 {
                    self.regs.a[si]
                } else {
                    let offset = self.read16(self.regs.pc);
                    self.regs.pc += 2;
                    panic!("Not implemented: JSR (${:04x}, A{})", offset, si);
                };
                self.push32(self.regs.pc);
                self.regs.pc = adr;
            },
            Opcode::Rts => {
                self.regs.pc = self.pop32();
            },
            Opcode::Rte => {
                self.regs.pc = self.pop32();
                // TODO: Switch to user mode.
            },
            Opcode::Trap => {
                let no = op & 0x000f;
                // TODO: Move to super visor mode.
                let adr = self.read32(TRAP_VECTOR_START + (no * 4) as u32);
                self.push32(self.regs.pc);
                self.regs.pc = adr;
            },
            Opcode::Reset => {
                // TODO: Implement.
            },
            _ => {
                eprintln!("{:08x}: {:04x}  ; Unknown opcode", startadr, op);
                panic!("Not implemented");
            },
        }
    }

    fn bcond(&mut self, op: Word, cond: bool) {
        let (ofs, sz) = get_branch_offset(op, &mut self.bus, self.regs.pc);
        self.regs.pc = if cond { (self.regs.pc as SLong).wrapping_add(ofs) as Adr } else { self.regs.pc + sz };
    }

    fn push32(&mut self, value: Long) {
        let sp = self.regs.a[SP] - 4;
        self.regs.a[SP] = sp;
        self.write32(sp, value);
    }

    fn pop32(&mut self) -> Long {
        let oldsp = self.regs.a[SP];
        self.regs.a[SP] = oldsp + 4;
        self.read32(oldsp)
    }

    fn read_source8(&mut self, src: usize, m: usize) -> Byte {
        self.read_source8_incpc(src, m, true)
    }
    fn read_source8_incpc(&mut self, src: usize, m: usize, incpc: bool) -> Byte {
        match src {
            0 => {  // move.l Dm, xx
                self.regs.d[m] as u8
            },
            2 => {  // move.b (Am), xx
                let adr = self.regs.a[m];
                self.read8(adr)
            },
            3 => {  // move.b (Am)+, xx
                let adr = self.regs.a[m];
                if incpc { self.regs.a[m] = adr + 1; }
                self.read8(adr)
            },
            5 => {  // move.b (123, Am), xx
                let ofs = self.read16(self.regs.pc) as SWord;
                if incpc { self.regs.pc += 2; }
                self.read8((self.regs.a[m] as SLong + ofs as SLong) as Adr)
            },
            7 => {  // Misc.
                match m {
                    1 => {  // move.b $XXXXXXXX.l, xx
                        let adr = self.read32(self.regs.pc);
                        if incpc { self.regs.pc += 4; }
                        self.read8(adr)
                    },
                    4 => {  // move.b #$XXXX, xx
                        if incpc {
                            let value = self.read16(self.regs.pc);
                            if incpc { self.regs.pc += 2; }
                            (value & 0xff) as u8
                        } else {
                            panic!("Not implemented, m={}", m);
                        }
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
        self.read_source16_incpc(src, m, true)
    }
    fn read_source16_incpc(&mut self, src: usize, m: usize, incpc: bool) -> Word {
        match src {
            0 => {  // move.w Dm, xx
                self.regs.d[m] as u16
            },
            2 => {  // move.w (Am), xx
                let adr = self.regs.a[m];
                self.read16(adr)
            },
            3 => {  // move.w (Am)+, xx
                let adr = self.regs.a[m];
                if incpc { self.regs.a[m] = adr + 2; }
                self.read16(adr)
            },
            5 => {  // move.w (123, Am), xx
                let ofs = self.read16(self.regs.pc) as SWord;
                if incpc { self.regs.pc += 2; }
                self.read16((self.regs.a[m] as SLong + ofs as SLong) as Adr)
            },
            6 => {  // Memory Indirect Pre-indexed: move.w xx, (123, An, Dx)
                let extension = self.read16(self.regs.pc);
                self.regs.pc += 2;
                if (extension & 0x100) != 0 {
                    panic!("Not implemented, src=6/{:04x}", extension);
                } else {
                    let ofs = extension as SByte as SLong;
                    let da = (extension & 0x8000) != 0;  // Displacement is address register?
                    let dr = ((extension >> 12) & 7) as usize;  // Displacement register.
                    let dl = (extension & 0x0800) != 0;  // Displacement long?
                    let regofs = if dl { (if da {self.regs.a[dr]} else {self.regs.d[dr]}) as SLong } else { (if da {self.regs.a[dr]} else {self.regs.d[dr]}) as SWord as SLong };
                    let adr = (ofs + (self.regs.a[m] as SLong) + regofs) as Long;
                    self.read16(adr)
                }
            },
            7 => {  // Misc.
                match m {
                    1 => {  // move.b $XXXXXXXX.l, xx
                        let adr = self.read32(self.regs.pc);
                        if incpc { self.regs.pc += 4; }
                        self.read16(adr)
                    },
                    4 => {  // move.w #$XXXX, xx
                        if incpc {
                            let value = self.read16(self.regs.pc);
                            self.regs.pc += 2;
                            value
                        } else {
                            self.regs.sr
                        }
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
        self.read_source32_incpc(src, m, true)
    }
    fn read_source32_incpc(&mut self, src: usize, m: usize, incpc: bool) -> Long {
        match src {
            0 => {  // move.l Dm, xx
                self.regs.d[m]
            },
            1 => {  // move.l Am, xx
                self.regs.a[m]
            },
            2 => {  // move.l (Am), xx
                let adr = self.regs.a[m];
                self.read32(adr)
            },
            3 => {  // move.l (Am)+, xx
                let adr = self.regs.a[m];
                if incpc { self.regs.a[m] = adr + 4; }
                self.read32(adr)
            },
            5 => {  // move.l (123, Am), xx
                let ofs = self.read16(self.regs.pc) as SWord;
                if incpc { self.regs.pc += 2; }
                self.read32((self.regs.a[m] as SLong + ofs as SLong) as Adr)
            },
            6 => {  // Memory Indirect Pre-indexed: move.l xx, (123, An, Dx)
                let extension = self.read16(self.regs.pc);
                self.regs.pc += 2;
                if (extension & 0x100) != 0 {
                    panic!("Not implemented, src=6/{:04x}", extension);
                } else {
                    let ofs = extension as SByte as SLong;
                    let da = (extension & 0x8000) != 0;  // Displacement is address register?
                    let dr = ((extension >> 12) & 7) as usize;  // Displacement register.
                    let dl = (extension & 0x0800) != 0;  // Displacement long?
                    let regofs = if dl { (if da {self.regs.a[dr]} else {self.regs.d[dr]}) as SLong } else { (if da {self.regs.a[dr]} else {self.regs.d[dr]}) as SWord as SLong };
                    let adr = (ofs + (self.regs.a[m] as SLong) + regofs) as Long;
                    self.read32(adr)
                }
            },
            7 => {  // Misc.
                match m {
                    1 => {  // move.b $XXXXXXXX.l, xx
                        let adr = self.read32(self.regs.pc);
                        if incpc { self.regs.pc += 4; }
                        self.read32(adr)
                    },
                    4 => {  // move.l #$XXXX, xx
                        if incpc {
                            let value = self.read32(self.regs.pc);
                            self.regs.pc += 4;
                            value
                        } else {
                            panic!("Not implemented, m={}", m);
                        }
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
                self.regs.d[n] = replace_byte(self.regs.d[n], value);
            },
            2 => {  // move.b xx, (An)
                self.write8(self.regs.a[n], value);
            },
            3 => {
                let adr = self.regs.a[n];
                self.write8(adr, value);
                self.regs.a[n] = adr + 1;
            },
            5 => {  // move.b xx, (123, An)
                let ofs = self.read16(self.regs.pc) as SWord;
                self.regs.pc += 2;
                self.write8((self.regs.a[n] as SLong + ofs as SLong) as Adr, value);
            },
            6 => {  // Memory Indirect Pre-indexed: move.b xx, (123, An, Dx)
                let extension = self.read16(self.regs.pc);
                self.regs.pc += 2;
                if (extension & 0x100) != 0 {
                    panic!("Not implemented, dst=6/{:04x}", extension);
                } else {
                    let ofs = extension as SByte as SLong;
                    let da = (extension & 0x8000) != 0;  // Displacement is address register?
                    let dr = ((extension >> 12) & 7) as usize;  // Displacement register.
                    let dl = (extension & 0x0800) != 0;  // Displacement long?
                    let regofs = if dl { (if da {self.regs.a[dr]} else {self.regs.d[dr]}) as SLong } else { (if da {self.regs.a[dr]} else {self.regs.d[dr]}) as SWord as SLong };
                    let adr = (ofs + (self.regs.a[n] as SLong) + regofs) as Long;
                    self.write8(adr, value);
                }
            },
            7 => {
                match n {
                    1 => {
                        let d = self.read32(self.regs.pc);
                        self.regs.pc += 4;
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
                self.regs.d[n] = replace_word(self.regs.d[n], value);
            },
            1 => {
                self.regs.a[n] = replace_word(self.regs.a[n], value);
            },
            2 => {  // move.w xx, (An)
                self.write16(self.regs.a[n], value);
            },
            3 => {
                let adr = self.regs.a[n];
                self.write16(adr, value);
                self.regs.a[n] = adr + 2;
            },
            4 => {
                let adr = self.regs.a[n] - 2;
                self.regs.a[n] = adr;
                self.write16(adr, value);
            },
            5 => {  // move.w xx, (123, An)
                let ofs = self.read16(self.regs.pc) as SWord;
                self.regs.pc += 2;
                self.write16((self.regs.a[n] as SLong + ofs as SLong) as Adr, value);
            },
            7 => {
                match n {
                    1 => {
                        let d = self.read32(self.regs.pc);
                        self.regs.pc += 4;
                        self.write16(d, value);
                    },
                    4 => {
                        self.regs.sr = value;
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
                self.regs.d[n] = value;
            },
            1 => {
                self.regs.a[n] = value;
            },
            2 => {  // move.l xx, (An)
                self.write32(self.regs.a[n], value);
            },
            3 => {
                let adr = self.regs.a[n];
                self.write32(adr, value);
                self.regs.a[n] = adr + 4;
            },
            4 => {
                let adr = self.regs.a[n] - 4;
                self.regs.a[n] = adr;
                self.write32(adr, value);
            },
            5 => {  // move.l xx, (123, An)
                let ofs = self.read16(self.regs.pc) as SWord;
                self.regs.pc += 2;
                self.write32((self.regs.a[n] as SLong + ofs as SLong) as Adr, value);
            },
            7 => {
                match n {
                    1 => {
                        let d = self.read32(self.regs.pc);
                        self.regs.pc += 4;
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

    fn set_cmp_sr(&mut self, borrow: bool, eq: bool, overflow: bool, neg: bool) {
        let mut ccr = 0;
        if borrow   { ccr |= FLAG_C; }
        if eq       { ccr |= FLAG_Z; }
        if overflow { ccr |= FLAG_V; }
        if neg      { ccr |= FLAG_N; }
        self.regs.sr = (self.regs.sr & !(FLAG_N | FLAG_Z | FLAG_V | FLAG_C)) | ccr;
    }

    fn set_and_sr(&mut self, zero: bool, neg: bool) {
        let mut ccr = 0;
        if zero { ccr |= FLAG_Z; }
        if neg  { ccr |= FLAG_N; }
        self.regs.sr = (self.regs.sr & !(FLAG_N | FLAG_Z | FLAG_V | FLAG_C)) | ccr;
    }

    fn set_tst_sr(&mut self, zero: bool, neg: bool) {
        let mut ccr = 0;
        if zero { ccr |= FLAG_Z; }
        if neg  { ccr |= FLAG_N; }
        self.regs.sr = (self.regs.sr & !(FLAG_V | FLAG_C | FLAG_Z | FLAG_N)) | ccr;
    }

    fn read8(&mut self, adr: Adr) -> Byte {
        self.bus.read8(adr)
    }

    fn read16(&mut self, adr: Adr) -> Word {
        self.bus.read16(adr)
    }

    fn read32(&mut self, adr: Adr) -> Long {
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

fn dump_mem<BusT: BusTrait>(bus: &mut BusT, adr: Adr, sz: usize, max: usize) -> String {
    let arr = (0..max).map(|i| {
        if i * 2 < sz {
            format!("{:04x}", bus.read16(adr + (i as u32) * 2))
        } else {
            String::from("    ")
        }
    });
    arr.collect::<Vec<String>>().join(" ")
}
