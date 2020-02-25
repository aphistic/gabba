use std::fmt;
use std::process::id;

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

const MASK_SIGNED16: i32 = 0x8000;
const MASK_SIGNED24: i32 = 0x800000;

const OP_B: u32 = 0x0A000000;
const MASK_B: u32 = 0x0E000000;
const MASK_B_L: u32 = 0x01000000;

const OP_MOV: u32 = 0x03A00000;
const MASK_MOV: u32 = 0x0BE00000;
const MASK_MOV_SHIFTER: u32 = 0x00000fff;
const MASK_MOV_R: u32 = 0x0000f000;

const OP_MSR: u32 = 0x01200000;
const MASK_MSR: u32 = 0x0DB00000;
const MASK_MSR_R: u32 = 0x00400000;
const MASK_MSR_25: u32 = 0x02000000;
const MASK_MSR_IMMEDIATE: u32 = 0x000000ff;
const MASK_MSR_ROTATE: u32 = 0x00000f00;
const MASK_MSR_FIELD_MASK: u32 = 0x000f0000;

#[derive(Debug, PartialEq)]
pub enum Op {
    // Branch
    B(i32),
    Bl(i32),
    Bx,
    Blx,
    Mov(usize, i32),
    Msr(bool, bool, u8, usize),
    Swi,
    Bkpt,
}

impl Op {
    pub fn parse(data: &[u8]) -> Option<Op> {
        let mut opbytes = [0; 4];
        opbytes.copy_from_slice(data[0..4].as_ref());

        let opdata = u32::from_le_bytes(opbytes);
        println!("op:\t{:032b}\t{:08x}", opdata, opdata);
        println!("mask:\t{:032b}", MASK_MSR);
        println!("msr:\t{:032b} {:0x}", opdata & MASK_MSR, opdata & MASK_MSR);
        if opdata & MASK_B == OP_B {
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
                1 => Some(Op::Bl(offset)),
                _ => Some(Op::Blx),
            }
        } else if opdata & MASK_MOV == OP_MOV {
            println!("maskmov: {:032b} {:0x}", opdata & MASK_MOV, opdata & MASK_MOV);
            match opdata & MASK_MOV {
                OP_MOV => {
                    println!("op mov");
                    let r = (opdata & MASK_MOV_R) as usize;
                    let shifter_operand = (opdata & MASK_MOV_SHIFTER) as i32;

                    Some(Op::Mov(r, shifter_operand))
                }
                _ => None,
            }
        } else if opdata & MASK_MSR == OP_MSR {
            let r = opdata & MASK_MSR_R == MASK_MSR_R;
            let field_mask = ((opdata & MASK_MSR_FIELD_MASK) >> 16) as u8;

            match opdata & MASK_MSR_25 {
                MASK_MSR_25 => {
                    let immediate = opdata & MASK_MSR_IMMEDIATE;
                    let rotate_imm = (opdata & MASK_MSR_ROTATE) >> 8;

                    Some(Op::Msr(true, r, field_mask, immediate as usize))
                }
                _ => Some(Op::Msr(false, r, field_mask, (opdata & MASK_MSR_IMMEDIATE) as usize))
            }
        } else {
            None
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

    mod mov {
        use super::super::*;

        #[test]
        fn parse() {
            assert_eq!(
                Some(Op::Mov(0, 0x12)),
                Op::parse(&vec![0x12, 0x00, 0xA0, 0xE3])
            )
        }
    }

    mod msr {
        use super::super::*;

        #[test]
        fn parse_register() {
            assert_eq!(
                Some(Op::Msr(false, false, 0x09, 0)),
                Op::parse(&vec![0x00, 0xF8, 0x29, 0xE1]),
            )
        }
    }
}