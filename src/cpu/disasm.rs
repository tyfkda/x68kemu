use super::bus_trait::{BusTrait};
use super::cpu::{get_branch_offset, conv07to18};
use super::opcode::{Opcode, INST};
use super::super::types::{Word, Long, SByte, SWord, SLong, Adr};

const DREG_NAMES: [&str; 8] = ["D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7"];
const AREG_NAMES: [&str; 8] = ["A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7"];
const AINDIRECT_NAMES: [&str; 8] = ["(A0)", "(A1)", "(A2)", "(A3)", "(A4)", "(A5)", "(A6)", "(A7)"];
const APOSTINC_NAMES: [&str; 8] = ["(A0)+", "(A1)+", "(A2)+", "(A3)+", "(A4)+", "(A5)+", "(A6)+", "(A7)+"];
const APREDEC_NAMES: [&str; 8] = ["-(A0)", "-(A1)", "-(A2)", "-(A3)", "-(A4)", "-(A5)", "-(A6)", "-(A7)"];

const MOVE_NAMES: [&str; 8] = ["move", "movea", "move", "move", "move", "move", "move", "move"];
const MOVEM_REG_NAMES: [&str; 16] = ["D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7"];

fn dreg(no: Word) -> String { DREG_NAMES[no as usize].to_string() }
fn areg(no: Word) -> String { AREG_NAMES[no as usize].to_string() }
fn aind(no: Word) -> String { AINDIRECT_NAMES[no as usize].to_string() }
fn apostinc(no: Word) -> String { APOSTINC_NAMES[no as usize].to_string() }
fn apredec(no: Word) -> String { APREDEC_NAMES[no as usize].to_string() }

