use emulator::disassembler::disassemble_file;

fn main() {
    let rom_files = [
        "rom/space_invaders/invaders.h".to_string(), // 0x0000 - 0x07FF
        "rom/space_invaders/invaders.g".to_string(), // 0x0800 - 0x0FFF
        "rom/space_invaders/invaders.f".to_string(), // 0x1000 - 0x17FF
        "rom/space_invaders/invaders.e".to_string(), // 0x1800 - 0x1FFF
    ];

    disassemble_file(&rom_files, "result.txt");
}
