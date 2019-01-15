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
    MoveToSr,            // move XX, SR
    MoveFromSr,          // move SR, XX
    LeaDirect,           // lea $xxxxxxxx, Ax
    LeaOffset,           // lea (xx, As), Ad
    LeaOffsetD,          // lea (xx, As, Dt), Ad
    LeaOffsetPc,         // lea (xx, PC), Ad
    ClrByte,             // clr.b xx
    ClrWord,             // clr.w xx
    ClrLong,             // clr.l xx
    Swap,                // swap Dd
    CmpByte,             // cmp.b XX, YY
    CmpWord,             // cmp.w XX, YY
    CmpLong,             // cmp.l XX, YY
    CmpiByte,            // cmpi.b #xx, YY
    CmpiWord,            // cmpi.w #xx, YY
    CmpaLong,            // cmpa.l XX, Ad
    CmpmByte,            // cmpm.b (Am)+, (An)+
    Cmp2Byte,            // cmp2.b XX, Dd
    TstByte,             // tst.b xx
    TstWord,             // tst.w xx
    TstLong,             // tst.l xx
    BtstIm,              // btst #xx, YY
    BclrIm,              // bclr #xx, YY
    Bset,                // bset Ds, YY
    BsetIm,              // bset #xx, YY
    AddByte,             // add.b XX, Dd
    AddWord,             // add.w XX, Dd
    AddLong,             // add.l XX, Dd
    AddiByte,            // addi.b XX, Dd
    AddiWord,            // addi.w XX, Dd
    AddaLong,            // adda.l XX, Ad
    AddqByte,            // addq.b #%d, D%d
    AddqWord,            // addq.w #%d, D%d
    AddqLong,            // addq.l #%d, D%d
    SubByte,             // sub.b XX, Dd
    SubWord,             // sub.w XX, Dd
    SubiByte,            // subi.b XX, Dd
    SubaLong,            // suba.l As, Ad
    SubqWord,            // subq.w #%d, D%d
    SubqLong,            // subq.l #%d, D%d
    MuluWord,            // mulu.w XX, Dd
    AndByte,             // and.b XX, Dd
    AndWord,             // and.w XX, Dd
    AndLong,             // and.l XX, Dd
    AndiWord,            // andi.w #xx, YY
    OrByte,              // or.b XX, Dd
    OrWord,              // or.w XX, Dd
    OriByte,             // ori.b #xx, YY
    OriWord,             // ori.w #xx, YY
    EorByte,             // eor.b XX, Dd
    EoriByte,            // eori.b #xx, YY
    EoriWord,            // eori.w #xx, YY
    AslImByte,           // asl.b #n, Dd
    AslImWord,           // asl.w #n, Dd
    AslImLong,           // asl.l #n, Dd
    LsrImByte,           // lsr.b #n, Dd
    LsrImWord,           // lsr.w #n, Dd
    LslImWord,           // lsl.w #n, Dd
    RorImWord,           // ror.w XX, Dd
    RolWord,             // rol.w Ds, Dd
    RolImByte,           // rol.b XX, Dd
    ExtWord,             // ext.w Dd
    Bra,                 // bra $xxxx
    Bcc,                 // bcc $xxxx
    Bcs,                 // bcs $xxxx
    Bne,                 // bne $xxxx
    Beq,                 // beq $xxxx
    Bpl,                 // bpl $xxxx
    Bmi,                 // bmi $xxxx
    Bge,                 // bge $xxxx
    Blt,                 // blt $xxxx
    Bgt,                 // bgt $xxxx
    Ble,                 // ble $xxxx
    Dbra,                // dbra $xxxx
    Bsr,                 // bsr $xxxx
    JsrA,                // jsr (Ax) or jsr ($ooo, Ax)
    Rts,                 // rts
    Rte,                 // rte
    Trap,                // trap #x
    Reset,               // reset
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
        mask_inst(&mut m, 0xffc0, 0x0000, &Inst {op: Opcode::OriByte});  // 0000-003f
        mask_inst(&mut m, 0xffc0, 0x0040, &Inst {op: Opcode::OriWord});  // 0040-007f
        mask_inst(&mut m, 0xf1c0, 0x01c0, &Inst {op: Opcode::Bset});  // 01c0-01ff, 03c0-03ff, ..., -0fff
        mask_inst(&mut m, 0xffc0, 0x0240, &Inst {op: Opcode::AndiWord});  // 0240-027f
        mask_inst(&mut m, 0xffc0, 0x0400, &Inst {op: Opcode::SubiByte});  // 0400-043f
        mask_inst(&mut m, 0xffc0, 0x0600, &Inst {op: Opcode::AddiByte});  // 0600-063f
        mask_inst(&mut m, 0xffc0, 0x0640, &Inst {op: Opcode::AddiWord});  // 0640-067f
        mask_inst(&mut m, 0xffc0, 0x0800, &Inst {op: Opcode::BtstIm});  // 0800-083f
        mask_inst(&mut m, 0xffc0, 0x0880, &Inst {op: Opcode::BclrIm});  // 0880-08bf
        mask_inst(&mut m, 0xffc0, 0x08c0, &Inst {op: Opcode::BsetIm});  // 08c0-08ff
        mask_inst(&mut m, 0xffc0, 0x0a00, &Inst {op: Opcode::EoriByte});  // 0a00-0a3f
        mask_inst(&mut m, 0xffc0, 0x0a40, &Inst {op: Opcode::EoriWord});  // 0a40-0a7f
        mask_inst(&mut m, 0xffc0, 0x0c00, &Inst {op: Opcode::CmpiByte});  // 0c00-0c3f
        mask_inst(&mut m, 0xffc0, 0x0c40, &Inst {op: Opcode::CmpiWord});  // 0c40-0c7f
        mask_inst(&mut m, 0xf000, 0x1000, &Inst {op: Opcode::MoveByte});  // 1000-1fff
        mask_inst(&mut m, 0xf000, 0x2000, &Inst {op: Opcode::MoveLong});  // 2000-2fff
        mask_inst(&mut m, 0xf000, 0x3000, &Inst {op: Opcode::MoveWord});  // 3000-3fff
        mask_inst(&mut m, 0xffc0, 0x40c0, &Inst {op: Opcode::MoveFromSr});  // 40c0-40ff
        mask_inst(&mut m, 0xf1f8, 0x41e8, &Inst {op: Opcode::LeaOffset});  // 41e8-41ef, 43e8-43ef, ..., -4fef
        mask_inst(&mut m, 0xf1f8, 0x41f0, &Inst {op: Opcode::LeaOffsetD});  // 41f0-41f7, 43f0-43f7, ..., -4ff7
        mask_inst(&mut m, 0xf1ff, 0x41f9, &Inst {op: Opcode::LeaDirect});  // 41f9, 43f9, ..., 4ff9
        mask_inst(&mut m, 0xf1ff, 0x41fa, &Inst {op: Opcode::LeaOffsetPc});  // 41fa, 43fa, ..., 4ffa
        m[0x46fc] = &Inst {op: Opcode::MoveToSrIm};
        m[0x4e70] = &Inst {op: Opcode::Reset};
        m[0x4e71] = &Inst {op: Opcode::Nop};
        m[0x4e73] = &Inst {op: Opcode::Rte};
        m[0x4e75] = &Inst {op: Opcode::Rts};
        mask_inst(&mut m, 0xffc0, 0x4200, &Inst {op: Opcode::ClrByte});  // 4200-423f
        mask_inst(&mut m, 0xffc0, 0x4240, &Inst {op: Opcode::ClrWord});  // 4240-427f
        mask_inst(&mut m, 0xffc0, 0x4280, &Inst {op: Opcode::ClrLong});  // 4280-42bf
        mask_inst(&mut m, 0xffc0, 0x46c0, &Inst {op: Opcode::MoveToSr});  // 46c0-46ff
        mask_inst(&mut m, 0xfff8, 0x4840, &Inst {op: Opcode::Swap});  // 4840-4847
        mask_inst(&mut m, 0xfff8, 0x4880, &Inst {op: Opcode::ExtWord});  // 4880-4887
        mask_inst(&mut m, 0xfff8, 0x48e0, &Inst {op: Opcode::MovemFrom});  // 48e0-48e7
        mask_inst(&mut m, 0xffc0, 0x4a00, &Inst {op: Opcode::TstByte});  // 4a00-4a3f
        mask_inst(&mut m, 0xffc0, 0x4a40, &Inst {op: Opcode::TstWord});  // 4a40-4a7f
        mask_inst(&mut m, 0xffc0, 0x4a80, &Inst {op: Opcode::TstLong});  // 4a80-4abf
        mask_inst(&mut m, 0xfff8, 0x4cd8, &Inst {op: Opcode::MovemTo});  // 4cd8-4cdf
        mask_inst(&mut m, 0xfff0, 0x4e40, &Inst {op: Opcode::Trap});  // 4e40-4e4f
        mask_inst(&mut m, 0xfff0, 0x4e90, &Inst {op: Opcode::JsrA});  // 4e90-4e9f
        for i in 0..8 {
            let o = i * 0x0200;
            range_inst(&mut m, &mut ((0x5000 + o)..(0x503a + o)), &Inst {op: Opcode::AddqByte});  // 5000...5039, 5200...5239, ..., 5e39
            range_inst(&mut m, &mut ((0x5040 + o)..(0x507a + o)), &Inst {op: Opcode::AddqWord});  // 5040...5079, 5240...5279, ..., 5e79
            range_inst(&mut m, &mut ((0x5080 + o)..(0x50ba + o)), &Inst {op: Opcode::AddqLong});  // 5080...50b9, 5280...52b9, ..., 5eb9
            range_inst(&mut m, &mut ((0x5140 + o)..(0x517a + o)), &Inst {op: Opcode::SubqWord});  // 5140...5179, 5340...5379, ..., 5f79
            range_inst(&mut m, &mut ((0x5180 + o)..(0x51ba + o)), &Inst {op: Opcode::SubqLong});  // 5180...51b9, 5380...53b9, ..., 5fb9
        }
        mask_inst(&mut m, 0xfff8, 0x51c8, &Inst {op: Opcode::Dbra});  // 51c8-51cf
        mask_inst(&mut m, 0xff00, 0x6000, &Inst {op: Opcode::Bra});  // 6000-60ff
        mask_inst(&mut m, 0xff00, 0x6100, &Inst {op: Opcode::Bsr});  // 6100-61ff
        mask_inst(&mut m, 0xff00, 0x6400, &Inst {op: Opcode::Bcc});  // 6400-64ff
        mask_inst(&mut m, 0xff00, 0x6500, &Inst {op: Opcode::Bcs});  // 6500-65ff
        mask_inst(&mut m, 0xff00, 0x6600, &Inst {op: Opcode::Bne});  // 6600-66ff
        mask_inst(&mut m, 0xff00, 0x6700, &Inst {op: Opcode::Beq});  // 6700-67ff
        mask_inst(&mut m, 0xff00, 0x6a00, &Inst {op: Opcode::Bpl});  // 6a00-6aff
        mask_inst(&mut m, 0xff00, 0x6b00, &Inst {op: Opcode::Bmi});  // 6b00-6bff
        mask_inst(&mut m, 0xff00, 0x6c00, &Inst {op: Opcode::Bge});  // 6c00-6cff
        mask_inst(&mut m, 0xff00, 0x6d00, &Inst {op: Opcode::Blt});  // 6d00-6dff
        mask_inst(&mut m, 0xff00, 0x6e00, &Inst {op: Opcode::Bgt});  // 6e00-6eff
        mask_inst(&mut m, 0xff00, 0x6f00, &Inst {op: Opcode::Ble});  // 6f00-6fff
        mask_inst(&mut m, 0xf100, 0x7000, &Inst {op: Opcode::Moveq});  // 7000...70ff, 7200...72ff, ..., 7eff
        mask_inst(&mut m, 0xf1c0, 0x8000, &Inst {op: Opcode::OrByte});  // 8000-803f, 8200-823f, ..., -8e3f
        mask_inst(&mut m, 0xf1c0, 0x8040, &Inst {op: Opcode::OrWord});  // 8040-807f, 8240-827f, ..., -8e7f
        mask_inst(&mut m, 0xf1c0, 0x9000, &Inst {op: Opcode::SubByte});  // 9000-903f, 9200-923f, ..., -9e3f
        mask_inst(&mut m, 0xf1c0, 0x9040, &Inst {op: Opcode::SubWord});  // 9040-907f, 9240-927f, ..., -9e7f
        mask_inst(&mut m, 0xf1c0, 0x91c0, &Inst {op: Opcode::SubaLong});  // 91c0-91ff, 93c0-93ff, ..., -9fff
        mask_inst(&mut m, 0xfff8, 0x00e8, &Inst {op: Opcode::Cmp2Byte});  // 00e8-00ef
        mask_inst(&mut m, 0xf1c0, 0xb000, &Inst {op: Opcode::CmpByte});  // b000-b03f, b200-b23f, ..., be3f
        mask_inst(&mut m, 0xf1c0, 0xb040, &Inst {op: Opcode::CmpWord});  // b040-b07f, b240-b27f, ..., be7f
        mask_inst(&mut m, 0xf1c0, 0xb080, &Inst {op: Opcode::CmpLong});  // b080-b0bf, b280-b2bf, ..., bebf
        mask_inst(&mut m, 0xf1c0, 0xb100, &Inst {op: Opcode::EorByte});  // b100-8000-803f, 8300-833f, ..., -8f3f
        mask_inst(&mut m, 0xf1f8, 0xb108, &Inst {op: Opcode::CmpmByte});  // b108-b10f, b308-b30f, ..., -bf0f
        mask_inst(&mut m, 0xf1c0, 0xb1c0, &Inst {op: Opcode::CmpaLong});  // b1c0-b1ff, b3c0-b3ff, ..., -bfff
        mask_inst(&mut m, 0xf1c0, 0xc000, &Inst {op: Opcode::AndByte});  // c000-c03f, c200-c23f, ..., -ce3f
        mask_inst(&mut m, 0xf1c0, 0xc040, &Inst {op: Opcode::AndWord});  // c040-c07f, c240-c27f, ..., -ce7f
        mask_inst(&mut m, 0xf1c0, 0xc080, &Inst {op: Opcode::AndLong});  // c080-c8bf, c280-c2bf, ..., -cebf
        mask_inst(&mut m, 0xf1c0, 0xc0c0, &Inst {op: Opcode::MuluWord});  // c0c0-c0fff, c2c0-c2ff, ..., -ceff
        mask_inst(&mut m, 0xf1c0, 0xd000, &Inst {op: Opcode::AddByte});  // d000-d03f, d200-d23f, ..., -de3f
        mask_inst(&mut m, 0xf1c0, 0xd040, &Inst {op: Opcode::AddWord});  // d040-d07f, d240-d27f, ..., -de7f
        mask_inst(&mut m, 0xf1c0, 0xd080, &Inst {op: Opcode::AddLong});  // d080-d0bf, d280-d2bf, ..., -debf
        mask_inst(&mut m, 0xf1c0, 0xd1c0, &Inst {op: Opcode::AddaLong});  // d1c8, d1c9, d3c8, ..., dfff
        mask_inst(&mut m, 0xf1f8, 0xe058, &Inst {op: Opcode::RorImWord});  // e058-e05f, e258-e25f, ..., ee58-ee5f
        mask_inst(&mut m, 0xf1f8, 0xe008, &Inst {op: Opcode::LsrImByte});  // e008-e00f, e208-e20f, ..., ee08-ee0f
        mask_inst(&mut m, 0xf1f8, 0xe048, &Inst {op: Opcode::LsrImWord});  // e048-e04f, e248-e24f, ..., ee48-ee4f
        mask_inst(&mut m, 0xf1f8, 0xe148, &Inst {op: Opcode::LslImWord});  // e148-e14f, e348-e34f, ..., ef48-ef4f
        mask_inst(&mut m, 0xf1f8, 0xe178, &Inst {op: Opcode::RolWord});  // e178-e17f, e378-e37f, ..., ef78-ef7f
        mask_inst(&mut m, 0xf1f8, 0xe118, &Inst {op: Opcode::RolImByte});  // e118-e11f, e318-e31f, ..., ef18-ef1f
        mask_inst(&mut m, 0xf1f8, 0xe100, &Inst {op: Opcode::AslImByte});  // e100-e107, e300-e307, ..., ef00-ef07
        mask_inst(&mut m, 0xf1f8, 0xe140, &Inst {op: Opcode::AslImWord});  // e140-e147, e340-e347, ..., ef40-ef47
        mask_inst(&mut m, 0xf1f8, 0xe180, &Inst {op: Opcode::AslImLong});  // e180-e187, e380-e387, ..., ef80-ef87
        m
    };
}
