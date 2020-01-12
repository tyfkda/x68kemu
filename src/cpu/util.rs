use super::bus_trait::{BusTrait};
use super::super::types::{Word, SByte, SWord, SLong, Adr};

pub fn get_branch_offset<BusT: BusTrait>(op: Word, bus: &mut BusT, adr: Adr) -> (SLong, u32) {
    let ofs = op & 0x00ff;
    match ofs {
        0 => {
            (bus.read16(adr) as SWord as SLong, 2)
        },
        0xff => {
            (bus.read32(adr) as SLong, 4)
        },
        _ => {
            (ofs as SByte as SWord as SLong , 0)
        },
    }
}

// Return 0~7 => 8,1~7
pub fn conv07to18(x: Word) -> Word {
    ((x & 7).wrapping_sub(1) & 7) + 1
}

#[test]
fn test_conv07to18() {
    assert_eq!(8, conv07to18(0));
    assert_eq!(1, conv07to18(1));
    assert_eq!(7, conv07to18(7));
}
