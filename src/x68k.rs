const AREG: usize = 8;
const SP: usize = 7 + AREG;  // Stack pointer = A7 register.

pub struct Cpu {
    ipl: Vec<u8>,
    regs: Vec<u32>,
    pc: u32,
}

impl Cpu {
    fn reset(&mut self) {
        self.regs[SP] = self.read32(0xff0000);
        self.pc = self.read32(0xff0004);
    }

    pub fn step(&mut self) {
        let op = self.read16(self.pc);
        self.pc += 2;
        println!("PC={:08x}, OP={:04x}", self.pc, op);
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
    let mut cpu = Cpu{ipl: ipl, regs: vec![0; 8 + 8], pc: 0};
    cpu.reset();
    cpu
}
