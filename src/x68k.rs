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
                let (src, src_str) = self.read_source32(((op >> 3) & 7) as usize, m);
                let dst_str = self.write_destination32(dt, n, src);
                println!("{:08x}: move.l {}, {}", startadr, src_str, dst_str);
            },
            Opcode::MoveToSrIm => {
                self.sr = self.read16(self.pc);
                self.pc += 2;
                println!("{:08x}: move #${:04x}, SR", startadr, self.sr);
            },
            Opcode::LeaDirect => {
                let di = ((op >> 9) & 7) as usize;
                let value = self.read32(self.pc);
                self.pc += 4;
                self.regs[di + AREG] = value;
                println!("{:08x}: lea ${:08x}.l, A{:?}", startadr, value, di);
            },
            Opcode::Reset => {
                // TODO: Implement.
                println!("{:08x}: reset", startadr);
            },
            Opcode::SubaLong => {
                let di = ((op >> 9) & 7) as usize;
                let si = (op & 7) as usize;
                self.regs[di + AREG] -= self.regs[si + AREG];
                println!("{:08x}: suba.l A{:?}, A{:?}", startadr, si, di);
            },
            Opcode::Bsr => {
                let mut ofs = ((op & 0x00ff) as i8) as i16;
                if ofs == 0 {
                    ofs = self.read16(self.pc) as i16;
                    self.pc += 2;
                }
                self.push32(self.pc);
                self.pc = ((startadr + 2) as i32 + ofs as i32) as u32;
                println!("{:08x}: bsr ${:06x}", startadr, self.pc);
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

    fn read_source32(&mut self, src: usize, m: usize) -> (u32, String) {
        match src {
            0 => {  // move.l Dm, xx
                (self.regs[m + DREG], String::from(format!("D{}", m)))
            },
            7 => {  // Misc.
                match m {
                    4 => {  // move.l #$XXXX, xx
                        let value = self.read32(self.pc);
                        self.pc += 4;
                        (value, String::from(format!("#${:08x}", value)))
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

    fn write_destination32(&mut self, dst: usize, n: usize, value: u32) -> String {
        match dst {
            0 => {
                self.regs[n + DREG] = value;
                String::from(format!("D{}", n))
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
