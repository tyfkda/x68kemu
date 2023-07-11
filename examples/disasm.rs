use std::env;
use std::error::Error;
use std::fs;

use x68kemu::{
    cpu,
    cpu::BusTrait,
    types::{Adr, Byte},
};

struct DummyBus {
    data: Vec<Byte>,
    start_address: Adr,
}

impl DummyBus {
    fn new(data: Vec<Byte>, start_address: Adr) -> Self {
        Self {
            data,
            start_address,
        }
    }
}

impl BusTrait for DummyBus {
    fn read8(&self, adr: Adr) -> Byte {
        if (self.start_address..self.start_address + self.data.len() as Adr).contains(&adr) {
            return self.data[(adr - self.start_address) as usize];
        } else {
            panic!("Out of range: {:06x}", adr);
        }
    }

    fn write8(&mut self, adr: Adr, value: Byte) {
        if (self.start_address..self.start_address + self.data.len() as Adr).contains(&adr) {
            self.data[(adr - self.start_address) as usize] = value;
        } else {
            panic!("Out of range: {:06x}", adr);
        }
    }
}

pub struct DisasmIpl {
    bus: DummyBus,
}

impl DisasmIpl {
    pub fn new(data: Vec<Byte>, start_address: Adr) -> Self {
        let bus = DummyBus::new(data, start_address);
        Self {
            bus,
        }
    }

    pub fn disasm(&mut self, pc: Adr) -> Adr {
        let (sz, mnemonic) = cpu::disasm::disasm(&mut self.bus, pc);
        println!("{:06x}: {}  {}", pc, dump_mem(&mut self.bus, pc, sz, 5), mnemonic);
        pc + sz as Adr
    }
}

fn dump_mem<BusT: BusTrait>(bus: &mut BusT, adr: Adr, sz: usize, max: usize) -> String {
    let arr = (0..max).map(|i| {
        if i * 2 < sz {
            format!("{:04x}", bus.read16(adr + (i as u32) * 2))
        } else {
            String::from("    ")
        }
    });
    arr.collect::<Vec<String>>().join(" ")
}

fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    if args.len() < 4 {
        panic!("Usage: [romfile-path] [start-address] [pc]\n    (ex. X68BIOSE/IPLROM.DAT fe0000 ff0010)");
    }

    let filename = &args[1];
    let data = fs::read(&filename)?;

    let start_address = u32::from_str_radix(&args[2], 16)?;
    let mut pc = u32::from_str_radix(&args[3], 16)?;

    let mut dasm = DisasmIpl::new(data, start_address);
    for _ in 0..100 {
        pc = dasm.disasm(pc);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    run(&args)
}
