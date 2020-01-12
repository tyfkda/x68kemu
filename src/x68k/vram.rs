use super::super::types::{Byte, Adr};

const GRAPHIC_SIZE: usize = 0x200000;
const TEXT_SIZE: usize    =  0x80000;

pub struct Vram {
    // 0xc00000~0xdfffff
    graphic: Box<[Byte; GRAPHIC_SIZE]>,
    // 0xe00000~0xe7ffff
    text: Box<[Byte; TEXT_SIZE]>,
}

impl Vram {
    pub fn new() -> Self {
        Self {
            graphic: Box::new([0; GRAPHIC_SIZE]),
            text: Box::new([0; TEXT_SIZE]),
        }
    }

    pub fn read_graphic(&self, adr: Adr) -> Byte {
        self.graphic[adr as usize]
    }

    pub fn read_text(&self, adr: Adr) -> Byte {
        self.text[adr as usize]
    }

    pub fn write_graphic(&mut self, adr: Adr, value: Byte) {
        self.graphic[adr as usize] = value;
    }

    pub fn write_text(&mut self, adr: Adr, value: Byte) {
        self.text[adr as usize] = value;
    }
}
