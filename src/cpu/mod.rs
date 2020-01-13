use std::fmt;

use crate::mem;
use std::fmt::{Formatter, Error};

mod opcode;

pub const REG_SP: usize = 13;
pub const REG_LR: usize = 14;
pub const REG_PC: usize = 15;

const MODE_SYS: usize = 0;
const MODE_FIQ: usize = 1;
const MODE_SVC: usize = 2;
const MODE_ABT: usize = 4;
const MODE_IRQ: usize = 5;
const MODE_UND: usize = 6;

pub struct ARM7TDMI {
    state: CPUState,
}

impl ARM7TDMI {
    pub fn new() -> ARM7TDMI {
        let mut cpu = ARM7TDMI {
            state: CPUState::new(),
        };
        cpu.state.reset();
        cpu
    }

    pub fn step(&mut self, m: &mem::Memory) {
        // Get the op at the current pc
        match m.read(self.get_reg(REG_PC), opcode::OP_SIZE) {
            Some(data) => match opcode::Op::parse(&data) {
                Some(op) => self.exec_op(m, &op),
                None => println!("no opcode found"),
            }
            None => println!("no data"),
        }
    }

    fn exec_op(&mut self, m: &mem::Memory, op: &opcode::Op) {
        match op {
            opcode::Op::B(offset) => {
                let old_pc = self.get_reg(REG_PC);

                let next_pc = (old_pc as i32 + *offset) as u32;
                self.set_reg(REG_PC, next_pc);
            }
            opcode::Op::BL(offset) => {
                let old_pc = self.get_reg(REG_PC);
                self.set_reg(REG_LR, old_pc);

                let next_pc = (old_pc as i32 + *offset) as u32;
                self.set_reg(REG_PC, next_pc);
            }
            _ => println!("op not implemented: {}", op)
        }
    }

    fn state(&self) -> CPUState {
        self.state.clone()
    }

    pub fn reset(&mut self) {
        self.state.reset()
    }

    pub fn get_reg(&self, reg: usize) -> u32 {
        self.state.get_reg(reg)
    }
    pub fn set_reg(&mut self, reg: usize, val: u32) {
        self.state.set_reg(reg, val)
    }
}

impl fmt::Debug for ARM7TDMI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.state.fmt(f)
    }
}

impl fmt::Display for ARM7TDMI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

#[derive(PartialEq, Copy, Clone)]
struct CPUState {
    mode: usize,
    gpreg: [u32; 8],
    regbank: [[u32; 16]; 7],
    cpsr: u32,
    spsr: [u32; 7],

}

impl CPUState {
    fn new() -> CPUState {
        CPUState {
            mode: MODE_SYS,
            gpreg: [0; 8],
            regbank: [
                [0; 16],
                [0; 16],
                [0; 16],
                [0; 16],
                [0; 16],
                [0; 16],
                [0; 16],
            ],
            cpsr: 0,
            spsr: [0; 7],
        }
    }

    pub fn reset(&mut self) {
        self.mode = MODE_SYS;
        self.cpsr = 0;
        for idx in 0..8 {
            self.gpreg[idx] = 0;
        }

        self.reset_mode(MODE_SYS);
        self.reset_mode(MODE_FIQ);
        self.reset_mode(MODE_SVC);
        self.reset_mode(MODE_ABT);
        self.reset_mode(MODE_IRQ);
        self.reset_mode(MODE_UND);
    }

    pub fn set_reg(&mut self, reg: usize, val: u32) {
        match reg {
            r if r < 8 => self.gpreg[r] = val,
            r => match r {
                13 | 14 => self.regbank[self.mode][r] = val,
                15 => self.regbank[MODE_SYS][15] = val,
                r => match self.mode {
                    MODE_FIQ => self.regbank[self.mode][r] = val,
                    _ => self.regbank[MODE_SYS][r] = val,
                },
            }
        }
    }

    pub fn get_reg(&self, reg: usize) -> u32 {
        match reg {
            r if r < 8 => self.gpreg[r],
            _ => match reg {
                13 | 14 => self.regbank[self.mode][reg],
                15 => self.regbank[MODE_SYS][15],
                _ => match self.mode {
                    MODE_FIQ => self.regbank[self.mode][reg],
                    _ => self.regbank[MODE_SYS][reg],
                }
            }
        }
    }

    fn set_mode(&mut self, mode: usize) {
        self.mode = mode;
    }

    fn reset_mode(&mut self, mode: usize) {
        self.spsr[mode] = 0;
        self.regbank[mode] = [0; 16];
    }

    fn set_cpsr(&mut self, val: u32) {
        self.cpsr = val;
    }

