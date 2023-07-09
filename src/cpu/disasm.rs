use super::bus_trait::BusTrait;
use super::opcode::{Opcode, INST};
use super::util::{get_branch_offset, conv07to18};
use super::super::types::{Byte, Word, Long, SByte, SWord, SLong, Adr};

const DREG_NAMES: [&str; 8] = ["D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7"];
const AREG_NAMES: [&str; 8] = ["A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7"];
const AINDIRECT_NAMES: [&str; 8] = ["(A0)", "(A1)", "(A2)", "(A3)", "(A4)", "(A5)", "(A6)", "(A7)"];
const APOSTINC_NAMES: [&str; 8] = ["(A0)+", "(A1)+", "(A2)+", "(A3)+", "(A4)+", "(A5)+", "(A6)+", "(A7)+"];
const APREDEC_NAMES: [&str; 8] = ["-(A0)", "-(A1)", "-(A2)", "-(A3)", "-(A4)", "-(A5)", "-(A6)", "-(A7)"];

const MOVE_NAMES: [&str; 8] = ["move", "movea", "move", "move", "move", "move", "move", "move"];

fn dreg(no: Word) -> String { DREG_NAMES[no as usize].to_string() }
fn areg(no: Word) -> String { AREG_NAMES[no as usize].to_string() }
fn aind(no: Word) -> String { AINDIRECT_NAMES[no as usize].to_string() }
fn apostinc(no: Word) -> String { APOSTINC_NAMES[no as usize].to_string() }
fn apredec(no: Word) -> String { APREDEC_NAMES[no as usize].to_string() }

