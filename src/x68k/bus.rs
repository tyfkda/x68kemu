use std::cell::Cell;

use super::vram::Vram;
use super::super::cpu::BusTrait;
use super::super::types::{Byte, Adr};

const RAM_SIZE: usize = 0x200000;
const SRAM_SIZE: usize = 0x4000;

pub struct Bus {
    mem: Vec<Byte>,
    sram: Vec<Byte>,
    ipl: Vec<Byte>,
    booting: Cell<bool>,
    vram: Vram,
}

impl BusTrait for Bus {
    fn reset(&mut self) {
        self.booting = true.into();
    }

    fn read8(&self, adr: Adr) -> Byte {
        if /*0x000000 <= adr &&*/ adr < RAM_SIZE as Adr {
            if self.booting.get() {
                self.ipl[(adr + 0x10000) as usize]
            } else {
                self.mem[adr as usize]
            }
        } else if (0xc00000..=0xdfffff).contains(&adr) {  // Graphic RAM
            return self.vram.read_graphic(adr - 0xc00000);
        } else if (0xe00000..=0xe7ffff).contains(&adr) {  // TEXT RAM
            return self.vram.read_text(adr - 0xe00000);
        } else if (0xe80000..=0xe80030).contains(&adr) {  // CRTC
            // TODO: Implement.
            return 0;
        } else if (0xe88000..=0xe89fff).contains(&adr) {  // MFP
            // TODO: Implement.
            match adr {
                0xe8802d => 0x80,  // Transmittance Status Register.
                _ => 0,
            }
        } else if (0xe8e000..=0xe8ffff).contains(&adr) {  // I/O port
            // TODO: Implement.
            0
        } else if (0xe94000..=0xe94fff).contains(&adr) {  // Floppy Disk Controller
            // TODO: Implement.
            match adr {
                0xe94001 => {
                    0xd0  // RQM: Request for Master
                },
                _ => {
                    0
                },
            }
        } else if (0xe96000..=0xe96fff).contains(&adr) {  // SASI
            0
        } else if (0xe9c000..=0xe9cfff).contains(&adr) {  // I/O Controller
            // TODO: Implement.
            0
        } else if (0xed0000..0xed0000 + (SRAM_SIZE as Adr)).contains(&adr) {
            self.sram[(adr - 0xed0000) as usize]
        } else if (0xfe0000..=0xffffff).contains(&adr) {
            if adr >= 0xff0000 {
                self.booting.set(false);
            }
            self.ipl[(adr - 0xfe0000) as usize]
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }

    fn write8(&mut self, adr: Adr, value: Byte) {
        if /*0x000000 <= adr &&*/ adr < RAM_SIZE as Adr {
            self.mem[adr as usize] = value;
        } else if (0xc00000..=0xdfffff).contains(&adr) {  // Graphic VRAM
            self.vram.write_graphic(adr - 0xc00000, value);
        } else if (0xe00000..=0xe7ffff).contains(&adr) {  // TEXT VRAM
            self.vram.write_text(adr - 0xe00000, value);
        } else if (0xe80000..=0xe81fff).contains(&adr) {  // CRTC
            // TODO: Implement.
        } else if (0xe82000..=0xe83fff).contains(&adr) {  // video
            // TODO: Implement.
        } else if (0xe84000..=0xe85fff).contains(&adr) {  // DMAC
            // TODO: Implement.
        } else if (0xe86000..=0xe87fff).contains(&adr) {  // AREA set
            // TODO: Implement.
        } else if (0xe88000..=0xe89fff).contains(&adr) {  // MFP
            // TODO: Implement.
        } else if (0xe8a000..=0xe8bfff).contains(&adr) {  // Printer
            // TODO: Implement.
        } else if (0xe8c000..=0xe8dfff).contains(&adr) {  // Sys port
            // TODO: Implement.
        } else if (0xe8e000..=0xe8ffff).contains(&adr) {  // I/O port
            // TODO: Implement.
        } else if (0xe90000..=0xe91fff).contains(&adr) {  // FM Audio
            // TODO: Implement.
        } else if (0xe92000..=0xe93fff).contains(&adr) {  // ADPCM
            // TODO: Implement.
        } else if (0xe94000..=0xe95fff).contains(&adr) {  // FDC
            // TODO: Implement.
        } else if (0xe96000..=0xe97fff).contains(&adr) {  // HDD
            // TODO: Implement.
        } else if (0xe98000..=0xe99fff).contains(&adr) {  // SCC
            // TODO: Implement.
        } else if (0xe9a000..=0xe9dfff).contains(&adr) {  // i8255
            // TODO: Implement.
        } else if (0xe9e000..=0xe9ffff).contains(&adr) {  // FPU
            // TODO: Implement.
        } else if (0xe9a000..=0xeaffff).contains(&adr) {  // SCSI
            // TODO: Implement.
        } else if (0xeb0000..=0xecffff).contains(&adr) {  // Sprite
            // TODO: Implement.
        } else if (0xed0000..=0xed3fff).contains(&adr) {
            self.sram[(adr - 0xed0000) as usize] = value;
        } else if (0xed4000..=0xefffff).contains(&adr) {
            // TODO: Implement.
        } else {
            panic!("Illegal address: {:08x}", adr);
        }
    }
}

impl Bus {
    pub fn new(ipl: Vec<Byte>, vram: Vram) -> Self {
        Self {
            mem: vec![0; RAM_SIZE],
            sram: vec![0; SRAM_SIZE],
            ipl,
            booting: true.into(),
            vram,
        }
    }
}
