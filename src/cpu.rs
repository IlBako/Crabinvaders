/// Registers and condition bits implementation
pub mod registers;

/// Actual instruction implementation
mod instructions;

/// CPU system bus
pub mod bus;

use registers::ConditionBits;
use registers::Registers;

pub use bus::Bus;

use crate::io;

pub struct Cpu {
    /// Current CPU cycles
    cycles: usize,

    halt: bool,
    ime: bool,
    pc: u16,
    sp: u16,
    registers: Registers,
    condition_bits: ConditionBits,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            halt: false,
            ime: false,
            pc: 0,
            sp: 0,
            registers: Registers::default(),
            condition_bits: ConditionBits::default(),
        }
    }

    /// Run a CPU step
    pub fn step<I: io::IOHandler>(&mut self, bus: &mut Bus<I>) -> usize {
        let start = self.cycles;

        // if self.ime {
        //     // Disable halting
        //     self.halt = false;
        // } else if !self.halt {
        //     // Run a single instruction
        //     self.run_instr(bus);
        // } else {
        //     // Cpu is halted, wait 1 cycle
        //     self.cycles += 1;
        // }

        if self.halt {
            self.cycles += 1;
        } else {
            // Run a single instruction
            self.run_instr(bus);
        }

        self.cycles - start
    }

    /// Fetch a single byte at pc
    #[inline(always)]
    fn fetch_u8<I: io::IOHandler>(&mut self, bus: &Bus<I>) -> u8 {
        let addr = self.pc;
        self.pc = addr.wrapping_add(1);
        bus.read_u8(addr)
    }

    /// Fetch two bytes at pc
    #[inline(always)]
    fn fetch_u16<I: io::IOHandler>(&mut self, bus: &mut Bus<I>) -> u16 {
        let addr = self.pc;
        self.pc = addr.wrapping_add(2);
        bus.read_u16(addr)
    }
}
