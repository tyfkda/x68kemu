use super::bus_trait::{BusTrait};
use super::cpu::{get_branch_offset};
use super::opcode::{Opcode, INST};
use super::super::types::{Word, SWord, Adr};

const DREG_NAMES: [&str; 8] = ["D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7"];
const AREG_NAMES: [&str; 8] = ["A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7"];
const AINDIRECT_NAMES: [&str; 8] = ["(A0)", "(A1)", "(A2)", "(A3)", "(A4)", "(A5)", "(A6)", "(A7)"];
const APOSTINC_NAMES: [&str; 8] = ["(A0)+", "(A1)+", "(A2)+", "(A3)+", "(A4)+", "(A5)+", "(A6)+", "(A7)+"];
const APREDEC_NAMES: [&str; 8] = ["-(A0)", "-(A1)", "-(A2)", "-(A3)", "-(A4)", "-(A5)", "-(A6)", "-(A7)"];
const BCOND_NAMES: [&str; 2] = ["bne", "beq"];

const MOVEM_NAMES: [&str; 16] = ["D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7"];

fn dreg(no: Word) -> String { DREG_NAMES[no as usize].to_string() }
fn areg(no: Word) -> String { AREG_NAMES[no as usize].to_string() }
fn aindname(no: Word) -> String { AINDIRECT_NAMES[no as usize].to_string() }
fn apostinc(no: Word) -> String { APOSTINC_NAMES[no as usize].to_string() }
fn apredec(no: Word) -> String { APREDEC_NAMES[no as usize].to_string() }
fn bcond(no: Word) -> String { BCOND_NAMES[no as usize].to_string() }

pub(crate) fn disasm<BusT: BusTrait>(bus: &BusT, adr: Adr) -> (usize, String) {
    let op = bus.read16(adr);
    let inst = &INST[op as usize];

    match inst.op {
        Opcode::MoveByte => {
            let n = (op >> 9) & 7;
            let m = op & 7;
            let dt = ((op >> 6) & 7) as usize;
            let (ssz, sstr) = read_source8(bus, adr + 2, ((op >> 3) & 7) as usize, m);
            let (dsz, dstr) = write_destination8(bus, adr + 2 + ssz, dt, n);
            ((2 + ssz + dsz) as usize, format!("move.b {}, {}", sstr, dstr))
        },
        Opcode::MoveLong => {
            let n = (op >> 9) & 7;
            let m = op & 7;
            let dt = ((op >> 6) & 7) as usize;
            let (ssz, sstr) = read_source32(bus, adr + 2, ((op >> 3) & 7) as usize, m);
            let (dsz, dstr) = write_destination32(bus, adr + 2 + ssz, dt, n);
            ((2 + ssz + dsz) as usize, format!("move.l {}, {}", sstr, dstr))  // TODO: Use movea for a-regs.
        },
        Opcode::MoveWord => {
            let n = (op >> 9) & 7;
            let m = op & 7;
            let dt = ((op >> 6) & 7) as usize;
            let (ssz, sstr) = read_source16(bus, adr + 2, ((op >> 3) & 7) as usize, m);
            let (dsz, dstr) = write_destination16(bus, adr + 2 + ssz, dt, n);
            ((2 + ssz + dsz) as usize, format!("move.w {}, {}", sstr, dstr))
        },
        Opcode::Moveq => {
            let di = (op >> 9) & 7;
            let v = op & 0xff;
            let val = if v < 0x80 { v as i16 } else { -256 + v as i16 };
            (2, format!("moveq #{}, {}", val, dreg(di)))
        },
        Opcode::MovemFrom => {
            let di = op & 7;
            let bits = bus.read16(adr + 2);
            let regs = (0..16)
                .filter(|i| { (bits & ((0x8000 as u16) >> i)) != 0 })
                .map(|i| { MOVEM_NAMES[i] })
                .collect::<Vec<&str>>().join("/");
            (4, format!("movem.l {}, {}", regs, apredec(di)))
        },
        Opcode::MovemTo => {
            let di = op & 7;
            let bits = bus.read16(adr + 2);
            let regs = (0..16)
                .filter(|i| { (bits & ((1 as u16) << i)) != 0 })
                .map(|i| { MOVEM_NAMES[i] })
                .collect::<Vec<&str>>().join("/");
            (4, format!("movem.l {}, {}", apostinc(di), regs))
        },
        Opcode::MoveToSrIm => {
            let sr = bus.read16(adr + 2);
            (4, format!("move #${:04x}, SR", sr))
        },
        Opcode::LeaDirect => {
            let di = (op >> 9) & 7;
            let value = bus.read32(adr + 2);
            (6, format!("lea ${:08x}.l, {}", value, areg(di)))
        },
        Opcode::Clr => {
            let dt = ((op >> 3) & 7) as usize;
            let n = op & 7;
            match op & 0xffc0 {
                0x4200 => {  // byte
                    let (dsz, dstr) = write_destination16(bus, adr + 2, dt, n);
                    ((2 + dsz) as usize, format!("clr.b {}", dstr))
                },
                0x4240 => {  // word
                    let (dsz, dstr) = write_destination16(bus, adr + 2, dt, n);
                    ((2 + dsz) as usize, format!("clr.w {}", dstr))
                },
                0x4280 => {  // long
                    let (dsz, dstr) = write_destination16(bus, adr + 2, dt, n);
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
            (2, format!("cmpm.b {}, {}", apostinc(si), apostinc(di)))
        },
        Opcode::TstWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("tst.w {}", sstr))
        },
        Opcode::Reset => {
            (2, "reset".to_string())
        },
        Opcode::AddLong => {
            let di = (op >> 9) & 7;
            let st = ((op >> 3) & 7) as usize;
            let si = op & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("add.l {}, {}", sstr, dreg(di)))
        },
        Opcode::AddaLong => {
            let di = (op >> 9) & 7;
            let st = ((op >> 3) & 7) as usize;
            let si = op & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("adda.l {}, {}", sstr, areg(di)))
        },
        Opcode::SubaLong => {
            let di = (op >> 9) & 7;
            let st = ((op >> 3) & 7) as usize;
            let si = op & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("suba.l {}, {}", sstr, areg(di)))
        },
        Opcode::AndLong => {
            let n = (op >> 9) & 7;
            let m = op & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, ((op >> 3) & 7) as usize, m);
            ((2 + ssz) as usize, format!("and.l {}, {}", sstr, dreg(n)))
        },
        Opcode::BranchCond => {
            let (ofs, sz) = get_branch_offset(op, bus, adr + 2);
            let jmp = ((adr + 2) as i32 + ofs as i32) as u32;
            let bt = (op >> 8) - 0x66;
            ((2 + sz) as usize, format!("{} ${:06x}", bcond(bt), jmp))
        },
        Opcode::Dbra => {
            let si = op & 7;
            let ofs = bus.read16(adr + 2) as i16;
            (4, format!("dbra D{}, ${:06x}", si, (adr + 2).wrapping_add((ofs as i32) as u32)))
        },
        Opcode::Bsr => {
            let (ofs, sz) = get_branch_offset(op, bus, adr + 2);
            let jmp = ((adr + 2) as i32 + ofs as i32) as u32;
            ((2 + sz) as usize, format!("bsr ${:06x}", jmp))
        },
        Opcode::JsrA => {
            let di = (op & 7) as usize;
            if (op & 15) < 8 {
                (2, format!("jsr (A{})", di))
            } else {
                let offset = bus.read16(adr + 2);
                (4, format!("jsr (${:04x}, A{})", offset, di))
            }
        },
        Opcode::Rts => {
            (2, String::from("rts"))
        },
        Opcode::Trap => {
            let no = op & 0x000f;
            (2, format!("trap #{}", no))
        },
        _ => {
            (2, format!("**{:04x}** Unknown opcode", op))
        },
    }
}

