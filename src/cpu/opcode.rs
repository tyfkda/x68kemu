use super::super::types::{Word};

use lazy_static::lazy_static;

#[derive(Clone)]
pub(crate) enum Opcode {
    Unknown,
    Nop,                 // nop
    MoveByte,            // move.b XX, YY
    MoveLong,            // move.l XX, YY
    MoveWord,            // move.w XX, YY
    Moveq,               // moveq #%d, D%d
    MovemFrom,           // movem Dx/Dy-Dz/Ai.., -(Am)
    MovemTo,             // movem (Am)+, Dx/Dy-Dz/Ai..
    MoveToSrIm,          // move #$xxxx, SR
    LeaDirect,           // lea $xxxxxxxx, Ax
    LeaOffset,           // lea (xx, As), Ad
    LeaOffsetPc,         // lea (xx, PC), Ad
    Clr,                 // clr xx
    CmpByte,             // cmp.b XX, YY
    CmpWord,             // cmp.w XX, YY
    CmpaLong,            // cmpa.l XX, Ad
    CmpmByte,            // cmpm.b (Am)+, (An)+
    TstByte,             // tst.b xx
    TstWord,             // tst.w xx
    TstLong,             // tst.l xx
    Reset,               // reset
    AddLong,             // add.l XX, Dd
    AddaLong,            // adda.l XX, Ad
    AddqLong,            // addq.l #%d, D%d
    SubaLong,            // suba.l As, Ad
    SubqWord,            // subq.w #%d, D%d
    AndLong,             // and.l XX, Dd
    Bcc,                 // bcc $xxxx
    Bcs,                 // bcs $xxxx
    Bne,                 // bne $xxxx
    Beq,                 // beq $xxxx
    Dbra,                // dbra $xxxx
    Bsr,                 // bsr $xxxx
    JsrA,                // jsr (Ax) or jsr ($ooo, Ax)
    Rts,                 // rts
    Rte,                 // rte
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

fn range_inst(m: &mut Vec<&Inst>, range: &mut std::ops::Range<Word>, inst: &'static Inst) {
    for op in range {
        m[op as usize] = inst;
    }
}

lazy_static! {
    pub(crate) static ref INST: Vec<&'static Inst> = {
        let mut m = vec![&Inst {op: Opcode::Unknown}; 0x10000];
        mask_inst(&mut m, 0xf000, 0x1000, &Inst {op: Opcode::MoveByte});  // 1000-1fff
        mask_inst(&mut m, 0xf000, 0x2000, &Inst {op: Opcode::MoveLong});  // 2000-2fff
        mask_inst(&mut m, 0xf000, 0x3000, &Inst {op: Opcode::MoveWord});  // 3000-3fff
        mask_inst(&mut m, 0xf1f8, 0x41e8, &Inst {op: Opcode::LeaOffset});  // 41e8, 41e9, 43e8, ..., 4fef
        mask_inst(&mut m, 0xf1ff, 0x41f9, &Inst {op: Opcode::LeaDirect});  // 41f9, 43f9, ..., 4ff9
        mask_inst(&mut m, 0xf1ff, 0x41fa, &Inst {op: Opcode::LeaOffsetPc});  // 41fa, 43fa, ..., 4ffa
        m[0x46fc] = &Inst {op: Opcode::MoveToSrIm};
        m[0x4e70] = &Inst {op: Opcode::Reset};
        m[0x4e71] = &Inst {op: Opcode::Nop};
        m[0x4e73] = &Inst {op: Opcode::Rte};
        m[0x4e75] = &Inst {op: Opcode::Rts};
        mask_inst(&mut m, 0xffc0, 0x4200, &Inst {op: Opcode::Clr});  // 4200-423f: clr.b
        mask_inst(&mut m, 0xffc0, 0x4240, &Inst {op: Opcode::Clr});  // 4240-427f: clr.w
        mask_inst(&mut m, 0xffc0, 0x4280, &Inst {op: Opcode::Clr});  // 4280-42bf: clr.l
        mask_inst(&mut m, 0xfff8, 0x48e0, &Inst {op: Opcode::MovemFrom});  // 48e0-48e7
        mask_inst(&mut m, 0xffc0, 0x4a00, &Inst {op: Opcode::TstByte});  // 4a00-4a3f
        mask_inst(&mut m, 0xffc0, 0x4a40, &Inst {op: Opcode::TstWord});  // 4a40-4a7f
        mask_inst(&mut m, 0xffc0, 0x4a80, &Inst {op: Opcode::TstLong});  // 4a80-4abf
        mask_inst(&mut m, 0xfff8, 0x4cd8, &Inst {op: Opcode::MovemTo});  // 4cd8-4cdf
        mask_inst(&mut m, 0xfff0, 0x4e40, &Inst {op: Opcode::Trap});  // 4e40-4e4f
        mask_inst(&mut m, 0xfff0, 0x4e90, &Inst {op: Opcode::JsrA});  // 4e90-4e9f
        for i in 0..8 {
            let o = i * 0x0200;
            range_inst(&mut m, &mut ((0x5080 + o)..(0x50ba + o)), &Inst {op: Opcode::AddqLong});  // 5080...50ba, 5280...52ba, ..., 5eba
            range_inst(&mut m, &mut ((0x5140 + o)..(0x517a + o)), &Inst {op: Opcode::SubqWord});  // 5140...517a, 5340...537a, ..., 5f7a
        }
        mask_inst(&mut m, 0xfff8, 0x51c8, &Inst {op: Opcode::Dbra});  // 51c8-51cf
        mask_inst(&mut m, 0xff00, 0x6100, &Inst {op: Opcode::Bsr});  // 6100-61ff
        mask_inst(&mut m, 0xff00, 0x6400, &Inst {op: Opcode::Bcc});  // 6400-64ff
        mask_inst(&mut m, 0xff00, 0x6500, &Inst {op: Opcode::Bcs});  // 6500-65ff
        mask_inst(&mut m, 0xff00, 0x6600, &Inst {op: Opcode::Bne});  // 6600-66ff
        mask_inst(&mut m, 0xff00, 0x6700, &Inst {op: Opcode::Beq});  // 6700-67ff
        mask_inst(&mut m, 0xf100, 0x7000, &Inst {op: Opcode::Moveq});  // 7000...70ff, 7200...72ff, ..., 7eff
        mask_inst(&mut m, 0xf1c0, 0x91c0, &Inst {op: Opcode::SubaLong});  // 91c0, 91c1, 93c0, ..., 9fff
        mask_inst(&mut m, 0xf1c0, 0xb000, &Inst {op: Opcode::CmpByte});  // b000-b03f, b200-b23f, ..., be3f
        mask_inst(&mut m, 0xf1c0, 0xb040, &Inst {op: Opcode::CmpWord});  // b040-b07f, b240-b27f, ..., be7f
        mask_inst(&mut m, 0xf1f8, 0xb108, &Inst {op: Opcode::CmpmByte});  // b108, b109, b308, ..., bf0f
        mask_inst(&mut m, 0xf1c0, 0xb1c0, &Inst {op: Opcode::CmpaLong});  // b1c0, b1c1, b3c0, b3c1, ..., bfff
        mask_inst(&mut m, 0xf1c0, 0xc080, &Inst {op: Opcode::AndLong});  // c080, c081, c280, ..., cebf
        mask_inst(&mut m, 0xf1c0, 0xd080, &Inst {op: Opcode::AddLong});  // d080, d081, d280, ..., de87
        mask_inst(&mut m, 0xf1c0, 0xd1c0, &Inst {op: Opcode::AddaLong});  // d1c8, d1c9, d3c8, ..., dfff
        m
    };
}