pub fn disasm<BusT: BusTrait>(bus: &mut BusT, adr: Adr) -> (usize, String) {
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
            let mnemonic = format!("{}.b", MOVE_NAMES[dt]);
            ((2 + ssz + dsz) as usize, format!("{:<7} {}, {}", mnemonic, sstr, dstr))
        },
        Opcode::MoveWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let dt = ((op >> 6) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination16(bus, adr + 2 + ssz, dt, di);
            let mnemonic = format!("{}.w", MOVE_NAMES[dt]);
            ((2 + ssz + dsz) as usize, format!("{:<7} {}, {}", mnemonic, sstr, dstr))
        },
        Opcode::MoveLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let dt = ((op >> 6) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination32(bus, adr + 2 + ssz, dt, di);
            let mnemonic = format!("{}.l", MOVE_NAMES[dt]);
            ((2 + ssz + dsz) as usize, format!("{:<7} {}, {}", mnemonic, sstr, dstr))
        },
        Opcode::Moveq => {
            let v = op as Byte;
            let di = (op >> 9) & 7;
            (2, format!("moveq   #{}, {}", signed_hex8(v), dreg(di)))
        },
        Opcode::MovemFrom => {
            let di = op & 7;
            let bits = bus.read16(adr + 2);
            let regs = movem_regs(bits, true);
            (4, format!("movem.l {}, {}", regs, apredec(di)))
        },
        Opcode::MovemTo => {
            let si = op & 7;
            let bits = bus.read16(adr + 2);
            let regs = movem_regs(bits, false);
            (4, format!("movem.l {}, {}", apostinc(si), regs))
        },
        Opcode::MoveToSrIm => {
            let val = bus.read16(adr + 2);
            (4, format!("move    #${:04x}, SR", val))
        },
        Opcode::MoveToSr => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("move    {}, SR", sstr))
        },
        Opcode::MoveFromSr => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let (dsz, dstr) = write_destination16(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("move    SR, {}", dstr))
        },
        Opcode::LeaDirect => {
            let di = (op >> 9) & 7;
            let value = bus.read32(adr + 2);
            (6, format!("lea     ${:x}.l, {}", value, areg(di)))
        },
        Opcode::LeaOffset => {
            let si = op & 7;
            let di = (op >> 9) & 7;
            let ofs = bus.read16(adr + 2);
            (4, format!("lea     ({},{}), {}", signed_hex16(ofs), areg(si), areg(di)))
        },
        Opcode::LeaOffsetD => {
            let si = op & 7;
            let di = (op >> 9) & 7;
            let next = bus.read16(adr + 2);
            if (next & 0x8f00) == 0x0000 {
                let ofs = next as Byte;
                let ii = (next >> 12) & 0x07;
                if ofs == 0 {
                    (4, format!("lea     ({},{}.w), {}", areg(si), dreg(ii), areg(di)))
                } else {
                    (4, format!("lea     ({},{},{}.w), {}", signed_hex8(ofs), areg(si), dreg(ii), areg(di)))
                }
            } else {
                (4, "**Not implemented**".to_string())
            }
        },
        Opcode::LeaOffsetPc => {
            let di = (op >> 9) & 7;
            let ofs = bus.read16(adr + 2);
            (4, format!("lea     ({},PC), {}", signed_hex16(ofs), areg(di)))
        },
        Opcode::ClrByte => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let (dsz, dstr) = write_destination8(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("clr.b   {}", dstr))
        },
        Opcode::ClrWord => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let (dsz, dstr) = write_destination16(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("clr.w   {}", dstr))
        },
        Opcode::ClrLong => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let (dsz, dstr) = write_destination16(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("clr.l   {}", dstr))
        },
        Opcode::Swap => {
            let di = op & 7;
            (2, format!("swap    {}", dreg(di)))
        },
        Opcode::CmpByte => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source8(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination8(bus, adr + 2 + ssz, 0, di);
            ((2 + ssz + dsz) as usize, format!("cmp.b   {}, {}", sstr, dstr))
        },
        Opcode::CmpWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination16(bus, adr + 2 + ssz, 0, di);
            ((2 + ssz + dsz) as usize, format!("cmp.w   {}, {}", sstr, dstr))
        },
        Opcode::CmpLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination32(bus, adr + 2 + ssz, 0, di);
            ((2 + ssz + dsz) as usize, format!("cmp.l   {}, {}", sstr, dstr))
        },
        Opcode::CmpiByte => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let val = bus.read16(adr + 2) as Byte;
            let (dsz, dstr) = write_destination8(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("cmpi.b  #{}, {}", signed_hex8(val), dstr))
        },
        Opcode::CmpiWord => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let val = bus.read16(adr + 2);
            let (dsz, dstr) = write_destination16(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("cmpi.w  #{}, {}", signed_hex16(val), dstr))
        },
        Opcode::CmpaLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            let (dsz, dstr) = write_destination32(bus, adr + 2 + ssz, 1, di);
            ((2 + ssz + dsz) as usize, format!("cmpa.l  {}, {}", sstr, dstr))
        },
        Opcode::CmpmByte => {
            let si = op & 7;
            let di = (op >> 9) & 7;
            (2, format!("cmpm.b  {}, {}", apostinc(si), apostinc(di)))
        },
        Opcode::Cmp2Byte => {
            let word2 = bus.read16(adr + 2);
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (word2 >> 12) & 15;
            let (ssz, sstr) = read_source8(bus, adr + 4, st, si);
            if di < 8 {
                ((4 + ssz) as usize, format!("cmp2.b  {}, {}", sstr, dreg(di)))
            } else {
                ((4 + ssz) as usize, format!("cmp2.b  {}, {}", sstr, areg(di - 8)))
            }
        },
        Opcode::TstByte => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let (ssz, sstr) = read_source8(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("tst.b   {}", sstr))
        },
        Opcode::TstWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("tst.w   {}", sstr))
        },
        Opcode::TstLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("tst.l   {}", sstr))
        },
        Opcode::BtstIm => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let bit = bus.read16(adr + 2);
            let (ssz, sstr) = read_source16(bus, adr + 4, st, si);
            ((4 + ssz) as usize, format!("btst    #${:x}, {}", bit, sstr))
        },
        Opcode::BclrIm => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let bit = bus.read16(adr + 2);
            let (dsz, dstr) = write_destination16(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("bclr    #${:x}, {}", bit, dstr))
        },
        Opcode::Bset => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let si = (op >> 9) & 7;
            let (dsz, dstr) = write_destination8(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("bset    {}, {}", dreg(si), dstr))
        },
        Opcode::BsetIm => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let bit = bus.read16(adr + 2);
            let (dsz, dstr) = write_destination16(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("bset    #${:x}, {}", bit, dstr))
        },
        Opcode::Reset => {
            (2, "reset".to_string())
        },
        Opcode::AddByte => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source8(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("add.b   {}, {}", sstr, dreg(di)))
        },
        Opcode::AddWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("add.w   {}, {}", sstr, dreg(di)))
        },
        Opcode::AddLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("add.l   {}, {}", sstr, dreg(di)))
        },
        Opcode::AddiByte => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = bus.read16(adr + 2) as Byte;
            let (dsz, dstr) = write_destination8(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("addi.b  #${:x}, {}", v, dstr))
        },
        Opcode::AddiWord => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = bus.read16(adr + 2);
            let (dsz, dstr) = write_destination16(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("addi.w  #${:x}, {}", v, dstr))
        },
        Opcode::AddaLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("adda.l  {}, {}", sstr, areg(di)))
        },
        Opcode::AddqByte => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = conv07to18(op >> 9);
            let (dsz, dstr) = write_destination8(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("addq.b  #{}, {}", v, dstr))
        },
        Opcode::AddqWord => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = conv07to18(op >> 9);
            let (dsz, dstr) = write_destination16(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("addq.w  #{}, {}", v, dstr))
        },
        Opcode::AddqLong => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = conv07to18(op >> 9);
            let (dsz, dstr) = write_destination32(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("addq.l  #{}, {}", v, dstr))
        },
        Opcode::SubByte => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source8(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("sub.b   {}, {}", sstr, dreg(di)))
        },
        Opcode::SubWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("sub.w   {}, {}", sstr, dreg(di)))
        },
        Opcode::SubiByte => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = bus.read16(adr + 2) as Byte;
            let (dsz, dstr) = write_destination8(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("subi.b  #${:02x}, {}", v, dstr))
        },
        Opcode::SubaLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("suba.l  {}, {}", sstr, areg(di)))
        },
        Opcode::SubqWord => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = conv07to18(op >> 9);
            let (dsz, dstr) = write_destination16(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("subq.w  #{}, {}", v, dstr))
        },
        Opcode::SubqLong => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = conv07to18(op >> 9);
            let (dsz, dstr) = write_destination32(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("subq.l  #{}, {}", v, dstr))
        },
        Opcode::MuluWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("mulu.w  {}, {}", sstr, dreg(di)))
        },
        Opcode::AndByte => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source8(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("and.b   {}, {}", sstr, dreg(di)))
        },
        Opcode::AndWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("and.w   {}, {}", sstr, dreg(di)))
        },
        Opcode::AndLong => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source32(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("and.l   {}, {}", sstr, dreg(di)))
        },
        Opcode::AndiWord => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = bus.read16(adr + 2);
            let (dsz, dstr) = write_destination16(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("andi.w  #${:x}, {}", v, dstr))
        },
        Opcode::OrByte => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source8(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("or.b    {}, {}", sstr, dreg(di)))
        },
        Opcode::OrWord => {
            let si = op & 7;
            let st = ((op >> 3) & 7) as usize;
            let di = (op >> 9) & 7;
            let (ssz, sstr) = read_source16(bus, adr + 2, st, si);
            ((2 + ssz) as usize, format!("or.w    {}, {}", sstr, dreg(di)))
        },
        Opcode::OriByte => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = bus.read16(adr + 2) as Byte;
            let (dsz, dstr) = write_destination8(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("ori.b   #${:x}, {}", v, dstr))
        },
        Opcode::OriWord => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = bus.read16(adr + 2);
            let (dsz, dstr) = write_destination16(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("ori.w   #${:x}, {}", v, dstr))
        },
        Opcode::EorByte => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let si = (op >> 9) & 7;
            let (dsz, dstr) = write_destination8(bus, adr + 2, dt, di);
            ((2 + dsz) as usize, format!("eor.b   {}, {}", dreg(si), dstr))
        },
        Opcode::EoriByte => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = bus.read16(adr + 2) as Byte;
            let (dsz, dstr) = write_destination8(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("eori.b  #${:x}, {}", v, dstr))
        },
        Opcode::EoriWord => {
            let di = op & 7;
            let dt = ((op >> 3) & 7) as usize;
            let v = bus.read16(adr + 2);
            let (dsz, dstr) = write_destination16(bus, adr + 4, dt, di);
            ((4 + dsz) as usize, format!("eori.w  #${:x}, {}", v, dstr))
        },
        Opcode::AslImByte => {
            let di = op & 7;
            let shift = conv07to18(op >> 9);
            (2, format!("asl.b   #{}, {}", shift, dreg(di)))
        },
        Opcode::AslImWord => {
            let di = op & 7;
            let shift = conv07to18(op >> 9);
            (2, format!("asl.w   #{}, {}", shift, dreg(di)))
        },
        Opcode::AslImLong => {
            let di = op & 7;
            let shift = conv07to18(op >> 9);
            (2, format!("asl.l   #{}, {}", shift, dreg(di)))
        },
        Opcode::LsrImByte => {
            let di = op & 7;
            let shift = conv07to18(op >> 9);
            (2, format!("lsr.b   #{}, {}", shift, dreg(di)))
        },
        Opcode::LsrImWord => {
            let di = op & 7;
            let shift = conv07to18(op >> 9);
            (2, format!("lsr.w   #{}, {}", shift, dreg(di)))
        },
        Opcode::LslImWord => {
            let di = op & 7;
            let shift = conv07to18(op >> 9);
            (2, format!("lsl.w   #{}, {}", shift, dreg(di)))
        },
        Opcode::RorImWord => {
            let di = op & 7;
            let si = conv07to18(op >> 9);
            (2, format!("ror.w   #{}, {}", si, dreg(di)))
        },
        Opcode::RorImLong => {
            let di = op & 7;
            let si = conv07to18(op >> 9);
            (2, format!("ror.l   #{}, {}", si, dreg(di)))
        },
        Opcode::RolWord => {
            let di = op & 7;
            let si = (op >> 9) & 7;
            (2, format!("rol.w   {}, {}", dreg(si), dreg(di)))
        },
        Opcode::RolImByte => {
            let di = op & 7;
            let si = conv07to18(op >> 9);
            (2, format!("rol.b   #{}, {}", si, dreg(di)))
        },
        Opcode::ExtWord => {
            let di = op & 7;
            (2, format!("ext.w   {}", dreg(di)))
        },
        Opcode::Bra => { bcond(bus, adr + 2, op, "bra") },
        Opcode::Bcc => { bcond(bus, adr + 2, op, "bcc") },
        Opcode::Bcs => { bcond(bus, adr + 2, op, "bcs") },
        Opcode::Bne => { bcond(bus, adr + 2, op, "bne") },
        Opcode::Beq => { bcond(bus, adr + 2, op, "beq") },
        Opcode::Bpl => { bcond(bus, adr + 2, op, "bpl") },
        Opcode::Bmi => { bcond(bus, adr + 2, op, "bmi") },
        Opcode::Bge => { bcond(bus, adr + 2, op, "bge") },
        Opcode::Blt => { bcond(bus, adr + 2, op, "blt") },
        Opcode::Bgt => { bcond(bus, adr + 2, op, "bgt") },
        Opcode::Ble => { bcond(bus, adr + 2, op, "ble") },
        Opcode::Dbra => {
            let si = op & 7;
            let ofs = bus.read16(adr + 2) as SWord;
            let jmp = ((adr + 2) as SLong).wrapping_add(ofs as SLong) as Long;
            (4, format!("dbra    {}, {:x}", dreg(si), jmp))
        },
        Opcode::Bsr => {
            let (ofs, sz) = get_branch_offset(op, bus, adr + 2);
            let jmp = ((adr + 2) as SLong + ofs) as Long;
            ((2 + sz) as usize, format!("bsr     {:x}", jmp))
        },
        Opcode::JsrA => {
            let si = op & 7;
            if (op & 15) < 8 {
                (2, format!("jsr     ({})", areg(si)))
            } else {
                let offset = bus.read16(adr + 2);
                (4, format!("jsr     (${:x}, {})", offset, areg(si)))
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
            (2, format!("trap    #${:x}", no))
        },
        _ => {
            (2, format!("**{:04x}** Unknown opcode", op))
        },
    }
}

