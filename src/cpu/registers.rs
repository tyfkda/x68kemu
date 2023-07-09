use super::super::types::{Word, Long, Adr};

#[derive (Default)]
pub struct Registers {
    pub a: [Adr; 8],  // Address registers
    pub d: [Long; 8],  // Data registers
    pub pc: Adr,
    pub sr: Word,
}

impl Registers {
    pub fn new() -> Self {
        Self::default()
    }
}
