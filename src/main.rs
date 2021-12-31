use std::fs;
use std::io::{ErrorKind};

mod cpu;
mod types;
mod x68k;

use self::x68k::{X68k};

const IPLROM_PATH: &str = "X68BIOSE/IPLROM.DAT";

fn main() {
    match fs::read(IPLROM_PATH) {
        Result::Ok(ipl) => {
            let mut x68k = X68k::new(ipl);
            loop {
                x68k.update(10000);
            }
        },
        Result::Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                eprintln!("Cannot load IPLROM: {}", IPLROM_PATH);
            } else {
                panic!("{}", err);
            }
        }
    }
}