fn signed_hex8(x: Byte) -> String {
    if x < 0x80 {
        format!("${:x}", x)
    } else {
        format!("-${:x}", (0 as SByte).wrapping_sub(x as SByte) as Byte)
    }
}

fn signed_hex16(x: Word) -> String {
    if x < 0x8000 {
        format!("${:x}", x)
    } else {
        format!("-${:x}", (0 as SWord).wrapping_sub(x as SWord) as Word)
    }
}

fn bcond<BusT: BusTrait>(bus: &mut BusT, adr: Adr, op: Word, bname: &str) -> (usize, String) {
    let (ofs, sz) = get_branch_offset(op, bus, adr);
    let jmp = (adr as SLong).wrapping_add(ofs) as Long;
    ((2 + sz) as usize, format!("{}     {:x}", bname, jmp))
}

fn movem_regs(bits: Word, inv: bool) -> String {
    const DA: [&str; 2] = ["D", "A"];

    fn bit(i: usize, j: usize, inv: bool) -> u16 {
        let index = i * 8 + j;
        let shift = if inv {15 - index} else {index};
        1 << shift
    }

    let mut regs = Vec::new();
    for (i, da) in DA.iter().enumerate() {
        let mut j = 0;
        loop {
            if (bits & bit(i, j, inv)) == 0 {
                j += 1;
            } else {
                let mut k = j;
                loop {
                    k += 1;
                    if k >= 8 || (bits & bit(i, k, inv)) == 0 { break; }
                }
                if k == j + 1 {
                    regs.push(format!("{}{}", da, j));
                } else {
                    regs.push(format!("{}{}-{}{}", da, j, da, k - 1));
                }
                j = k;
            }
            if j >= 8 { break; }
        }
    }
    regs.join("/")
}

