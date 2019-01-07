use super::super::cpu::{BusTrait};
use super::super::types::{Byte, Adr};

pub struct Bus {
    mem: Vec<Byte>,
    sram: Vec<Byte>,
    ipl: Vec<Byte>,
}

impl BusTrait for Bus {
    fn read8(&self, adr: Adr) -> Byte {
        if /*0x000000 <= adr &&*/ adr <= 0xffff {
            self.mem[adr as usize]
        } else if 0xe80000 <= adr && adr <= 0xe80030 {  // CRTC
            // TODO: Implement.
            return 0;
        } else if 0xed0000 <= adr && adr <= 0xed3fff {
            self.sram[(adr - 0xed0000) as usize]
        } else if 0xfe0000 <= adr && adr <= 0xffffff {
            self.ipl[(adr - 0xfe0000) as usize]
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }

    fn write8(&mut self, adr: Adr, value: Byte) {
        if /*0x000000 <= adr &&*/ adr <= 0xffff {
            self.mem[adr as usize] = value;
        } else if 0xe00000 <= adr && adr <= 0xe7ffff {  // TEXT VRAM
            // TODO: Implement.
        } else if 0xe80000 <= adr && adr <= 0xe81fff {  // CRTC
            // TODO: Implement.
        } else if 0xe82000 <= adr && adr <= 0xe83fff {  // video
            // TODO: Implement.
        } else if 0xe84000 <= adr && adr <= 0xe85fff {  // DMAC
            // TODO: Implement.
        } else if 0xe8a000 <= adr && adr <= 0xe8bfff {  // Printer
            // TODO: Implement.
        } else if 0xe8e00d == adr {  // ?
            // TODO: Implement.
        } else if 0xe9a000 <= adr && adr <= 0xe9bfff {  // i8255
            // TODO: Implement.
        } else if 0xeb0000 <= adr && adr <= 0xebffff {  // Sprite
            // TODO: Implement.
        } else if 0xed0000 <= adr && adr <= 0xed3fff {
            self.sram[(adr - 0xed0000) as usize] = value;
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }
}

impl Bus {
    pub fn new(ipl: Vec<Byte>) -> Bus {
        Bus {
            mem: vec![0; 0x10000],
            sram: vec![0; 0x4000],
            ipl: ipl,
        }
    }
}
