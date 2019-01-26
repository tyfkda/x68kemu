use super::super::types::{Byte, Adr};

// Command
const SPECIFY: u8 = 3;
const SENSE_INTERRUPT_STATUS: u8 = 8;

const CMD_TABLE: [usize; 32] = [
    0, 0, 8, 2, 1, 8, 8, 1, 0, 8, 1, 0, 8, 5, 0, 2,
    0, 8, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 8, 0, 0,
];

const DAT_TABLE: [usize; 32] = [
    0, 0, 7, 0, 1, 7, 7, 0, 2, 7, 7, 0, 7, 7, 0, 0,
    0, 7, 0, 0, 1, 0, 0, 0, 0, 7, 0, 0, 0, 7, 0, 0,
];

pub struct Fdc {
    #[allow(dead_code)]
    status: usize,
    ctrl: Byte,
    cmd: u8,
    bufnum: usize,
    rdnum: usize,
    rdptr: usize,
    wrnum: usize,
    wrptr: usize,
    st0: Byte,
    st1: Byte,
    st2: Byte,
    wexec: bool,
    prm_buf: [Byte; 10],
    rsp_buf: [Byte; 10],
}

impl Fdc {
    pub fn new() -> Fdc {
        Fdc {
            status: 0,
            ctrl: 0,
            cmd: 0,
            bufnum: 0,
            rdnum: 0,
            rdptr: 0,
            wrnum: 0,
            wrptr: 0,
            st0: 0,
            st1: 0,
            st2: 0,
            wexec: false,
            prm_buf: [0; 10],
            rsp_buf: [0; 10],
        }
    }

    pub fn read8(&mut self, adr: Adr) -> Byte {
        println!("FDC read: adr={:x}", adr);
        match adr {
            0xe94001 => {
/*
                let command = [0x90, 0x80, 0x80, ];  //0xd0, 0x80, 0x80, 0x80, 0xd0, ];
                if self.status < command.len() {
                    let v = command[self.status];
                    self.status += 1;
                    v
                } else {
                    0x80
                }
                 */
                let mut ret = 0x80;
                if self.rdnum > 0 && !self.wexec { ret |= 0x40; }
                if self.wrnum > 0 || self.rdnum > 0 { ret |= 0x10; }
                //if self.rdnum == 1 && self.cmd == SENSE_INTERRUPT_STATUS { ret &= 0xaf; }
println!("  => {:02x}", ret);
                ret
            },
            0xe94003 => {
                if self.bufnum > 0 {
                    panic!("not implemented");
                } else {
                    let ret = self.rsp_buf[self.rdptr];
                    self.rdptr += 1;
                    self.rdnum -= 1;
                    ret
                }
            },
            0xe94005 => {
                let mut ret = 0;
                if (self.ctrl & (1 << 0)) != 0 /* && FDD_IsReady(0) */ { ret = 0x80; }
                if (self.ctrl & (1 << 1)) != 0 /* && FDD_IsReady(1) */ { ret = 0x80; }
                ret
            },
            _ => {
                0
            },
        }
    }

    pub fn write8(&mut self, adr: Adr, value: Byte) {
        println!("FDC write: adr={:x}, value={:02x}", adr, value);
        match adr {
            0xe94003 => {
                if self.bufnum == 0 {
                    if self.wrnum == 0 {
                        self.cmd = value & 0x1f;
                        self.rdptr = 0;
                        self.wrptr = 0;
                        self.rdnum = 0;
                        self.wrnum = CMD_TABLE[self.cmd as usize];
                        self.prm_buf[self.wrptr] = value;
                        self.wrptr += 1;
println!("FDC start command={}, wrnum={}", self.cmd, self.wrnum);
                    } else {
                        self.prm_buf[self.wrptr] = value;
                        self.wrptr += 1;
                        self.wrnum -= 1;
                    }
                    if self.wrnum == 0 {
                        self.wrptr = 0;
                        self.rdnum = DAT_TABLE[self.cmd as usize];
                        self.st1 = 0;
                        self.st2 = 0;
                        if self.cmd==17 || self.cmd==25 || self.cmd==29 {
                            self.st2 |= 8;
                        }
                        self.exec_cmd();
println!("  cmd={} done, rdnum={}", self.cmd, self.rdnum);
                    }
                } else {
panic!("Unhandled!");
                }
            },
            0xe94005 => {
                self.ctrl = value;
            },
            _ => {
            },
        }
    }

    fn exec_cmd(&mut self) {
println!("FDC exec_cmd: cmd={}", self.cmd);
        match self.cmd {
            SPECIFY => {
                // Nothing to do
            },
            SENSE_INTERRUPT_STATUS => {
                //rsp.st0 = self.st0;
                //rsp.st1 = self.cyl;
                self.st0 = 0x80;
            },
            _ => {
                panic!("FDC: Unhandled command: {}", self.cmd);
            },
        }
    }
}
