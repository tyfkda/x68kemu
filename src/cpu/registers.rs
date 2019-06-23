use super::super::types::{Word, Long, Adr};

pub struct Registers {
    pub(crate) a: [Adr; 8],  // Address registers
    pub(crate) d: [Long; 8],  // Data registers
    pub(crate) pc: Adr,
    pub(crate) sr: Word,
}

impl Registers {
    pub fn new() -> Registers {
        let regs = Registers {
            a: [0; 8],
            d: [0; 8],
            pc: 0,
            sr: 0,
        };
        regs
    }
}