fn read_source8<BusT: BusTrait>(bus: &mut BusT, adr: Adr,  src: usize, m: Word) -> (u32, String) {
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
            (2, format!("(${:x},{})", ofs, areg(m)))
        },
        7 => {  // Misc.
            match m {
                1 => {  // move.b $XXXXXXXX.l, xx
                    let adr = bus.read32(adr);
                    (4, format!("${:x}.l", adr))
                },
                4 => {  // move.b #$XXXX, xx
                    let value = bus.read16(adr);
                    (2, format!("#${:x}", value & 0x00ff))
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

fn read_source16<BusT: BusTrait>(bus: &mut BusT, adr: Adr,  src: usize, m: Word) -> (u32, String) {
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
            (2, format!("(${:x},{})", ofs, areg(m)))
        },
        6 => {  // Memory Indirect Pre-indexed: move.w xx, (123, An, Dx)
            let extension = bus.read16(adr);
            if (extension & 0x100) != 0 {
                (2, format!("UnhandledSrc(6/{:04x})", extension))
            } else {
                let ofs = extension as SByte;
                let da = (extension & 0x8000) != 0;  // Displacement is address register?
                let dr = (extension >> 12) & 7;  // Displacement register.
                let dl = (extension & 0x0800) != 0;  // Displacement long?
                if ofs == 0 {
                    (2, format!("({},{}.{})", areg(m), if da {areg(dr)} else {dreg(dr)}, if dl {'l'} else {'w'}))
                } else {
                    (2, format!("({},{},{}.{})", ofs, areg(m), if da {areg(dr)} else {dreg(dr)}, if dl {'l'} else {'w'}))
                }
            }
        },
        7 => {  // Misc.
            match m {
                1 => {  // move.b $XXXXXXXX.l, xx
                    let adr = bus.read32(adr);
                    (4, format!("${:x}.l", adr))
                },
                4 => {  // move.w #$XXXX, xx
                    let value = bus.read16(adr);
                    (2, format!("#${:x}", value))
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

fn read_source32<BusT: BusTrait>(bus: &mut BusT, adr: Adr,  src: usize, m: Word) -> (u32, String) {
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
            (2, format!("(${:x},{})", ofs, areg(m)))
        },
        6 => {  // Memory Indirect Pre-indexed: move.l xx, (123, An, Dx)
            let extension = bus.read16(adr);
            if (extension & 0x100) != 0 {
                (2, format!("UnhandledSrc(6/{:04x})", extension))
            } else {
                let ofs = extension as SByte;
                let da = (extension & 0x8000) != 0;  // Displacement is address register?
                let dr = (extension >> 12) & 7;  // Displacement register.
                let dl = (extension & 0x0800) != 0;  // Displacement long?
                if ofs == 0 {
                    (2, format!("({},{}.{})", areg(m), if da {areg(dr)} else {dreg(dr)}, if dl {'l'} else {'w'}))
                } else {
                    (2, format!("({},{},{}.{})", ofs, areg(m), if da {areg(dr)} else {dreg(dr)}, if dl {'l'} else {'w'}))
                }
            }
        },
        7 => {  // Misc.
            match m {
                1 => {  // move.b $XXXXXXXX.l, xx
                    let adr = bus.read32(adr);
                    (4, format!("${:x}.l", adr))
                },
                4 => {  // move.l #$XXXX, xx
                    let value = bus.read32(adr);
                    (4, format!("#${:x}", value))
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

fn write_destination8<BusT: BusTrait>(bus: &mut BusT, adr: Adr, dst: usize, n: Word) -> (u32, String) {
    match dst {
        0 => {
            (0, dreg(n))
        },
        2 => {  // move.b xx, (An)
            (0, aind(n))
        },
        3 => {
            (0, apostinc(n))
        },
        5 => {  // move.b xx, (123, An)
            let ofs = bus.read16(adr) as SWord;
            (2, format!("(${:x},{})", ofs, areg(n)))
        },
        6 => {  // Memory Indirect Pre-indexed: move.b xx, (123, An, Dx)
            let extension = bus.read16(adr);
            if (extension & 0x100) != 0 {
                (2, format!("UnhandledDst(6/{:04x})", extension))
            } else {
                let ofs = extension as SByte;
                let da = (extension & 0x8000) != 0;  // Displacement is address register?
                let dr = (extension >> 12) & 7;  // Displacement register.
                let dl = (extension & 0x0800) != 0;  // Displacement long?
                if ofs == 0 {
                    (2, format!("({},{}.{})", areg(n), if da {areg(dr)} else {dreg(dr)}, if dl {'l'} else {'w'}))
                } else {
                    (2, format!("({},{},{}.{})", ofs, areg(n), if da {areg(dr)} else {dreg(dr)}, if dl {'l'} else {'w'}))
                }
            }
        },
        7 => {
            match n {
                1 => {
                    let d = bus.read32(adr);
                    (4, format!("${:x}.l", d))
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

fn write_destination16<BusT: BusTrait>(bus: &mut BusT, adr: Adr, dst: usize, n: Word) -> (u32, String) {
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
            (2, format!("(${:x},{})", ofs, areg(n)))
        },
        7 => {
            match n {
                1 => {
                    let d = bus.read32(adr);
                    (4, format!("${:x}.l", d))
                },
                4 => {
                    (0, "SR".to_string())
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

fn write_destination32<BusT: BusTrait>(bus: &mut BusT, adr: Adr, dst: usize, n: Word) -> (u32, String) {
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
            (2, format!("(${:x},{})", ofs, areg(n)))
        },
        7 => {
            match n {
                1 => {
                    let d = bus.read32(adr);
                    (4, format!("${:x}.l", d))
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
