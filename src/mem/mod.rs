use std::collections::BTreeMap;

// TODO Mirroring support in memory

const KBYTE: usize = 1024;

pub const SYS_ROM: u32 = 0x00_00_00_00;
const SYS_ROM_SIZE: usize = 16 * KBYTE;

pub const EXT_WRAM: u32 = 0x02_00_00_00;
const EXT_WRAM_SIZE: usize = 256 * KBYTE;

pub const INT_WRAM: u32 = 0x03_00_00_00;
const INT_WRAM_SIZE: usize = 32 * KBYTE;

pub const IORAM: u32 = 0x04_00_00_00;
const IORAM_SIZE: usize = 1 * KBYTE;

pub const PAL_RAM: u32 = 0x05_00_00_00;
const PAL_RAM_SIZE: usize = 1 * KBYTE;

pub const VRAM: u32 = 0x06_00_00_00;
const VRAM_SIZE: usize = 96 * KBYTE;

pub const OAM: u32 = 0x07_00_00_00;
const OAM_SIZE: usize = 1 * KBYTE;

pub const PAK_ROM: u32 = 0x08_00_00_00;
pub const PAK_ROM1: u32 = 0x0A_00_00_00;
pub const PAK_ROM2: u32 = 0x0C_00_00_00;

pub const PAK_RAM: u32 = 0x0E_00_00_00;
const PAK_RAM_SIZE: usize = 64 * KBYTE;

pub struct Memory {
    blocks: BTreeMap<u32, Block>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory { blocks: Memory::new_blocks() }
    }

    fn new_blocks() -> BTreeMap<u32, Block> {
        let mut blocks = BTreeMap::new();

        blocks.insert(SYS_ROM, Block::new(SYS_ROM_SIZE));

        blocks.insert(EXT_WRAM, Block::new(EXT_WRAM_SIZE));
        blocks.insert(INT_WRAM, Block::new(INT_WRAM_SIZE));

        blocks.insert(IORAM, Block::new(IORAM_SIZE));
        blocks.insert(PAL_RAM, Block::new(PAL_RAM_SIZE));
        blocks.insert(VRAM, Block::new(VRAM_SIZE));
        blocks.insert(OAM, Block::new(OAM_SIZE));

        blocks.insert(PAK_RAM, Block::new(PAK_RAM_SIZE));

        blocks
    }

    pub fn load_pak(&mut self, data: &[u8]) {
        self.blocks.insert(PAK_ROM, Block::with_contents(data));
        self.blocks.insert(PAK_ROM1, Block::with_contents(data));
        self.blocks.insert(PAK_ROM2, Block::with_contents(data));
    }

    pub fn clear(&mut self) {
        self.blocks = Memory::new_blocks()
    }

    pub fn read(&self, addr: u32, size: usize) -> Option<Vec<u8>> {
        match self.blocks.range(0..=addr).last() {
            Some((start, block)) =>
                match addr >= *start && addr + (size as u32) < *start + block.len() as u32 {
                    // Make sure our addr actually fits in this block
                    true => {
                        // TODO Accessing the vec elements directly might be
                        //      faster than iter/skip/take?
                        let block_offset = addr - *start;
                        Some(block.data.iter()
                            .skip(block_offset as usize)
                            .take(size as usize)
                            .cloned()
                            .collect())
                    }
                    false => None,
                }
            None => None
        }
    }

    pub fn read_u32(&self, addr: u32) -> Option<u32> {
        match self.read(addr, 4) {
            Some(data) => {
                let mut buf = [0; 4];
                buf.copy_from_slice(&data);

                Some(u32::from_le_bytes(buf))
            }
            None => None,
        }
    }

    pub fn write(&mut self, addr: u32, data: &[u8]) {
        match self.blocks.range_mut(0..=addr).last() {
            Some((start, block)) => {
                match addr >= *start && addr + (data.len() as u32) < *start + block.len() as u32 {
                    true => {
                        let block_offset = (addr - *start) as usize;
                        for (idx, d) in data.iter().enumerate() {
                            block.data[block_offset + idx] = *d;
                        }
                    }
                    false => panic!("write extends beyond block"),
                }
            }
            None => panic!("memory block not found"),
        }
    }
}

pub struct Block {
    data: Vec<u8>,
}

impl Block {
    fn new(size: usize) -> Block {
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(0);
        }
        Block { data }
    }

    fn with_contents(contents: &[u8]) -> Block {
        Block { data: contents.to_vec() }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    mod block {
        use super::super::*;

        #[test]
        fn new() {
            let block = Block::new(1024);
            assert_eq!(1024, block.data.capacity());
            assert_eq!(1024, block.data.len());
            assert_eq!(1024, block.len())
        }
    }

    mod memory {
        use super::super::*;

        #[test]
        fn new_blocks() {
            let blocks = Memory::new_blocks();
            assert_eq!(true, blocks.contains_key(&SYS_ROM));
            assert_eq!(16 * KBYTE, blocks[&SYS_ROM].len());

            assert_eq!(true, blocks.contains_key(&EXT_WRAM));
            assert_eq!(256 * KBYTE, blocks[&EXT_WRAM].len());
            assert_eq!(true, blocks.contains_key(&INT_WRAM));
            assert_eq!(32 * KBYTE, blocks[&INT_WRAM].len());

            assert_eq!(true, blocks.contains_key(&IORAM));
            assert_eq!(1 * KBYTE, blocks[&IORAM].len());
            assert_eq!(true, blocks.contains_key(&PAL_RAM));
            assert_eq!(1 * KBYTE, blocks[&PAL_RAM].len());
            assert_eq!(true, blocks.contains_key(&VRAM));
            assert_eq!(96 * KBYTE, blocks[&VRAM].len());
            assert_eq!(true, blocks.contains_key(&OAM));
            assert_eq!(1 * KBYTE, blocks[&OAM].len());

            assert_eq!(true, blocks.contains_key(&PAK_RAM));
            assert_eq!(64 * KBYTE, blocks[&PAK_RAM].len());
        }

        #[test]
        fn write() {
            let mut m = Memory::new();
            m.write(PAK_RAM, &vec![1, 2, 3, 4]);

            assert_eq!(1, m.blocks[&PAK_RAM].data[0]);
            assert_eq!(2, m.blocks[&PAK_RAM].data[1]);
            assert_eq!(3, m.blocks[&PAK_RAM].data[2]);
            assert_eq!(4, m.blocks[&PAK_RAM].data[3]);
        }

        #[test]
        fn read() {
            let mut m = Memory::new();
            m.blocks.get_mut(&PAK_RAM).unwrap().data[0] = 1;
            m.blocks.get_mut(&PAK_RAM).unwrap().data[1] = 2;
            m.blocks.get_mut(&PAK_RAM).unwrap().data[2] = 3;
            m.blocks.get_mut(&PAK_RAM).unwrap().data[3] = 4;

            assert_eq!(Some(vec![1, 2, 3, 4]), m.read(PAK_RAM, 4));
        }
    }
}