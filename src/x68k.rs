use lazy_static::lazy_static;

#[derive(Clone)]
enum Opcode {
    Unknown,
    MoveLong,            // move.l XX, YY
    MoveToSrIm,          // move #$xxxx, SR
    LeaDirect,           // lea $xxxxxxxx, Ax
    Reset,               // reset
    SubaLong,            // suba.l As, Ad
    Bsr,                 // bsr $xxxx
}

#[derive(Clone)]
struct Inst {
    op: Opcode,
}

fn mask_inst(m: &mut Vec<&Inst>, mask: u16, value: u16, inst: &'static Inst) {
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
    static ref INST: Vec<&'static Inst> = {
        let mut m = vec![&Inst {op: Opcode::Unknown}; 0x10000];
        mask_inst(&mut m, 0xf000, 0x2000, &Inst {op: Opcode::MoveLong});  // 2000-2fff
        mask_inst(&mut m, 0xf1ff, 0x41f9, &Inst {op: Opcode::LeaDirect});  // 41f9, 43f9, ..., 4ff9
        m[0x46fc] = &Inst {op: Opcode::MoveToSrIm};
        m[0x4e70] = &Inst {op: Opcode::Reset};
        mask_inst(&mut m, 0xff00, 0x6100, &Inst {op: Opcode::Bsr});  // 6100-61ff
        mask_inst(&mut m, 0xf1f8, 0x91c8, &Inst {op: Opcode::SubaLong});  // 91c8, 91c9, 93c8, ..., 9fcf
        m
    };
}

const DREG: usize = 0;
const AREG: usize = 8;
const SP: usize = 7 + AREG;  // Stack pointer = A7 register.

pub struct Cpu {
    mem: Vec<u8>,
    ipl: Vec<u8>,
    regs: Vec<u32>,
    pc: u32,
    sr: u16,
}

impl Cpu {
    fn reset(&mut self) {
        self.sr = 0;
        self.regs[SP] = self.read32(0xff0000);
        self.pc = self.read32(0xff0004);
    }

    pub fn run(&mut self) {
        loop {
            let (_sz, mnemonic) = disasm(&self, self.pc);
            println!("{:08x}: {}", self.pc, mnemonic);
            self.step();
        }
    }

    fn step(&mut self) {
        let startadr = self.pc;
        let op = self.read16(self.pc);
        self.pc += 2;
        let inst = &INST[op as usize];

        match inst.op {
            Opcode::MoveLong => {
                let n = ((op >> 9) & 7) as usize;
                let m = (op & 7) as usize;
                let dt = ((op >> 6) & 7) as usize;
                let src = self.read_source32(((op >> 3) & 7) as usize, m);
                self.write_destination32(dt, n, src);
            },
            Opcode::MoveToSrIm => {
                self.sr = self.read16(self.pc);
                self.pc += 2;
            },
            Opcode::LeaDirect => {
                let di = ((op >> 9) & 7) as usize;
                let value = self.read32(self.pc);
                self.pc += 4;
                self.regs[di + AREG] = value;
            },
            Opcode::Reset => {
                // TODO: Implement.
            },
            Opcode::SubaLong => {
                let di = ((op >> 9) & 7) as usize;
                let si = (op & 7) as usize;
                self.regs[di + AREG] -= self.regs[si + AREG];
            },
            Opcode::Bsr => {
                let mut ofs = ((op & 0x00ff) as i8) as i16;
                if ofs == 0 {
                    ofs = self.read16(self.pc) as i16;
                    self.pc += 2;
                }
                self.push32(self.pc);
                self.pc = ((startadr + 2) as i32 + ofs as i32) as u32;
            },
            _ => {
                eprintln!("{:08x}: {:04x}  ; Unknown opcode", startadr, op);
                panic!("Not implemented");
            },
        }
    }

    fn push32(&mut self, value: u32) {
        let sp = self.regs[SP] - 4;
        self.regs[SP] = sp;
        self.write32(sp, value);
    }

