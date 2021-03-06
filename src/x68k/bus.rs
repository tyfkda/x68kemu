use super::super::cpu::{BusTrait};
use super::super::types::{Byte, Adr};

const RAM_SIZE: usize = 0x200000;
const SRAM_SIZE: usize = 0x4000;

pub struct Bus {
    mem: Vec<Byte>,
    sram: Vec<Byte>,
    ipl: Vec<Byte>,
    booting: bool,
}

impl BusTrait for Bus {
    fn reset(&mut self) {
        self.booting = true;
    }

    fn read8(&mut self, adr: Adr) -> Byte {
        if /*0x000000 <= adr &&*/ adr < RAM_SIZE as Adr {
            if self.booting {
                self.ipl[(adr + 0x10000) as usize]
            } else {
                self.mem[adr as usize]
            }
        } else if 0xe80000 <= adr && adr <= 0xe80030 {  // CRTC
            // TODO: Implement.
            return 0;
        } else if 0xe88000 <= adr && adr <= 0xe89fff {  // MFP
            // TODO: Implement.
            match adr {
                0xe8802d => 0x80,  // Transmittance Status Register.
                _ => 0,
            }
        } else if 0xe8e000 <= adr && adr <= 0xe8ffff {  // I/O port
            // TODO: Implement.
            0
        } else if 0xe94000 <= adr && adr <= 0xe94fff {  // Floppy Disk Controller
            // TODO: Implement.
            match adr {
                0xe94001 => {
                    0xd0  // RQM: Request for Master
                },
                _ => {
                    0
                },
            }
        } else if 0xe96000 <= adr && adr <= 0xe96fff {  // SASI
            0
        } else if 0xe9c000 <= adr && adr <= 0xe9cfff {  // I/O Controller
            // TODO: Implement.
            0
        } else if 0xed0000 <= adr && adr < 0xed0000 + (SRAM_SIZE as Adr) {
            self.sram[(adr - 0xed0000) as usize]
        } else if 0xfe0000 <= adr && adr <= 0xffffff {
            if adr >= 0xff0000 {
                self.booting = false;
            }
            self.ipl[(adr - 0xfe0000) as usize]
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }

    fn write8(&mut self, adr: Adr, value: Byte) {
        if /*0x000000 <= adr &&*/ adr < RAM_SIZE as Adr {
            self.mem[adr as usize] = value;
        } else if 0xe00000 <= adr && adr <= 0xe7ffff {  // TEXT VRAM
            // TODO: Implement.
        } else if 0xe80000 <= adr && adr <= 0xe81fff {  // CRTC
            // TODO: Implement.
        } else if 0xe82000 <= adr && adr <= 0xe83fff {  // video
            // TODO: Implement.
        } else if 0xe84000 <= adr && adr <= 0xe85fff {  // DMAC
            // TODO: Implement.
        } else if 0xe86000 <= adr && adr <= 0xe87fff {  // AREA set
            // TODO: Implement.
        } else if 0xe88000 <= adr && adr <= 0xe89fff {  // MFP
            // TODO: Implement.
        } else if 0xe8a000 <= adr && adr <= 0xe8bfff {  // Printer
            // TODO: Implement.
        } else if 0xe8c000 <= adr && adr <= 0xe8dfff {  // Sys port
            // TODO: Implement.
        } else if 0xe8e000 <= adr && adr <= 0xe8ffff {  // I/O port
            // TODO: Implement.
        } else if 0xe90000 <= adr && adr <= 0xe91fff {  // FM Audio
            // TODO: Implement.
        } else if 0xe92000 <= adr && adr <= 0xe93fff {  // ADPCM
            // TODO: Implement.
        } else if 0xe94000 <= adr && adr <= 0xe95fff {  // FDC
            // TODO: Implement.
        } else if 0xe96000 <= adr && adr <= 0xe97fff {  // HDD
            // TODO: Implement.
        } else if 0xe98000 <= adr && adr <= 0xe99fff {  // SCC
            // TODO: Implement.
        } else if 0xe9a000 <= adr && adr <= 0xe9dfff {  // i8255
            // TODO: Implement.
        } else if 0xe9e000 <= adr && adr <= 0xe9ffff {  // FPU
            // TODO: Implement.
        } else if 0xe9a000 <= adr && adr <= 0xeaffff {  // SCSI
            // TODO: Implement.
        } else if 0xeb0000 <= adr && adr <= 0xecffff {  // Sprite
            // TODO: Implement.
        } else if 0xed0000 <= adr && adr <= 0xed3fff {
            self.sram[(adr - 0xed0000) as usize] = value;
        } else if 0xed4000 <= adr && adr <= 0xefffff {
            // TODO: Implement.
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }
}

impl Bus {
    pub fn new(ipl: Vec<Byte>) -> Bus {
        Bus {
            mem: vec![0; RAM_SIZE],
            sram: vec![0; SRAM_SIZE],
            ipl: ipl,
            booting: true,
        }
    }
}
