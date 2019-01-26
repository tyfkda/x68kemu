use super::super::types::{Byte, Adr};

pub struct Fdc {
    status: usize,
}

impl Fdc {
    pub fn new() -> Fdc {
        Fdc {
            status: 0,
        }
    }

    pub fn read8(&mut self, adr: Adr) -> Byte {
        println!("FDC read: adr={:x}", adr);
        match adr {
            0xe94001 => {
                let command = [0x90, 0x80, 0x80, ];  //0xd0, 0x80, 0x80, 0x80, 0xd0, ];
                if self.status < command.len() {
                    let v = command[self.status];
                    self.status += 1;
                    v
                } else {
                    0x80
                }
            },
            _ => {
                0
            },
        }
    }

    pub fn write8(&mut self, adr: Adr, value: Byte) {
        println!("FDC write: adr={:x}, value={:02x}", adr, value);
    }
}
