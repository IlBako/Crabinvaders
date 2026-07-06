/// Memory size is 64 KiB containing both ROM and RAM
const MEMORY_SIZE: usize = 65536;

#[derive(Debug)]
pub struct Memory {
    /// Flat 64 KiB array representing the entire addressable space
    memory: Box<[u8; MEMORY_SIZE]>,

    /// Boundaries for write-protection ROM region
    rom_bounds: Option<(usize, usize)>,
}

impl Memory {
    /// Initialize memory
    /// - For CPU Diagnostic Tests: Pass None to allow full 64 KB read/write RAM.
    /// - For Space Invaders: Pass Some((0x0000, 0x1FFF)) to protect the game's ROM.
    pub fn new(rom_bounds: Option<(usize, usize)>) -> Self {
        if let Some((start, end)) = rom_bounds {
            assert!(start <= end, "Invalid ROM bounds: start must be <= end");
            assert!(end <= MEMORY_SIZE, "ROM bounds exceed 64 KB address space");
        }
        Self {
            memory: Box::new([0; MEMORY_SIZE]),
            rom_bounds,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        let addr_usize = addr as usize;

        if let Some((start, end)) = self.rom_bounds {
            if addr_usize >= start && addr_usize <= end {
                return; // Protected ROM section, write is ignored
            }
        }

        self.memory[addr_usize] = val;
    }

    pub fn load_binary(&mut self, data: &[u8], start_addr: u16) {
        let start = start_addr as usize;
        let end = std::cmp::min(start + data.len(), MEMORY_SIZE);
        self.memory[start..end].copy_from_slice(&data[0..(end - start)]);
    }
}
