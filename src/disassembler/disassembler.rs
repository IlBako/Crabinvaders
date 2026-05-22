use std::fs::OpenOptions;
use std::io::Write;

pub fn disassemble_file(rom_files: &[String], out_path: &str) {
    let mut total_buffer = Vec::new();

    for path in rom_files {
        let mut part =
            std::fs::read(path).unwrap_or_else(|_| panic!("Missing ROM file segment: {}", path));
        total_buffer.append(&mut part);
    }

    std::fs::create_dir_all("out").expect("Unable to create output directory");
    let mut out_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&format!("out/{}", out_path))
        .expect("Unable to open output file!");

    let fsize = total_buffer.len();
    let mut pc = 0;

    while pc < fsize {
        let (opbytes, instruction) = decode_instruction(&total_buffer, pc);
        writeln!(out_file, "{:04x} {}", pc, instruction).expect("Failed to write to file");
        pc += opbytes as usize;
    }
    println!("Disassembly complete! Result saved to out/{}", out_path);
}

fn decode_instruction(codebuffer: &[u8], pc: usize) -> (u8, String) {
    let mut opbytes: u8 = 1;
    let code = &codebuffer[pc..];

    let instruction = match code[0] {
        0x00 => "NOP".to_string(),
        0x01 => {
            opbytes = 3;
            format!("LXI\tB,#${:02x}{:02x}", code[2], code[1])
        }
        0x02 => "STAX\tB".to_string(),
        0x03 => "INX\tB".to_string(),
        0x04 => "INR\tB".to_string(),
        0x05 => "DCR\tB".to_string(),
        0x06 => {
            opbytes = 2;
            format!("MVI\tB,#${:02x}", code[1])
        }
        0x07 => "RLC".to_string(),

        0x08 => "NOP".to_string(),
        0x09 => "DAD\tB".to_string(),
        0x0a => "LDAX\tB".to_string(),
        0x0b => "DCX\tB".to_string(),
        0x0c => "INR\tC".to_string(),
        0x0d => "DCR\tC".to_string(),
        0x0e => {
            opbytes = 2;
            format!("MVI\tC,#${:02x}", code[1])
        }
        0x0f => "RRC".to_string(),

        0x10 => "NOP".to_string(),
        0x11 => {
            opbytes = 3;
            format!("LXI\tD,#${:02x}{:02x}", code[2], code[1])
        }
        0x12 => "STAX\tD".to_string(),
        0x13 => "INX\tD".to_string(),
        0x14 => "INR\tD".to_string(),
        0x15 => "DCR\tD".to_string(),
        0x16 => {
            opbytes = 2;
            format!("MVI\tD,#${:02x}", code[1])
        }
        0x17 => "RAL".to_string(),

        0x18 => "NOP".to_string(),
        0x19 => "DAD\tD".to_string(),
        0x1a => "LDAX\tD".to_string(),
        0x1b => "DCX\tC".to_string(),
        0x1c => "INR\tE".to_string(),
        0x1d => "DCR\tE".to_string(),
        0x1e => {
            opbytes = 2;
            format!("MVI\tE,#${:02x}", code[1])
        }
        0x1f => "RAC".to_string(),

        0x20 => "NOP".to_string(),
        0x21 => {
            opbytes = 3;
            format!("LXI\tH,#${:02x}{:02x}", code[2], code[1])
        }
        0x22 => {
            opbytes = 3;
            format!("SHLD\tH,#${:02x}{:02x}", code[2], code[1])
        }
        0x23 => "INX\tH".to_string(),
        0x24 => "INR\tH".to_string(),
        0x25 => "DCR\tH".to_string(),
        0x26 => {
            opbytes = 2;
            format!("MVI\tH,#${:02x}", code[1])
        }
        0x27 => "DAA".to_string(),

        0x28 => "NOP".to_string(),
        0x29 => "DAD\tH".to_string(),
        0x2a => {
            opbytes = 3;
            format!("LHLD\t${:02x}{:02x}", code[2], code[1])
        }
        0x2b => "DCX\tH".to_string(),
        0x2c => "INR\tL".to_string(),
        0x2d => "DCR\tL".to_string(),
        0x2e => {
            opbytes = 2;
            format!("MVI\tL,#${:02x}", code[1])
        }
        0x2f => "CMA".to_string(),

        0x30 => "NOP".to_string(),
        0x31 => {
            opbytes = 3;
            format!("LXI\tSP,#${:02x}{:02x}", code[2], code[1])
        }
        0x32 => {
            opbytes = 3;
            format!("STA\t${:02x}{:02x}", code[2], code[1])
        }
        0x33 => "INX\tSP".to_string(),
        0x34 => "INR\tM".to_string(),
        0x35 => "DCR\tM".to_string(),
        0x36 => {
            opbytes = 2;
            format!("MVI\tM,#${:02x}", code[1])
        }
        0x37 => "STC".to_string(),

        0x38 => "NOP".to_string(),
        0x39 => "DAD\tSP".to_string(),
        0x3a => {
            opbytes = 3;
            format!("LDA\t${:02x}{:02x}", code[2], code[1])
        }
        0x3b => "DCX\tSP".to_string(),
        0x3c => "INR\tA".to_string(),
        0x3d => "DCR\tA".to_string(),
        0x3e => {
            opbytes = 2;
            format!("MVI\tA,#${:02x}", code[1])
        }
        0x3f => "CMC".to_string(),

        0x40 => "MOV\tB,B".to_string(),
        0x41 => "MOV\tB,C".to_string(),
        0x42 => "MOV\tB,D".to_string(),
        0x43 => "MOV\tB,E".to_string(),
        0x44 => "MOV\tB,H".to_string(),
        0x45 => "MOV\tB,L".to_string(),
        0x46 => "MOV\tB,M".to_string(),
        0x47 => "MOV\tB,A".to_string(),

        0x48 => "MOV\tC,B".to_string(),
        0x49 => "MOV\tC,C".to_string(),
        0x4a => "MOV\tC,D".to_string(),
        0x4b => "MOV\tC,E".to_string(),
        0x4c => "MOV\tC,H".to_string(),
        0x4d => "MOV\tC,L".to_string(),
        0x4e => "MOV\tC,M".to_string(),
        0x4f => "MOV\tC,A".to_string(),

        0x50 => "MOV\tD,B".to_string(),
        0x51 => "MOV\tD,C".to_string(),
        0x52 => "MOV\tD,D".to_string(),
        0x53 => "MOV\tD,E".to_string(),
        0x54 => "MOV\tD,H".to_string(),
        0x55 => "MOV\tD,L".to_string(),
        0x56 => "MOV\tD,M".to_string(),
        0x57 => "MOV\tD,A".to_string(),

        0x58 => "MOV\tE,B".to_string(),
        0x59 => "MOV\tE,C".to_string(),
        0x5a => "MOV\tE,D".to_string(),
        0x5b => "MOV\tE,E".to_string(),
        0x5c => "MOV\tE,H".to_string(),
        0x5d => "MOV\tE,L".to_string(),
        0x5e => "MOV\tE,M".to_string(),
        0x5f => "MOV\tE,A".to_string(),

        0x60 => "MOV\tH,B".to_string(),
        0x61 => "MOV\tH,C".to_string(),
        0x62 => "MOV\tH,D".to_string(),
        0x63 => "MOV\tH,E".to_string(),
        0x64 => "MOV\tH,H".to_string(),
        0x65 => "MOV\tH,L".to_string(),
        0x66 => "MOV\tH,M".to_string(),
        0x67 => "MOV\tH,A".to_string(),

        0x68 => "MOV\tL,B".to_string(),
        0x69 => "MOV\tL,C".to_string(),
        0x6a => "MOV\tL,D".to_string(),
        0x6b => "MOV\tL,E".to_string(),
        0x6c => "MOV\tL,H".to_string(),
        0x6d => "MOV\tL,L".to_string(),
        0x6e => "MOV\tL,M".to_string(),
        0x6f => "MOV\tL,A".to_string(),

        0x70 => "MOV\tM,B".to_string(),
        0x71 => "MOV\tM,C".to_string(),
        0x72 => "MOV\tM,D".to_string(),
        0x73 => "MOV\tM,E".to_string(),
        0x74 => "MOV\tM,H".to_string(),
        0x75 => "MOV\tM,L".to_string(),
        0x76 => "HLT".to_string(),
        0x77 => "MOV\tM,A".to_string(),

        0x78 => "MOV\tA,B".to_string(),
        0x79 => "MOV\tA,C".to_string(),
        0x7a => "MOV\tA,D".to_string(),
        0x7b => "MOV\tA,E".to_string(),
        0x7c => "MOV\tA,H".to_string(),
        0x7d => "MOV\tA,L".to_string(),
        0x7e => "MOV\tA,M".to_string(),
        0x7f => "MOV\tA,A".to_string(),

        0x80 => "ADD\tB".to_string(),
        0x81 => "ADD\tC".to_string(),
        0x82 => "ADD\tD".to_string(),
        0x83 => "ADD\tE".to_string(),
        0x84 => "ADD\tH".to_string(),
        0x85 => "ADD\tL".to_string(),
        0x86 => "ADD\tM".to_string(),
        0x87 => "ADD\tA".to_string(),

        0x88 => "ADC\tB".to_string(),
        0x89 => "ADC\tC".to_string(),
        0x8a => "ADC\tD".to_string(),
        0x8b => "ADC\tE".to_string(),
        0x8c => "ADC\tH".to_string(),
        0x8d => "ADC\tL".to_string(),
        0x8e => "ADC\tM".to_string(),
        0x8f => "ADC\tA".to_string(),

        0x90 => "SUB\tB".to_string(),
        0x91 => "SUB\tC".to_string(),
        0x92 => "SUB\tD".to_string(),
        0x93 => "SUB\tE".to_string(),
        0x94 => "SUB\tH".to_string(),
        0x95 => "SUB\tL".to_string(),
        0x96 => "SUB\tM".to_string(),
        0x97 => "SUB\tA".to_string(),

        0x98 => "SBB\tB".to_string(),
        0x99 => "SBB\tC".to_string(),
        0x9a => "SBB\tD".to_string(),
        0x9b => "SBB\tE".to_string(),
        0x9c => "SBB\tH".to_string(),
        0x9d => "SBB\tL".to_string(),
        0x9e => "SBB\tM".to_string(),
        0x9f => "SBB\tA".to_string(),

        0xa0 => "ANA\tB".to_string(),
        0xa1 => "ANA\tC".to_string(),
        0xa2 => "ANA\tD".to_string(),
        0xa3 => "ANA\tE".to_string(),
        0xa4 => "ANA\tH".to_string(),
        0xa5 => "ANA\tL".to_string(),
        0xa6 => "ANA\tM".to_string(),
        0xa7 => "ANA\tA".to_string(),

        0xa8 => "XRA\tB".to_string(),
        0xa9 => "XRA\tC".to_string(),
        0xaa => "XRA\tD".to_string(),
        0xab => "XRA\tE".to_string(),
        0xac => "XRA\tH".to_string(),
        0xad => "XRA\tL".to_string(),
        0xae => "XRA\tM".to_string(),
        0xaf => "XRA\tA".to_string(),

        0xb0 => "ORA\tB".to_string(),
        0xb1 => "ORA\tC".to_string(),
        0xb2 => "ORA\tD".to_string(),
        0xb3 => "ORA\tE".to_string(),
        0xb4 => "ORA\tH".to_string(),
        0xb5 => "ORA\tL".to_string(),
        0xb6 => "ORA\tM".to_string(),
        0xb7 => "ORA\tA".to_string(),

        0xb8 => "CMP\tB".to_string(),
        0xb9 => "CMP\tC".to_string(),
        0xba => "CMP\tD".to_string(),
        0xbb => "CMP\tE".to_string(),
        0xbc => "CMP\tH".to_string(),
        0xbd => "CMP\tL".to_string(),
        0xbe => "CMP\tM".to_string(),
        0xbf => "CMP\tA".to_string(),

        0xc0 => "RNZ".to_string(),
        0xc1 => "POP\tB".to_string(),
        0xc2 => {
            opbytes = 3;
            format!("JNZ\t${:02x}{:02x}", code[2], code[1])
        }
        0xc3 => {
            opbytes = 3;
            format!("JMP\t${:02x}{:02x}", code[2], code[1])
        }
        0xc4 => {
            opbytes = 3;
            format!("CNZ\t${:02x}{:02x}", code[2], code[1])
        }
        0xc5 => "PUSH\tB".to_string(),
        0xc6 => {
            opbytes = 2;
            format!("ADI\t${:02x}", code[1])
        }
        0xc7 => "RST\t0".to_string(),

        0xc8 => "RZ".to_string(),
        0xc9 => "RET".to_string(),
        0xca => {
            opbytes = 3;
            format!("JZ\t\t${:02x}{:02x}", code[2], code[1])
        }
        0xcb => "NOP".to_string(),
        0xcc => {
            opbytes = 3;
            format!("CZ\t\t${:02x}{:02x}", code[2], code[1])
        }
        0xcd => {
            opbytes = 3;
            format!("CALL\t${:02x}{:02x}", code[2], code[1])
        }
        0xce => {
            opbytes = 2;
            format!("ACI\t${:02x}", code[1])
        }
        0xcf => "RST\t1".to_string(),

        0xd0 => "RNC".to_string(),
        0xd1 => "POP\tD".to_string(),
        0xd2 => {
            opbytes = 3;
            format!("JNC\t${:02x}{:02x}", code[2], code[1])
        }
        0xd3 => {
            opbytes = 2;
            format!("OUT\t${:02x}", code[1])
        }
        0xd4 => {
            opbytes = 3;
            format!("CNC\t${:02x}{:02x}", code[2], code[1])
        }
        0xd5 => "PUSH\tD".to_string(),
        0xd6 => {
            opbytes = 2;
            format!("SUI\t${:02x}", code[1])
        }
        0xd7 => "RST\t2".to_string(),

        0xd8 => "RC".to_string(),
        0xd9 => "NOP".to_string(),
        0xda => {
            opbytes = 3;
            format!("JC\t\t${:02x}{:02x}", code[2], code[1])
        }
        0xdb => {
            opbytes = 2;
            format!("IN\t\t${:02x}", code[1])
        }
        0xdc => {
            opbytes = 3;
            format!("CC\t\t${:02x}{:02x}", code[2], code[1])
        }
        0xdd => "NOP".to_string(),
        0xde => {
            opbytes = 2;
            format!("SBI\t${:02x}", code[1])
        }
        0xdf => "RST\t3".to_string(),

        0xe0 => "RPO".to_string(),
        0xe1 => "POP\tH".to_string(),
        0xe2 => {
            opbytes = 3;
            format!("JPO\t${:02x}{:02x}", code[2], code[1])
        }
        0xe3 => "XTHL".to_string(),
        0xe4 => {
            opbytes = 3;
            format!("CPO\t${:02x}{:02x}", code[2], code[1])
        }
        0xe5 => "PUSH\tH".to_string(),
        0xe6 => {
            opbytes = 2;
            format!("ANI\t${:02x}", code[1])
        }
        0xe7 => "RST\t4".to_string(),

        0xe8 => "RPE".to_string(),
        0xe9 => "PCHL".to_string(),
        0xea => {
            opbytes = 3;
            format!("JPE\t${:02x}{:02x}", code[2], code[1])
        }
        0xeb => "XCHG".to_string(),
        0xec => {
            opbytes = 3;
            format!("CPE\t${:02x}{:02x}", code[2], code[1])
        }
        0xed => "NOP".to_string(),
        0xee => {
            opbytes = 2;
            format!("XRI\t${:02x}", code[1])
        }
        0xef => "RST\t5".to_string(),

        0xf0 => "RP".to_string(),
        0xf1 => "POP\tPSW".to_string(),
        0xf2 => {
            opbytes = 3;
            format!("JP\t\t${:02x}{:02x}", code[2], code[1])
        }
        0xf3 => "DI".to_string(),
        0xf4 => {
            opbytes = 3;
            format!("CP\t\t${:02x}{:02x}", code[2], code[1])
        }
        0xf5 => "PUSH\tPSW".to_string(),
        0xf6 => {
            opbytes = 2;
            format!("ORI\t${:02x}", code[1])
        }
        0xf7 => "RST\t6".to_string(),

        0xf8 => "RM".to_string(),
        0xf9 => "SPHL".to_string(),
        0xfa => {
            opbytes = 3;
            format!("JM\t\t${:02x}{:02x}", code[2], code[1])
        }
        0xfb => "EI".to_string(),
        0xfc => {
            opbytes = 3;
            format!("CM\t\t${:02x}{:02x}", code[2], code[1])
        }
        0xfd => "NOP".to_string(),
        0xfe => {
            opbytes = 2;
            format!("CPI\t${:02x}", code[1])
        }
        0xff => "RST\t7".to_string(),
    };

    return (opbytes, instruction);
}
