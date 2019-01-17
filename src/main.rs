use std::fs;
use std::io::ErrorKind;

mod app;
mod cpu;
mod types;
mod x68k;

use self::app::App;

const IPLROM_PATH: &str = "X68BIOSE/IPLROM.DAT";

fn main() {
    match fs::read(IPLROM_PATH) {
        Result::Ok(ipl) => {
            let mut app = App::new(ipl);
            app.run();
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
