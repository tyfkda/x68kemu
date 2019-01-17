use std::fs;

mod app;
mod cpu;
mod types;
mod x68k;

use self::app::{App};

fn main() {
    match fs::read("X68BIOSE/IPLROM.DAT") {
        Result::Ok(ipl) => {
            let mut app = App::new(ipl);
            app.run();
        },
        Result::Err(err) => {
            panic!(err);
        }
    }
}
