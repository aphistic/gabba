use crate::cpu;
use crate::gamepak;
use crate::mem;

pub struct GBA {
    cpu: cpu::ARM7TDMI,

    mem: mem::Memory,
}

impl GBA {
    pub fn new() -> GBA {
        GBA {
            cpu: cpu::ARM7TDMI::new(),
            mem: mem::Memory::new(),
        }
    }

    pub fn load(&mut self, gp: gamepak::GamePak) -> Result<(), String> {
        self.mem.load_pak(gp.data());

        self.cpu.reset();
        self.cpu.set_reg(cpu::REG_PC, mem::PAK_ROM as u32);

        Ok(())
    }

    pub fn step(&mut self) {
        println!("cpu:\n{:?}", self.cpu);
        self.cpu.step(&self.mem);
        println!("cpu:\n{:?}", self.cpu);
        self.cpu.step(&self.mem);
        println!("cpu:\n{:?}", self.cpu);
    }
}