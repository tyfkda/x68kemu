use lazy_static::lazy_static;

#[derive(Clone)]
enum Opcode {
    Unknown,
    MoveToSrIm,
}

#[derive(Clone)]
struct Inst {
    op: Opcode,
}

lazy_static! {
    static ref INST: Vec<Inst> = {
        let mut m = vec![Inst {op: Opcode::Unknown}; 0x10000];
        m[0x46fc] = Inst {op: Opcode::MoveToSrIm};
        m
    };
}

const AREG: usize = 8;
const SP: usize = 7 + AREG;  // Stack pointer = A7 register.

pub struct Cpu {
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

    pub fn step(&mut self) {
        let startadr = self.pc;
        let opcode = self.read16(self.pc);
        self.pc += 2;
        let inst = &INST[opcode as usize];

        match inst.op {
            Opcode::MoveToSrIm => {
                self.sr = self.read16(self.pc);
                self.pc += 2;
                println!("{:08x}: move #${:04x}, SR", startadr, self.sr);
            },
            _ => {
                println!("{:08x}: {:04x}  -- Unknown opcode:", startadr, opcode);
            },
        }
    }

    fn read8(&self, adr: u32) -> u8 {
        if 0xfe0000 <= adr && adr <= 0xffffff {
            self.ipl[(adr - 0xfe0000) as usize]
        } else {
            panic!("Illegal address");
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
}

pub fn new_cpu(ipl: Vec<u8>) -> Cpu {
    let mut cpu = Cpu{ipl: ipl, regs: vec![0; 8 + 8], pc: 0, sr: 0};
    cpu.reset();
    cpu
}
