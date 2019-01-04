use super::types::{Byte, Word, Long, Adr};

pub struct Bus {
    pub mem: Vec<Byte>,
    pub sram: Vec<Byte>,
    pub ipl: Vec<Byte>,
}

impl Bus {
    pub fn read8(&self, adr: Adr) -> Byte {
        if /*0x000000 <= adr &&*/ adr <= 0xffff {
            self.mem[adr as usize]
        } else if 0xed0000 <= adr && adr <= 0xed3fff {
            self.sram[(adr - 0xed0000) as usize]
        } else if 0xfe0000 <= adr && adr <= 0xffffff {
            self.ipl[(adr - 0xfe0000) as usize]
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }

    pub(crate) fn read16(&self, adr: Adr) -> Word {
        let d0 = self.read8(adr) as Word;
        let d1 = self.read8(adr + 1) as Word;
        (d0 << 8) | d1
    }

    pub(crate) fn read32(&self, adr: Adr) -> Long {
        let d0 = self.read8(adr) as Long;
        let d1 = self.read8(adr + 1) as Long;
        let d2 = self.read8(adr + 2) as Long;
        let d3 = self.read8(adr + 3) as Long;
        (d0 << 24) | (d1 << 16) | (d2 << 8) | d3
    }

    pub fn write8(&mut self, adr: Adr, value: Byte) {
        if /*0x000000 <= adr &&*/ adr <= 0xffff {
            self.mem[adr as usize] = value;
        } else if 0xed0000 <= adr && adr <= 0xed3fff {
            self.sram[(adr - 0xed0000) as usize] = value;
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }

    pub fn write32(&mut self, adr: Adr, value: Long) {
        self.write8(adr,     (value >> 24) as Byte);
        self.write8(adr + 1, (value >> 16) as Byte);
        self.write8(adr + 2, (value >>  8) as Byte);
        self.write8(adr + 3,  value        as Byte);
    }

    pub fn dump_mem(&self, adr: Adr, sz: usize, max: usize) -> String {
        let arr = (0..max).map(|i| {
            if i * 2 < sz {
                format!("{:04x}", self.read16(adr + (i as u32) * 2))
            } else {
                String::from("    ")
            }
        });
        arr.collect::<Vec<String>>().join(" ")
    }
}
