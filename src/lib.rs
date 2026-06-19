// 8080 Space invaders clock speed: 1.9968 MHz
pub const CYCLES_SECOND: usize = 1_996_800;

pub mod disassembler;

mod utils;

pub use utils::real_time;

pub mod cpu;
pub mod memory;
