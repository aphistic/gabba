use std::fmt;

pub const OP_SIZE: usize = 4;

const COND_EQ: u8 = 0b0000;
const COND_NE: u8 = 0b0001;
const COND_CSHS: u8 = 0b0010;
const COND_CCLO: u8 = 0b0011;
const COND_MI: u8 = 0b0100;
const COND_PL: u8 = 0b0101;
const COND_VS: u8 = 0b0110;
const COND_VC: u8 = 0b0111;
const COND_HI: u8 = 0b1000;
const COND_LS: u8 = 0b1001;
const COND_GE: u8 = 0b1010;
const COND_LT: u8 = 0b1011;
const COND_GT: u8 = 0b1100;
const COND_LE: u8 = 0b1101;
const COND_AL: u8 = 0b1110;
// Always
const COND_UNDEF: u8 = 0b1111;

const MASK_OP3: u32 = 0x0E000000;
const MASK_SIGNED16: i32 = 0x8000;
const MASK_SIGNED24: i32 = 0x800000;
const MASK_B_L: u32 = 0x01000000;

const OP_B: u32 = 0x0A000000;

#[derive(Debug, PartialEq)]
pub enum Op {
    // Branch
    B(i32),
    BL(i32),
    BX,
    BLX,
    SWI,
    BKPT,
}

impl Op {
    pub fn parse(data: &[u8]) -> Option<Op> {
        let mut opbytes = [0; 4];
        opbytes.copy_from_slice(data[0..4].as_ref());

        let opdata = u32::from_le_bytes(opbytes);
        println!("op:\t{:032b}\t{:08x}", opdata, opdata);
        println!("op3:\t{:032b}\t{:08x}", opdata & MASK_OP3, opdata & MASK_OP3);
        match opdata & MASK_OP3 {
            OP_B => {
                let mut offset = opdata as i32 & 0x00FFFFFF;
                // TODO This is probably really wrong (it should be 24 instead of 16)
                //      but I don't have a good example of a real negative offset yet.
                println!("offset:\t{:032b}", offset);
                println!("mask:\t{:032b}", MASK_SIGNED24);
                println!("xor:\t{:032b}", offset ^ MASK_SIGNED24);
                let offset = match offset & MASK_SIGNED24 {
                    MASK_SIGNED24 => panic!("negative branch"), // ((offset ^ MASK_SIGNED24 << 2) * -1) + 8,
                    _ => (offset << 2) + 8,
                };
                println!("after:\t{:032b}\t{:0x}", offset, offset);

                match opdata & MASK_B_L {
                    0 => Some(Op::B(offset)),
                    1 => Some(Op::BL(offset)),
                    _ => Some(Op::BLX),
                }
            }
            _ => None
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::B(addr) => write!(f, "B {:#x}", addr),
            _ => write!(f, "unsupported op"),
        }
    }
}

#[cfg(test)]
mod tests {
    mod b {
        use super::super::*;

        #[test]
        fn parse_positive_offset() {
            assert_eq!(
                Some(Op::B(0xd0)),
                Op::parse(&vec![0x32, 0x00, 0x00, 0xEA]),
            );
        }

        #[test]
        fn parse_negative_offset() {
            // TODO I don't have a good example of this in practice yet
//            assert_eq!(
//                Some(Op::B(-0xd0)),
//                Op::parse(&vec![0x32, 0x80, 0x00, 0xEA]),
//            );
        }
    }
}