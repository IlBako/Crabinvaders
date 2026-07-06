// 8080 Space invaders clock speed: 1.9968 MHz
pub const CYCLES_SECOND: usize = 1_996_800;

pub mod disassembler;

mod utils;
pub use utils::*;

pub mod audio;
pub mod cpu;
pub mod hardware_impl;
pub mod int;
pub mod io;
pub mod memory;
pub mod video;

// Test utilities
#[cfg(test)]
mod tests;