pub(crate) fn disasm<BusT: BusTrait>(bus: &BusT, adr: Adr) -> (usize, String) {
    let op = bus.read16(adr);
    let inst = &INST[op as usize];

    match inst.op {
        Opcode::Nop => {
            (2, "nop".to_string())
        },
        Opcode::MoveByte => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let dt = ((op >> 6) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source8(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination8(bus, adr + 2 + ssz, dt, di);
            ((2 + ssz + dsz) as usize, format!("{}.b {}, {}", MOVE_NAMES[dt], sstr, dstr))
        },
        Opcode::MoveWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let dt = ((op >> 6) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination16(bus, adr + 2 + ssz, dt, di);
            ((2 + ssz + dsz) as usize, format!("{}.w {}, {}", MOVE_NAMES[dt], sstr, dstr))
        },
        Opcode::MoveLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let dt = ((op >> 6) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination32(bus, adr + 2 + ssz, dt, di);
            ((2 + ssz + dsz) as usize, format!("{}.l {}, {}", MOVE_NAMES[dt], sstr, dstr))
        },
        Opcode::Moveq => {
            let v = op & 0xff;
            let di = (op >> 9) & 7;
            let val = if v < 0x80 { v as SWord } else { -256 + v as SWord };
            (2, format!("moveq #{}, {}", val, dreg(di)))
        },
        Opcode::MovemFrom => {
            let di = op & 7;
            let bits = bus.read16(adr + 2);
            let regs = (0..16)
                .filter(|i| { (bits & ((0x8000 as u16) >> i)) != 0 })
                .map(|i| { MOVEM_REG_NAMES[i] })
                .collect::<Vec<&str>>().join("/");
            (4, format!("movem.l {}, {}", regs, apredec(di)))
        },
        Opcode::MovemTo => {
            let si = op & 7;
            let bits = bus.read16(adr + 2);
            let regs = (0..16)
                .filter(|i| { (bits & ((1 as u16) << i)) != 0 })
                .map(|i| { MOVEM_REG_NAMES[i] })
                .collect::<Vec<&str>>().join("/");
            (4, format!("movem.l {}, {}", apostinc(si), regs))
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
        Opcode::LeaOffset => {
            let si = op & 7;
            let di = (op >> 9) & 7;
            let ofs = bus.read16(adr + 2) as SWord;
            (4, format!("lea ({}, {}), {}", ofs, areg(si), areg(di)))
        },
        Opcode::LeaOffsetD => {
            let si = op & 7;
            let di = (op >> 9) & 7;
            let next = bus.read16(adr + 2);
            if (next & 0x8f00) == 0x0000 {
                let ofs = next as SByte;
                let ii = (next >> 12) & 0x07;
                (4, format!("lea ({}, {}, {}.w), {}", ofs, areg(si), dreg(ii), areg(di)))
            } else {
                (4, "**Not implemented**".to_string())
            }
        },
        Opcode::LeaOffsetPc => {
            let di = (op >> 9) & 7;
            let ofs = bus.read16(adr + 2) as SWord;
            (4, format!("lea ({}, PC), {}", ofs, areg(di)))
        },
        Opcode::Clr => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            match op & 0xffc0 {
                0x4200 => {  // byte
                    let (dsz, dstr) = write_destination16(bus, adr + 2, dt, di);
                    ((2 + dsz) as usize, format!("clr.b {}", dstr))
                },
                0x4240 => {  // word
                    let (dsz, dstr) = write_destination16(bus, adr + 2, dt, di);
                    ((2 + dsz) as usize, format!("clr.w {}", dstr))
                },
                0x4280 => {  // long
                    let (dsz, dstr) = write_destination16(bus, adr + 2, dt, di);
                    ((2 + dsz) as usize, format!("clr.l {}", dstr))
                },
                _ => {
                    panic!("Must not happen");
                },
            }
        },
        Opcode::Swap => {
            let di = op & 7;
            (2, format!("swap {}", dreg(di)))
        },
        Opcode::CmpByte => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source8(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination8(bus, adr + 2 + ssz, 0, di);
            ((2 + ssz + dsz) as usize, format!("cmp.b {}, {}", sstr, dstr))
        },
        Opcode::CmpWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination16(bus, adr + 2 + ssz, 0, di);
            ((2 + ssz + dsz) as usize, format!("cmp.w {}, {}", sstr, dstr))
        },
        Opcode::CmpaLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination32(bus, adr + 2 + ssz, 1, di);
            ((2 + ssz + dsz) as usize, format!("cmpa.l {}, {}", sstr, dstr))
        },
        Opcode::CmpmByte => {
            let si = op & 7;
            let di = (op >> 9) & 7;
            (2, format!("cmpm.b {}, {}", apostinc(si), apostinc(di)))
        },
        Opcode::TstByte => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let (ssz, sstr) = read_source8(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("tst.b {}", sstr))
        },
        Opcode::TstWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("tst.w {}", sstr))
        },
        Opcode::TstLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("tst.l {}", sstr))
        },
        Opcode::Reset => {
            (2, "reset".to_string())
        },
        Opcode::AddLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("add.l {}, {}", sstr, dreg(di)))
        },
        Opcode::AddaLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("adda.l {}, {}", sstr, areg(di)))
        },
        Opcode::AddqLong => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = conv07to18(op >> 9);
            let (dsz, dstr) = write_destination32(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("addq.l #{}, {}", v, dstr))
        },
        Opcode::SubaLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("suba.l {}, {}", sstr, areg(di)))
        },
        Opcode::SubqWord => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = conv07to18(op >> 9);
            let (dsz, dstr) = write_destination16(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("subq.w #{}, {}", v, dstr))
        },
        Opcode::AndWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("and.w {}, {}", sstr, dreg(di)))
        },
        Opcode::AndLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("and.l {}, {}", sstr, dreg(di)))
        },
        Opcode::AndiWord => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = bus.read16(adr + 2);
            let (dsz, dstr) = write_destination16(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("andi.w #{:04x}, {}", v, dstr))
        },
        Opcode::AslImWord => {
            let di = op & 7;
            let shift = conv07to18(op >> 9);
            (2, format!("asl.w #{}, {}", shift, dreg(di)))
        },
        Opcode::AslImLong => {
            let di = op & 7;
            let shift = conv07to18(op >> 9);
            (2, format!("asl.l #{}, {}", shift, dreg(di)))
        },
        Opcode::LsrImWord => {
            let di = op & 7;
            let shift = conv07to18(op >> 9);
            (2, format!("lsr.w #{}, {}", shift, dreg(di)))
        },
        Opcode::RorWord => {
            let di = op & 7;
            let si = conv07to18(op >> 9);
            (2, format!("ror.w #{}, {}", si, dreg(di)))
        },
        Opcode::RolByte => {
            let di = op & 7;
            let si = conv07to18(op >> 9);
            (2, format!("rol.b #{}, {}", si, dreg(di)))
        },
        Opcode::Bra => { bcond(bus, adr + 2, op, "bra") },
        Opcode::Bcc => { bcond(bus, adr + 2, op, "bcc") },
        Opcode::Bcs => { bcond(bus, adr + 2, op, "bcs") },
        Opcode::Bne => { bcond(bus, adr + 2, op, "bne") },
        Opcode::Beq => { bcond(bus, adr + 2, op, "beq") },
        Opcode::Bge => { bcond(bus, adr + 2, op, "bge") },
        Opcode::Blt => { bcond(bus, adr + 2, op, "blt") },
        Opcode::Bgt => { bcond(bus, adr + 2, op, "bgt") },
        Opcode::Ble => { bcond(bus, adr + 2, op, "ble") },
        Opcode::Dbra => {
            let si = op & 7;
            let ofs = bus.read16(adr + 2) as SWord;
            let jmp = ((adr + 2) as SLong).wrapping_add(ofs as SLong) as Long;
            (4, format!("dbra {}, ${:06x}", dreg(si), jmp))
        },
        Opcode::Bsr => {
            let (ofs, sz) = get_branch_offset(op, bus, adr + 2);
            let jmp = ((adr + 2) as SLong + ofs) as Long;
            ((2 + sz) as usize, format!("bsr ${:06x}", jmp))
        },
        Opcode::JsrA => {
            let si = op & 7;
            if (op & 15) < 8 {
                (2, format!("jsr ({})", areg(si)))
            } else {
                let offset = bus.read16(adr + 2);
                (4, format!("jsr (${:04x}, {})", offset, areg(si)))
            }
        },
        Opcode::Rts => {
            (2, "rts".to_string())
        },
        Opcode::Rte => {
            (2, "rte".to_string())
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

fn bcond<BusT: BusTrait>(bus: &BusT, adr: Adr, op: Word, bname: &str) -> (usize, String) {
    let (ofs, sz) = get_branch_offset(op, bus, adr);
    let jmp = (adr as SLong).wrapping_add(ofs) as Long;
    ((2 + sz) as usize, format!("{} ${:06x}", bname, jmp))
}

fn read_source8<BusT: BusTrait>(bus: &BusT, adr: Adr,  src: usize, m: Word) -> (u32, String) {
    match src {
        0 => {  // move.b Dm, xx
            (0, dreg(m))
        },
        2 => {  // move.b (Am), xx
            (0, aind(m))
        },
        3 => {  // move.b (Am)+, xx
            (0, apostinc(m))
        },
        5 => {  // move.b (123, An), xx
            let ofs = bus.read16(adr) as SWord;
            (2, format!("({}, {})", ofs, areg(m)))
        },
        7 => {  // Misc.
            match m {
                1 => {  // move.b $XXXXXXXX.l, xx
                    let adr = bus.read32(adr);
                    (4, format!("${:08x}", adr))
                },
                4 => {  // move.b #$XXXX, xx
                    let value = bus.read16(adr);
                    (2, format!("#${:02x}", value & 0x00ff))
                },
                _ => {
                    (0, format!("UnhandledSrc(7/{})", m))
                },
            }
        },
        _ => {
            (0, format!("UnhandledSrc({})", src))
        },
    }
}

fn read_source16<BusT: BusTrait>(bus: &BusT, adr: Adr,  src: usize, m: Word) -> (u32, String) {
    match src {
        0 => {  // move.w Dm, xx
            (0, dreg(m))
        },
        2 => {  // move.w (Am), xx
            (0, aind(m))
        },
        3 => {  // move.w (Am)+, xx
            (0, apostinc(m))
        },
        5 => {  // move.w (123, An), xx
            let ofs = bus.read16(adr) as SWord;
            (2, format!("({}, {})", ofs, areg(m)))
        },
        7 => {  // Misc.
            match m {
                1 => {  // move.b $XXXXXXXX.l, xx
                    let adr = bus.read32(adr);
                    (4, format!("${:08x}", adr))
                },
                4 => {  // move.w #$XXXX, xx
                    let value = bus.read16(adr);
                    (2, format!("#${:04x}", value))
                },
                _ => {
                    (0, format!("UnhandledSrc(7/{})", m))
                },
            }
        },
        _ => {
            (0, format!("UnhandledSrc({})", src))
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
            (0, aind(m))
        },
        3 => {  // move.l (Am)+, xx
            (0, apostinc(m))
        },
        5 => {  // move.l (123,Am), xx
            let ofs = bus.read16(adr) as SWord;
            (2, format!("({}, {})", ofs, areg(m)))
        },
        7 => {  // Misc.
            match m {
                1 => {  // move.b $XXXXXXXX.l, xx
                    let adr = bus.read32(adr);
                    (4, format!("${:08x}", adr))
                },
                4 => {  // move.l #$XXXX, xx
                    let value = bus.read32(adr);
                    (4, format!("#${:08x}", value))
                },
                _ => {
                    (0, format!("UnhandledSrc(7/{})", m))
                },
            }
        },
        _ => {
            (0, format!("UnhandledSrc({})", src))
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
        5 => {  // move.b xx, (123, An)
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
                    (0, format!("UnhandledDst(7/{})", n))
                },
            }
        },
        _ => {
            (0, format!("UnhandledDst({})", dst))
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
        2 => {  // move.w xx, (An)
            (0, aind(n))
        },
        3 => {
            (0, apostinc(n))
        },
        4 => {
            (0, apredec(n))
        },
        5 => {  // move.w xx, (123, An)
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
                    (0, format!("UnhandledDst(7/{})", n))
                },
            }
        },
        _ => {
            (0, format!("UnhandledDst({})", dst))
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
        2 => {  // move.l xx, (An)
            (0, aind(n))
        },
        3 => {
            (0, apostinc(n))
        },
        4 => {
            (0, apredec(n))
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
                    (0, format!("UnhandledDst(7/{})", n))
                },
            }
        },
        _ => {
            (0, format!("UnhandledDst({})", dst))
        },
    }
}
