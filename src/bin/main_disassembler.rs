use emulator::disassembler::disassemble_file;

fn main() {
    const ROM: &[u8] = include_bytes!("rom/space_invaders/invaders.rom");

    disassemble_file(&ROM, "result.txt");
}
