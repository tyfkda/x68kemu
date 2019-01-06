use super::super::types::{Word};

use lazy_static::lazy_static;

#[derive(Clone)]
pub(crate) enum Opcode {
    Unknown,
    MoveByte,            // move.b XX, YY
    MoveLong,            // move.l XX, YY
    MoveWord,            // move.w XX, YY
    Moveq,               // moveq #%d, D%d
    MovemFrom,           // movem Dx/Dy-Dz/Ai.., -(Am)
    MovemTo,             // movem (Am)+, Dx/Dy-Dz/Ai..
    MoveToSrIm,          // move #$xxxx, SR
    LeaDirect,           // lea $xxxxxxxx, Ax
    Clr,                 // clr xx
    CmpmByte,            // cmpm.b (Am)+, (An)+
    TstWord,             // tst.w xx
    Reset,               // reset
    AddLong,             // add.l XX, Dd
    AddaLong,            // adda.l XX, Ad
    SubaLong,            // suba.l As, Ad
    AndLong,             // and.l XX, Dd
    BranchCond,          // bxx $xxxx
    Dbra,                // dbra $xxxx
    Bsr,                 // bsr $xxxx
    JsrA,                // jsr (Ax) or jsr ($ooo, Ax)
    Rts,                 // rts
    Trap,                // trap #x
}

#[derive(Clone)]
pub(crate) struct Inst {
    pub(crate) op: Opcode,
}

fn mask_inst(m: &mut Vec<&Inst>, mask: Word, value: Word, inst: &'static Inst) {
    let mut shift = mask;
    let mut masked: Vec<usize> = vec!();
    // Find masked bits.
    for i in 0..16 {
        if (shift & 1) == 0 {
            masked.push(i);
        }
        shift >>= 1;
    }

    for i in 0..(1 << masked.len()) {
        let mut opcode = value;
        for j in 0..masked.len() {
            opcode |= ((i >> j) & 1) << masked[j];
        }
        m[opcode as usize] = inst;
    }
}

lazy_static! {
    pub(crate) static ref INST: Vec<&'static Inst> = {
        let mut m = vec![&Inst {op: Opcode::Unknown}; 0x10000];
        mask_inst(&mut m, 0xf000, 0x1000, &Inst {op: Opcode::MoveByte});  // 1000-1fff
        mask_inst(&mut m, 0xf000, 0x2000, &Inst {op: Opcode::MoveLong});  // 2000-2fff
        mask_inst(&mut m, 0xf000, 0x3000, &Inst {op: Opcode::MoveWord});  // 3000-3fff
        mask_inst(&mut m, 0xf1ff, 0x41f9, &Inst {op: Opcode::LeaDirect});  // 41f9, 43f9, ..., 4ff9
        m[0x46fc] = &Inst {op: Opcode::MoveToSrIm};
        m[0x4e70] = &Inst {op: Opcode::Reset};
        m[0x4e75] = &Inst {op: Opcode::Rts};
        mask_inst(&mut m, 0xffc0, 0x4240, &Inst {op: Opcode::Clr});  // 4240-427f
        mask_inst(&mut m, 0xffc0, 0x4280, &Inst {op: Opcode::Clr});  // 4280-42bf
        mask_inst(&mut m, 0xfff8, 0x48e0, &Inst {op: Opcode::MovemFrom});  // 48e0-48e7
        mask_inst(&mut m, 0xffc0, 0x4a40, &Inst {op: Opcode::TstWord});  // 4a40-4a7f
        mask_inst(&mut m, 0xfff8, 0x4cd8, &Inst {op: Opcode::MovemTo});  // 4cd8-4cdf
        mask_inst(&mut m, 0xfff0, 0x4e40, &Inst {op: Opcode::Trap});  // 4e40-4e4f
        mask_inst(&mut m, 0xfff8, 0x51c8, &Inst {op: Opcode::Dbra});  // 51c8-51cf
        mask_inst(&mut m, 0xff00, 0x6100, &Inst {op: Opcode::Bsr});  // 6100-61ff
        mask_inst(&mut m, 0xfff0, 0x4e90, &Inst {op: Opcode::JsrA});  // 4e90-4e9f
        mask_inst(&mut m, 0xff00, 0x6600, &Inst {op: Opcode::BranchCond});  // 6600-66ff: bne
        mask_inst(&mut m, 0xff00, 0x6700, &Inst {op: Opcode::BranchCond});  // 6700-67ff: beq
        mask_inst(&mut m, 0xf100, 0x7000, &Inst {op: Opcode::Moveq});  // 7000...70ff, 7200...72ff, ..., 7eff
        mask_inst(&mut m, 0xf1c0, 0x91c0, &Inst {op: Opcode::SubaLong});  // 91c0, 91c1, 93c0, ..., 9fff
        mask_inst(&mut m, 0xf1f8, 0xb108, &Inst {op: Opcode::CmpmByte});  // b108, b109, b308, ..., bf0f
        mask_inst(&mut m, 0xf1c0, 0xc080, &Inst {op: Opcode::AndLong});  // c080, c081, c280, ..., cebf
        mask_inst(&mut m, 0xf1c0, 0xd080, &Inst {op: Opcode::AddLong});  // d080, d081, d280, ..., de87
        mask_inst(&mut m, 0xf1c0, 0xd1c0, &Inst {op: Opcode::AddaLong});  // d1c8, d1c9, d3c8, ..., dfff
        m
    };
}