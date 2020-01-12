use super::super::types::{Byte, Adr};

const TEXT_SIZE: usize = 0x80000;

pub struct Vram {
    text: [Byte; TEXT_SIZE],
}

impl Vram {
    pub fn new() -> Vram {
        Vram {
            text: [0; TEXT_SIZE],
        }
    }

    pub fn write_text(&mut self, adr: Adr, value: Byte) {
        self.text[adr as usize] = value;
    }
}
