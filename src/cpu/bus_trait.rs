use super::super::types::{Byte, Word, Long, Adr};

pub trait BusTrait {
    fn reset(&mut self) {}
    fn read8(&self, adr: Adr) -> Byte;
    fn write8(&mut self, adr: Adr, value: Byte);

    fn read16(&self, adr: Adr) -> Word {
        let d0 = self.read8(adr) as Word;
        let d1 = self.read8(adr + 1) as Word;
        (d0 << 8) | d1
    }

    fn read32(&self, adr: Adr) -> Long {
        let d0 = self.read8(adr) as Long;
        let d1 = self.read8(adr + 1) as Long;
        let d2 = self.read8(adr + 2) as Long;
        let d3 = self.read8(adr + 3) as Long;
        (d0 << 24) | (d1 << 16) | (d2 << 8) | d3
    }

    fn write16(&mut self, adr: Adr, value: Word) {
        self.write8(adr    , (value >>  8) as Byte);
        self.write8(adr + 1,  value        as Byte);
    }

    fn write32(&mut self, adr: Adr, value: Long) {
        self.write8(adr,     (value >> 24) as Byte);
        self.write8(adr + 1, (value >> 16) as Byte);
        self.write8(adr + 2, (value >>  8) as Byte);
        self.write8(adr + 3,  value        as Byte);
    }
}
