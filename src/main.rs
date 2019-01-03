use std::fs;

fn main() {
    let res = fs::read("X68BIOS/IPLROM.DAT");
    match res {
        Result::Ok(data) => {
            println!("len={:?}", data.len());
            println!("[0,1,2,3]={:?},{:?},{:?},{:?}", data[0x10000], data[0x10001], data[0x10002], data[0x10003]);
        },
        Result::Err(err) => {
            panic!(err);
        }
    }
}