    fn read_source32(&mut self, src: usize, m: usize) -> u32 {
        match src {
            0 => {  // move.l Dm, xx
                self.regs[m + DREG]
            },
            7 => {  // Misc.
                match m {
                    4 => {  // move.l #$XXXX, xx
                        let value = self.read32(self.pc);
                        self.pc += 4;
                        value
                    },
                    _ => {
                        panic!("Not implemented, m={:?}", m);
                    },
                }
            },
            _ => {
                panic!("Not implemented, src={:?}", src);
            },
        }
    }

    fn write_destination32(&mut self, dst: usize, n: usize, value: u32) {
        match dst {
            0 => {
                self.regs[n + DREG] = value;
            },
            _ => {
                panic!("Not implemented, dst={:?}", dst);
            },
        }
    }

    fn read8(&self, adr: u32) -> u8 {
        if 0xfe0000 <= adr && adr <= 0xffffff {
            self.ipl[(adr - 0xfe0000) as usize]
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }

    fn read16(&self, adr: u32) -> u16 {
        let d0 = self.read8(adr) as u16;
        let d1 = self.read8(adr + 1) as u16;
        (d0 << 8) | d1
    }

    fn read32(&self, adr: u32) -> u32 {
        let d0 = self.read8(adr) as u32;
        let d1 = self.read8(adr + 1) as u32;
        let d2 = self.read8(adr + 2) as u32;
        let d3 = self.read8(adr + 3) as u32;
        (d0 << 24) | (d1 << 16) | (d2 << 8) | d3
    }

    fn write8(&mut self, adr: u32, value: u8) {
        if /*0x000000 <= adr &&*/ adr <= 0xffff {
            self.mem[adr as usize] = value;
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }

    fn write32(&mut self, adr: u32, value: u32) {
        self.write8(adr,     (value >> 24) as u8);
        self.write8(adr + 1, (value >> 16) as u8);
        self.write8(adr + 2, (value >>  8) as u8);
        self.write8(adr + 3,  value        as u8);
    }
}

pub fn new_cpu(ipl: Vec<u8>) -> Cpu {
    let mut cpu = Cpu{mem: vec![0; 0x10000], ipl: ipl, regs: vec![0; 8 + 8], pc: 0, sr: 0};
    cpu.reset();
    cpu
}

////////////////////////////////////////////////////////////////
// disasm

fn disasm(cpu: &Cpu, adr: u32) -> (usize, String) {
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
        Opcode::MoveToSrIm => {
            let sr = cpu.read16(adr + 2);
            (2, format!("move #${:04x}, SR", sr))
        },
        Opcode::LeaDirect => {
            let di = ((op >> 9) & 7) as usize;
            let value = cpu.read32(adr + 2);
            (4, format!("lea ${:08x}.l, A{:?}", value, di))
        },
        Opcode::Reset => {
            (0, "reset".to_string())
        },
        Opcode::SubaLong => {
            let di = ((op >> 9) & 7) as usize;
            let si = (op & 7) as usize;
            (0, format!("suba.l A{}, A{}", si, di))
        },
        Opcode::Bsr => {
            let mut ofs = ((op & 0x00ff) as i8) as i16;
            let mut sz = 0;
            if ofs == 0 {
                ofs = cpu.read16(adr + 2) as i16;
                sz = 2;
            }
            let jmp = ((adr + 2) as i32 + ofs as i32) as u32;
            (sz, format!("bsr ${:06x}", jmp))
        },
        _ => {
            eprintln!("{:08x}: {:04x}  ; Unknown opcode", adr, op);
            panic!("Not implemented");
        },
    }
}

fn disasm_read_source32(cpu: &Cpu, adr: u32,  src: usize, m: u16) -> (u32, String) {
    match src {
        0 => {  // move.l Dm, xx
            (0, format!("D{}", m))
        },
        7 => {  // Misc.
            match m {
                4 => {  // move.l #$XXXX, xx
                    let value = cpu.read32(adr);
                    (4, format!("#${:08x}", value))
                },
                _ => {
                    panic!("Not implemented, m={:?}", m);
                },
            }
        },
        _ => {
            panic!("Not implemented, src={:?}", src);
        },
    }
}

fn disasm_write_destination32(_cpu: &Cpu, _adr: u32, dst: usize, n: u16) -> (u32, String) {
    match dst {
        0 => {
            (0, format!("D{}", n))
        },
        _ => {
            panic!("Not implemented, dst={:?}", dst);
        },
    }
}
