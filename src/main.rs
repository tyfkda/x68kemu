use std::fs;

mod cpu;
mod types;
mod x68k;

use self::x68k::{X68k};

fn main() {
    match fs::read("X68BIOSE/IPLROM.DAT") {
        Result::Ok(ipl) => {
            let mut x68k = X68k::new(ipl);
            loop {
                x68k.update(10000);
            }
        },
        Result::Err(err) => {
            panic!(err);
        }
    }
}
