use crate::*;

/// Bus that connects all devices

pub struct Bus<'a, I: io::IOHandler> {
    pub memory: &'a mut memory::Memory,
    pub io: &'a mut I,
}

impl<'a, I: io::IOHandler> Bus<'_, I> {
    /// Read a single byte from memory
    pub fn read_u8(&self, addr: u16) -> u8 {
        self.memory.read(addr)
    }

    /// Write a single byte to memory
    pub fn write_u8(&mut self, addr: u16, val: u8) {
        self.memory.write(addr, val);
    }

    /// Read two bytes from memory
    pub fn read_u16(&mut self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read_u8(addr), self.read_u8(addr.wrapping_add(1))])
    }

    /// Write two bytes to memory
    pub fn write_u16(&mut self, addr: u16, val: u16) {
        let val = val.to_le_bytes();
        self.write_u8(addr, val[0]);
        self.write_u8(addr.wrapping_add(1), val[1]);
    }

    /// Read from port
    pub fn io_read(&self, port: u8) -> u8 {
        self.io.read_port(port)
    }

    // Write to port
    pub fn io_write(&mut self, port: u8, value: u8) {
        self.io.write_port(port, value);
    }
}