    fn get_spsr(&self) -> u32 {
        match self.mode {
            MODE_SYS => panic!("get spsr with mode sys"),
            _ => self.spsr[self.mode],
        }
    }
    fn set_spsr(&mut self, val: u32) {
        match self.mode {
            MODE_SYS => panic!("set spsr with mode sys"),
            _ => self.spsr[self.mode] = val,
        }
    }
}

impl fmt::Debug for CPUState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\
r0: {:#x}\t\tr4: {:#x}
r1: {:#x}\t\tr5: {:#x}
r2: {:#x}\t\tr6: {:#x}
r3: {:#x}\t\tr7: {:#x}
r8: {:#x}\t\tr8_fiq: {:#x}
r9: {:#x}\t\tr9_fiq: {:#x}
r10: {:#x}\tr10_fiq: {:#x}
r11: {:#x}\tr11_fiq: {:#x}
r12: {:#x}\tr12_fiq: {:#x}
r13: {:#x}\tr13_fiq: {:#x}\tr13_svc: {:#x}\tr13_abt: {:#x}\tr13_irq: {:#x}\tr13_und: {:#x}
r14: {:#x}\tr14_fiq: {:#x}\tr14_svc: {:#x}\tr14_abt: {:#x}\tr14_irq: {:#x}\tr14_und: {:#x}
r15: {:#x}\
        ",
               self.gpreg[0], self.gpreg[4],
               self.gpreg[1], self.gpreg[5],
               self.gpreg[2], self.gpreg[6],
               self.gpreg[3], self.gpreg[7],
               self.regbank[MODE_SYS][8], self.regbank[MODE_FIQ][8],
               self.regbank[MODE_SYS][9], self.regbank[MODE_FIQ][9],
               self.regbank[MODE_SYS][10], self.regbank[MODE_FIQ][10],
               self.regbank[MODE_SYS][11], self.regbank[MODE_FIQ][11],
               self.regbank[MODE_SYS][12], self.regbank[MODE_FIQ][12],
               self.regbank[MODE_SYS][13], self.regbank[MODE_FIQ][13],
               self.regbank[MODE_SVC][13], self.regbank[MODE_ABT][13],
               self.regbank[MODE_IRQ][13], self.regbank[MODE_UND][13],
               self.regbank[MODE_SYS][14], self.regbank[MODE_FIQ][14],
               self.regbank[MODE_SVC][14], self.regbank[MODE_ABT][14],
               self.regbank[MODE_IRQ][14], self.regbank[MODE_UND][14],
               self.regbank[MODE_SYS][15],
        )
    }
}

#[cfg(test)]
mod tests {
    mod cpu_state {
        use super::super::*;

        #[test]
        fn get_r_sys() {
            let mut state = CPUState::new();
            state.set_mode(MODE_SYS);

            state.gpreg = [1234; 8];
            state.regbank[MODE_SYS] = [1234; 16];

            for reg in 0..16 {
                assert_eq!(1234, state.get_reg(reg));
            }
        }

        #[test]
        fn get_r_fiq() {
            let mut state = CPUState::new();
            state.set_mode(MODE_FIQ);

            state.gpreg = [1234; 8];
            state.regbank[MODE_SYS] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1234];
            state.regbank[MODE_FIQ] = [0, 0, 0, 0, 0, 0, 0, 0, 1234, 1234, 1234, 1234, 1234, 1234, 1234, 0];

            for reg in 0..16 {
                assert_eq!(1234, state.get_reg(reg));
            }
        }

        #[test]
        fn get_r_svc() {
            let mut state = CPUState::new();
            state.set_mode(MODE_SVC);

            state.gpreg = [1234; 8];
            state.regbank[MODE_SYS] = [0, 0, 0, 0, 0, 0, 0, 0, 1234, 1234, 1234, 1234, 1234, 0, 0, 1234];
            state.regbank[MODE_SVC] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1234, 1234, 0];

            for reg in 0..16 {
                assert_eq!(1234, state.get_reg(reg));
            }
        }

        #[test]
        fn get_r_abt() {
            let mut state = CPUState::new();
            state.set_mode(MODE_ABT);

            state.gpreg = [1234; 8];
            state.regbank[MODE_SYS] = [0, 0, 0, 0, 0, 0, 0, 0, 1234, 1234, 1234, 1234, 1234, 0, 0, 1234];
            state.regbank[MODE_ABT] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1234, 1234, 0];

            for reg in 0..16 {
                assert_eq!(1234, state.get_reg(reg));
            }
        }

        #[test]
        fn get_r_irq() {
            let mut state = CPUState::new();
            state.set_mode(MODE_IRQ);

            state.gpreg = [1234; 8];
            state.regbank[MODE_SYS] = [0, 0, 0, 0, 0, 0, 0, 0, 1234, 1234, 1234, 1234, 1234, 0, 0, 1234];
            state.regbank[MODE_IRQ] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1234, 1234, 0];

            for reg in 0..16 {
                assert_eq!(1234, state.get_reg(reg));
            }
        }

        #[test]
        fn get_r_und() {
            let mut state = CPUState::new();
            state.set_mode(MODE_UND);

            state.gpreg = [1234; 8];
            state.regbank[MODE_SYS] = [0, 0, 0, 0, 0, 0, 0, 0, 1234, 1234, 1234, 1234, 1234, 0, 0, 1234];
            state.regbank[MODE_UND] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1234, 1234, 0];

            for reg in 0..16 {
                assert_eq!(1234, state.get_reg(reg));
            }
        }

        #[test]
        fn get_spsr_fiq() {
            let mut state = CPUState::new();
            state.set_mode(MODE_FIQ);

            state.spsr[MODE_FIQ] = 1234;

            assert_eq!(1234, state.get_spsr());
        }

        #[test]
        fn get_spsr_svc() {
            let mut state = CPUState::new();
            state.set_mode(MODE_SVC);

            state.spsr[MODE_SVC] = 1234;

            assert_eq!(1234, state.get_spsr());
        }

        #[test]
        fn get_spsr_abt() {
            let mut state = CPUState::new();
            state.set_mode(MODE_ABT);

            state.spsr[MODE_ABT] = 1234;

            assert_eq!(1234, state.get_spsr());
        }

        #[test]
        fn get_spsr_irq() {
            let mut state = CPUState::new();
            state.set_mode(MODE_IRQ);

            state.spsr[MODE_IRQ] = 1234;

            assert_eq!(1234, state.get_spsr());
        }

        #[test]
        fn get_spsr_und() {
            let mut state = CPUState::new();
            state.set_mode(MODE_UND);

            state.spsr[MODE_UND] = 1234;

            assert_eq!(1234, state.get_spsr());
        }

        #[test]
        fn set_r_sys() {
            let mut state = CPUState::new();
            state.set_mode(MODE_SYS);

            for reg in 0..16 {
                state.set_reg(reg, 1234);
                match reg {
                    0..=7 => assert_eq!(1234, state.gpreg[reg]),
                    _ => assert_eq!(1234, state.regbank[MODE_SYS][reg]),
                }
            }
        }

        #[test]
        fn set_r_fiq() {
            let mut state = CPUState::new();
            state.set_mode(MODE_FIQ);

            for reg in 0..16 {
                state.set_reg(reg, 1234);

                match reg {
                    0..=7 => assert_eq!(1234, state.gpreg[reg]),
                    _ => match reg {
                        8..=14 => assert_eq!(1234, state.regbank[MODE_FIQ][reg]),
                        _ => assert_eq!(1234, state.regbank[MODE_SYS][reg]),
                    }
                }
            }
        }

        #[test]
        fn set_r_svc() {
            let mut state = CPUState::new();
            state.set_mode(MODE_SVC);

            for reg in 0..16 {
                state.set_reg(reg, 1234);

                match reg {
                    0..=7 => assert_eq!(1234, state.gpreg[reg]),
                    _ => match reg {
                        8..=12 => assert_eq!(1234, state.regbank[MODE_SYS][reg]),
                        13..=14 => assert_eq!(1234, state.regbank[MODE_SVC][reg]),
                        _15 => assert_eq!(1234, state.regbank[MODE_SYS][reg]),
                    }
                }
            }
        }

        #[test]
        fn set_r_abt() {
            let mut state = CPUState::new();
            state.set_mode(MODE_ABT);

            for reg in 0..16 {
                state.set_reg(reg, 1234);

                match reg {
                    0..=7 => assert_eq!(1234, state.gpreg[reg]),
                    _ => match reg {
                        8..=12 => assert_eq!(1234, state.regbank[MODE_SYS][reg]),
                        13..=14 => assert_eq!(1234, state.regbank[MODE_ABT][reg]),
                        _15 => assert_eq!(1234, state.regbank[MODE_SYS][reg]),
                    }
                }
            }
        }

        #[test]
        fn set_r_irq() {
            let mut state = CPUState::new();
            state.set_mode(MODE_IRQ);

            for reg in 0..16 {
                state.set_reg(reg, 1234);

                match reg {
                    0..=7 => assert_eq!(1234, state.gpreg[reg]),
                    _ => match reg {
                        8..=12 => assert_eq!(1234, state.regbank[MODE_SYS][reg]),
                        13..=14 => assert_eq!(1234, state.regbank[MODE_IRQ][reg]),
                        _15 => assert_eq!(1234, state.regbank[MODE_SYS][reg]),
                    }
                }
            }
        }

        #[test]
        fn set_r_und() {
            let mut state = CPUState::new();
            state.set_mode(MODE_UND);

            for reg in 0..16 {
                state.set_reg(reg, 1234);

                match reg {
                    0..=7 => assert_eq!(1234, state.gpreg[reg]),
                    _ => match reg {
                        8..=12 => assert_eq!(1234, state.regbank[MODE_SYS][reg]),
                        13..=14 => assert_eq!(1234, state.regbank[MODE_UND][reg]),
                        _15 => assert_eq!(1234, state.regbank[MODE_SYS][reg]),
                    }
                }
            }
        }

        #[test]
        fn set_spsr_fiq() {
            let mut state = CPUState::new();
            state.set_mode(MODE_FIQ);

            state.set_spsr(1234);
            assert_eq!(1234, state.spsr[MODE_FIQ]);
        }

        #[test]
        fn set_spsr_svc() {
            let mut state = CPUState::new();
            state.set_mode(MODE_SVC);

            state.set_spsr(1234);
            assert_eq!(1234, state.spsr[MODE_SVC]);
        }

        #[test]
        fn set_spsr_abt() {
            let mut state = CPUState::new();
            state.set_mode(MODE_ABT);

            state.set_spsr(1234);
            assert_eq!(1234, state.spsr[MODE_ABT]);
        }

        #[test]
        fn set_spsr_irq() {
            let mut state = CPUState::new();
            state.set_mode(MODE_IRQ);

            state.set_spsr(1234);
            assert_eq!(1234, state.spsr[MODE_IRQ]);
        }

        #[test]
        fn set_spsr_und() {
            let mut state = CPUState::new();
            state.set_mode(MODE_UND);

            state.set_spsr(1234);
            assert_eq!(1234, state.spsr[MODE_UND]);
        }
    }

    mod cpu {
        mod b {
            use super::super::super::*;

            #[test]
            fn exec_offset_positive() {
                let mut cpu = ARM7TDMI::new();
                cpu.set_reg(REG_PC, 0x50_00);

                let old_state = cpu.state();
                assert_eq!(0x50_00, cpu.get_reg(REG_PC));
                assert_eq!(0, cpu.get_reg(REG_LR));

                cpu.exec_op(&mem::Memory::new(), &opcode::Op::B(0x32));

                let new_state = cpu.state();
                assert_eq!(0x50_32, cpu.get_reg(REG_PC));
                assert_eq!(0, cpu.get_reg(REG_LR));
            }

            #[test]
            fn exec_offset_negative() {
                let mut cpu = ARM7TDMI::new();
                cpu.set_reg(REG_PC, 0x50_00);

                let old_state = cpu.state();
                assert_eq!(0x50_00, cpu.get_reg(REG_PC));
                assert_eq!(0, cpu.get_reg(REG_LR));

                cpu.exec_op(&mem::Memory::new(), &opcode::Op::B(-0x32));

                let new_state = cpu.state();
                assert_eq!(0x4F_CE, cpu.get_reg(REG_PC));
                assert_eq!(0, cpu.get_reg(REG_LR));
            }
        }

        mod bl {
            use super::super::super::*;

            #[test]
            fn exec_offset_positive() {
                let mut cpu = ARM7TDMI::new();
                cpu.set_reg(REG_PC, 0x50_00);

                let old_state = cpu.state();
                assert_eq!(0x50_00, cpu.get_reg(REG_PC));
                assert_eq!(0, cpu.get_reg(REG_LR));

                cpu.exec_op(&mem::Memory::new(), &opcode::Op::BL(0x32));

                let new_state = cpu.state();
                assert_eq!(0x50_32, cpu.get_reg(REG_PC));
                assert_eq!(0x50_00, cpu.get_reg(REG_LR));
            }

            #[test]
            fn exec_offset_negative() {
                let mut cpu = ARM7TDMI::new();
                cpu.set_reg(REG_PC, 0x50_00);

                let old_state = cpu.state();
                assert_eq!(0x50_00, cpu.get_reg(REG_PC));
                assert_eq!(0, cpu.get_reg(REG_LR));

                cpu.exec_op(&mem::Memory::new(), &opcode::Op::BL(-0x32));

                let new_state = cpu.state();
                assert_eq!(0x4F_CE, cpu.get_reg(REG_PC));
                assert_eq!(0x50_00, cpu.get_reg(REG_LR));
            }
        }
    }
}