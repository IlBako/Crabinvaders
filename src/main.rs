mod disassembler;
use disassembler::disassemble_file;

use crate::cpu::CPUState;

mod cpu;

fn main() {
    let rom_files = [
        "rom/space_invaders/invaders.h".to_string(), // 0x0000 - 0x07FF
        "rom/space_invaders/invaders.g".to_string(), // 0x0800 - 0x0FFF
        "rom/space_invaders/invaders.f".to_string(), // 0x1000 - 0x17FF
        "rom/space_invaders/invaders.e".to_string(), // 0x1800 - 0x1FFF
    ];

    let mut total_buffer = Vec::new();

    for path in &rom_files {
        let mut part =
            std::fs::read(path).unwrap_or_else(|_| panic!("Missing ROM file segment: {}", path));
        total_buffer.append(&mut part);
    }

    let mut state = cpu::init(total_buffer.to_vec());

    loop {
        match state.cpu_state {
            CPUState::RUNNING => state = cpu::read_instruction(state),
            CPUState::HALTED => {}
        }
    }
}

#[allow(unused)]
fn disassemble() {
    let rom_files = [
        "rom/space_invaders/invaders.h".to_string(), // 0x0000 - 0x07FF
        "rom/space_invaders/invaders.g".to_string(), // 0x0800 - 0x0FFF
        "rom/space_invaders/invaders.f".to_string(), // 0x1000 - 0x17FF
        "rom/space_invaders/invaders.e".to_string(), // 0x1800 - 0x1FFF
    ];

    disassemble_file(&rom_files, "result.txt");
}
