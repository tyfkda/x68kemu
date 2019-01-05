use super::bus::{Bus};
use super::cpu::{get_branch_offset};
use super::opcode::{Opcode, INST};
use super::types::{Word, Adr};

pub(crate) fn disasm(bus: &Bus, adr: Adr) -> (usize, String) {
    let op = bus.read16(adr);
    let inst = &INST[op as usize];

    match inst.op {
        Opcode::MoveByte => {
            let n = (op >> 9) & 7;
            let m = op & 7;
            let dt = ((op >> 6) & 7) as usize;
            let (ssz, sstr) = disasm_read_source8(bus, adr + 2, ((op >> 3) & 7) as usize, m);
            let (dsz, dstr) = disasm_write_destination8(bus, adr + 2 + ssz, dt, n);
            ((2 + ssz + dsz) as usize, format!("move.b {}, {}", sstr, dstr))
        },
        Opcode::MoveLong => {
            let n = (op >> 9) & 7;
            let m = op & 7;
            let dt = ((op >> 6) & 7) as usize;
            let (ssz, sstr) = disasm_read_source32(bus, adr + 2, ((op >> 3) & 7) as usize, m);
            let (dsz, dstr) = disasm_write_destination32(bus, adr + 2 + ssz, dt, n);
            ((2 + ssz + dsz) as usize, format!("move.l {}, {}", sstr, dstr))  // TODO: Use movea for a-regs.
        },
        Opcode::MoveWord => {
            let n = (op >> 9) & 7;
            let m = op & 7;
            let dt = ((op >> 6) & 7) as usize;
            let (ssz, sstr) = disasm_read_source16(bus, adr + 2, ((op >> 3) & 7) as usize, m);
            let (dsz, dstr) = disasm_write_destination16(bus, adr + 2 + ssz, dt, n);
            ((2 + ssz + dsz) as usize, format!("move.w {}, {}", sstr, dstr))
        },
        Opcode::Moveq => {
            let di = (op >> 9) & 7;
            let v = op & 0xff;
            let val = if v < 0x80 { v as i16 } else { -256 + v as i16 };
            //d[di].l = val;
            (2, format!("moveq #{}, D{}", val, di))
        },
        Opcode::MovemFrom => {
            let di = op & 7;
            let bits = bus.read16(adr + 2);
            (4, format!("movem.l #{:04x}, -(A{})", bits, di))  // TODO: Print registers.
        },
        Opcode::MovemTo => {
            let di = op & 7;
            let bits = bus.read16(adr + 2);
            (4, format!("movem.l (A{})+, #{:04x}", bits, di))  // TODO: Print registers.
        },
        Opcode::MoveToSrIm => {
            let sr = bus.read16(adr + 2);
            (4, format!("move #${:04x}, SR", sr))
        },
        Opcode::LeaDirect => {
            let di = ((op >> 9) & 7) as usize;
            let value = bus.read32(adr + 2);
            (6, format!("lea ${:08x}.l, A{:?}", value, di))
        },
        Opcode::Clr => {
            let dt = ((op >> 3) & 7) as usize;
            let n = op & 7;
            match op & 0xffc0 {
                0x4200 => {  // byte
                    let (dsz, dstr) = disasm_write_destination16(bus, adr + 2, dt, n);
                    ((2 + dsz) as usize, format!("clr.b {}", dstr))
                },
                0x4240 => {  // word
                    let (dsz, dstr) = disasm_write_destination16(bus, adr + 2, dt, n);
                    ((2 + dsz) as usize, format!("clr.w {}", dstr))
                },
                0x4280 => {  // long
                    let (dsz, dstr) = disasm_write_destination16(bus, adr + 2, dt, n);
                    ((2 + dsz) as usize, format!("clr.l {}", dstr))
                },
                _ => {
                    panic!("Must not happen");
                },
            }
        },
        Opcode::CmpmByte => {
            let si = op & 7;
            let di = (op >> 9) & 7;
            (2, format!("cmpm.b (A{})+, (A{})+", si, di))
        },
        Opcode::Reset => {
            (2, "reset".to_string())
        },
        Opcode::AddLong => {
            let di = ((op >> 9) & 7) as usize;
            let st = ((op >> 3) & 7) as usize;
            let si = op & 7;
            let (ssz, sstr) = disasm_read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("add.l {}, D{}", sstr, di))
        },
        Opcode::AddaLong => {
            let di = ((op >> 9) & 7) as usize;
            let st = ((op >> 3) & 7) as usize;
            let si = op & 7;
            let (ssz, sstr) = disasm_read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("adda.l {}, A{}", sstr, di))
        },
        Opcode::SubaLong => {
            let di = ((op >> 9) & 7) as usize;
            let st = ((op >> 3) & 7) as usize;
            let si = op & 7;
            let (ssz, sstr) = disasm_read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("suba.l {}, A{}", sstr, di))
        },
        Opcode::AndLong => {
            let n = (op >> 9) & 7;
            let m = op & 7;
            let (ssz, sstr) = disasm_read_source32(bus, adr + 2, ((op >> 3) & 7) as usize, m);
            ((2 + ssz) as usize, format!("and.l {}, D{}", sstr, n))
        },
        Opcode::BranchCond => {
            let (ofs, sz) = get_branch_offset(op, bus, adr + 2);
            let jmp = ((adr + 2) as i32 + ofs as i32) as u32;
            ((2 + sz) as usize, format!("bne ${:06x}", jmp))
        },
        Opcode::Dbra => {
            let si = op & 7;
            let ofs = bus.read16(adr + 2) as i16;
            (4, format!("dbra D{}, {:06x}", si, (adr + 2).wrapping_add((ofs as i32) as u32)))
        },
        Opcode::Bsr => {
            let (ofs, sz) = get_branch_offset(op, bus, adr + 2);
            let jmp = ((adr + 2) as i32 + ofs as i32) as u32;
            ((2 + sz) as usize, format!("bsr ${:06x}", jmp))
        },
        Opcode::Rts => {
            (2, String::from("rts"))
        },
        Opcode::Trap => {
            let no = op & 0x000f;
            (2, format!("trap #{}", no))
        },
        _ => {
            eprintln!("{:06x}: {:04x}  ; Unknown opcode", adr, op);
            panic!("Not implemented");
        },
    }
}