fn read_source8<BusT: BusTrait>(bus: &BusT, adr: Adr,  src: usize, m: Word) -> (u32, String) {
    match src {
        0 => {  // move.b Dm, xx
            (0, dreg(m))
        },
        3 => {  // move.b (Am)+, xx
            (0, apostinc(m))
        },
        7 => {  // Misc.
            match m {
                1 => {  // move.b $XXXXXXXX.l, xx
                    let adr = bus.read32(adr);
                    (4, format!("${:08x}", adr))
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

fn read_source16<BusT: BusTrait>(bus: &BusT, adr: Adr,  src: usize, m: Word) -> (u32, String) {
    match src {
        0 => {  // move.w Dm, xx
            (0, dreg(m))
        },
        5 => {  // move.l (123, An), xx
            let ofs = bus.read16(adr) as SWord;
            (2, format!("({}, {})", ofs, areg(m)))
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

fn read_source32<BusT: BusTrait>(bus: &BusT, adr: Adr,  src: usize, m: Word) -> (u32, String) {
    match src {
        0 => {  // move.l Dm, xx
            (0, dreg(m))
        },
        1 => {  // move.l Am, xx
            (0, areg(m))
        },
        2 => {  // move.l (Am), xx
            (0, aindname(m))
        },
        3 => {  // move.l (Am)+, xx
            (0, apostinc(m))
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

fn write_destination8<BusT: BusTrait>(bus: &BusT, adr: Adr, dst: usize, n: Word) -> (u32, String) {
    match dst {
        0 => {
            (0, dreg(n))
        },
        3 => {
            (0, apostinc(n))
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

fn write_destination16<BusT: BusTrait>(bus: &BusT, adr: Adr, dst: usize, n: Word) -> (u32, String) {
    match dst {
        0 => {
            (0, dreg(n))
        },
        1 => {  // move.w xx, An
            (0, areg(n))
        },
        3 => {
            (0, apostinc(n))
        },
        5 => {  // move.l xx, (123, An)
            let ofs = bus.read16(adr) as SWord;
            (2, format!("({}, {})", ofs, areg(n)))
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

fn write_destination32<BusT: BusTrait>(bus: &BusT, adr: Adr, dst: usize, n: Word) -> (u32, String) {
    match dst {
        0 => {
            (0, dreg(n))
        },
        1 => {  // move.l xx, An
            (0, areg(n))
        },
        3 => {
            (0, apostinc(n))
        },
        5 => {  // move.l xx, (123, An)
            let ofs = bus.read16(adr) as SWord;
            (2, format!("({}, {})", ofs, areg(n)))
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
