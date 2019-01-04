use super::cpu::{Cpu};
use super::opcode::{Opcode, INST};
use super::types::{Word, Adr};

pub(crate) fn disasm(cpu: &Cpu, adr: Adr) -> (usize, String) {
    let op = cpu.read16(adr);
    let inst = &INST[op as usize];

    match inst.op {
        Opcode::MoveLong => {
            let n = (op >> 9) & 7;
            let m = op & 7;
            let dt = ((op >> 6) & 7) as usize;
            let (ssz, sstr) = disasm_read_source32(cpu, adr + 2, ((op >> 3) & 7) as usize, m);
            let (dsz, dstr) = disasm_write_destination32(cpu, adr + 2 + ssz, dt, n);
            ((2 + ssz + dsz) as usize, format!("move.l {}, {}", sstr, dstr))
        },
        Opcode::MoveWord => {
            let n = (op >> 9) & 7;
            let m = op & 7;
            let dt = ((op >> 6) & 7) as usize;
            let (ssz, sstr) = disasm_read_source16(cpu, adr + 2, ((op >> 3) & 7) as usize, m);
            let (dsz, dstr) = disasm_write_destination16(cpu, adr + 2 + ssz, dt, n);
            ((2 + ssz + dsz) as usize, format!("move.w {}, {}", sstr, dstr))
        },
        Opcode::Moveq => {
            let di = (op >> 9) & 7;
            let v = op & 0xff;
            let val = if v < 0x80 { v as i16 } else { -256 + v as i16 };
            //d[di].l = val;
            (2, format!("moveq #{}, D{}", val, di))
        },
        Opcode::MoveToSrIm => {
            let sr = cpu.read16(adr + 2);
            (4, format!("move #${:04x}, SR", sr))
        },
        Opcode::LeaDirect => {
            let di = ((op >> 9) & 7) as usize;
            let value = cpu.read32(adr + 2);
            (6, format!("lea ${:08x}.l, A{:?}", value, di))
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
            let si = (op & 7) as usize;
            (2, format!("add.l D{}, D{}", si, di))
        },
        Opcode::SubaLong => {
            let di = ((op >> 9) & 7) as usize;
            let si = (op & 7) as usize;
            (2, format!("suba.l A{}, A{}", si, di))
        },
        Opcode::Dbra => {
            let si = op & 7;
            let ofs = cpu.read16(adr + 2) as i16;
            (4, format!("dbra D{}, {:06x}", si, (adr + 2).wrapping_add((ofs as i32) as u32)))
        },
        Opcode::Bsr => {
            let mut ofs = ((op & 0x00ff) as i8) as i16;
            let mut sz = 0;
            if ofs == 0 {
                ofs = cpu.read16(adr + 2) as i16;
                sz = 2;
            }
            let jmp = ((adr + 2) as i32 + ofs as i32) as u32;
            (2 + sz, format!("bsr ${:06x}", jmp))
        },
        Opcode::Rts => {
            (2, String::from("rts"))
        },
        _ => {
            eprintln!("{:06x}: {:04x}  ; Unknown opcode", adr, op);
            panic!("Not implemented");
        },
    }
}

fn disasm_read_source16(cpu: &Cpu, adr: Adr,  src: usize, m: Word) -> (u32, String) {
    match src {
        7 => {  // Misc.
            match m {
                4 => {  // move.w #$XXXX, xx
                    let value = cpu.read16(adr);
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

fn disasm_read_source32(cpu: &Cpu, adr: Adr,  src: usize, m: Word) -> (u32, String) {
    match src {
        0 => {  // move.l Dm, xx
            (0, format!("D{}", m))
        },
        3 => {  // move.l (Am)+, xx
            (0, format!("(A{})+", m))
        },
        7 => {  // Misc.
            match m {
                4 => {  // move.l #$XXXX, xx
                    let value = cpu.read32(adr);
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

fn disasm_write_destination16(_cpu: &Cpu, _adr: Adr, dst: usize, n: Word) -> (u32, String) {
    match dst {
        0 => {
            (0, format!("D{}", n))
        },
        _ => {
            panic!("Not implemented, dst={}", dst);
        },
    }
}

fn disasm_write_destination32(cpu: &Cpu, adr: Adr, dst: usize, n: Word) -> (u32, String) {
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
                    let d = cpu.read32(adr);
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