fn disasm_read_source8(bus: &Bus, adr: Adr,  src: usize, m: Word) -> (u32, String) {
    match src {
        0 => {  // move.b Dm, xx
            (0, format!("D{}", m))
        },
        3 => {  // move.b (Am)+, xx
            (0, format!("(A{})+", m))
        },
        7 => {  // Misc.
            match m {
                1 => {  // move.b $XXXXXXXX.l, xx
                    let adr = bus.read32(adr);
                    (4, format!("${:06x}", adr))
                },
                4 => {  // move.b #$XXXX, xx
                    let value = bus.read16(adr);
                    (2, format!("#${:04x}", value))
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

fn disasm_read_source16(bus: &Bus, adr: Adr,  src: usize, m: Word) -> (u32, String) {
    match src {
        0 => {  // move.w Dm, xx
            (0, format!("D{}", m))
        },
        7 => {  // Misc.
            match m {
                4 => {  // move.w #$XXXX, xx
                    let value = bus.read16(adr);
                    (2, format!("#${:04x}", value))
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

fn disasm_read_source32(bus: &Bus, adr: Adr,  src: usize, m: Word) -> (u32, String) {
    match src {
        0 => {  // move.l Dm, xx
            (0, format!("D{}", m))
        },
        1 => {  // move.l Am, xx
            (0, format!("A{}", m))
        },
        2 => {  // move.l (Am), xx
            (0, format!("(A{})", m))
        },
        3 => {  // move.l (Am)+, xx
            (0, format!("(A{})+", m))
        },
        7 => {  // Misc.
            match m {
                4 => {  // move.l #$XXXX, xx
                    let value = bus.read32(adr);
                    (4, format!("#${:08x}", value))
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

fn disasm_write_destination8(bus: &Bus, adr: Adr, dst: usize, n: Word) -> (u32, String) {
    match dst {
        0 => {
            (0, format!("D{}", n))
        },
        3 => {
            (0, format!("(A{})+", n))
        },
        7 => {
            match n {
                1 => {
                    let d = bus.read32(adr);
                    (4, format!("${:08x}", d))
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

fn disasm_write_destination16(bus: &Bus, adr: Adr, dst: usize, n: Word) -> (u32, String) {
    match dst {
        0 => {
            (0, format!("D{}", n))
        },
        1 => {  // move.w xx, An
            (0, format!("A{}", n))
        },
        3 => {
            (0, format!("(A{})+", n))
        },
        7 => {
            match n {
                1 => {
                    let d = bus.read32(adr);
                    (4, format!("${:08x}", d))
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

fn disasm_write_destination32(bus: &Bus, adr: Adr, dst: usize, n: Word) -> (u32, String) {
    match dst {
        0 => {
            (0, format!("D{}", n))
        },
        1 => {  // move.l xx, An
            (0, format!("A{}", n))
        },
        3 => {
            (0, format!("(A{})+", n))
        },
        7 => {
            match n {
                1 => {
                    let d = bus.read32(adr);
                    (4, format!("${:08x}", d))
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
