use crate::{cpu, io, memory};
use cpu::Bus;

struct DummyIO;

impl io::IOHandler for DummyIO {
    fn read_port(&self, _port: u8) -> u8 {
        0xFF
    }
    fn write_port(&mut self, _port: u8, _value: u8) {}
}

fn run_cpu_test(rom: &[u8]) -> String {
    let mut output = String::new();
    let mut cpu = cpu::Cpu::new();
    let mut memory = memory::Memory::new(None);
    let mut io = DummyIO;

    // ROM is loaded at 0x0100 as 0x0000-0x00FF was reserved for BOOT instruction
    memory.load_binary(rom, 0x0100);

    // Skip booting and go directly to test start
    cpu.set_pc(0x0100);

    // Inject a RET instruction (0xC9) at 0x0005 to return after a system call
    memory.write(0x0005, 0xC9);

    let mut cycles: u64 = 0;
    // Hard limit to prevent infinite loops if CPU has a bug and hangs
    let cycle_limit: u64 = 30_000_000_000;

    loop {
        // Intercept CP/M BDOS System Calls
        if cpu.get_pc() == 0x0005 {
            let c = cpu.get_register("C");

            if c == 9 {
                // Print string starting at DE until '$'
                let mut addr = cpu.get_register_pair("DE");
                loop {
                    let char_byte = memory.read(addr);
                    if char_byte == b'$' {
                        break;
                    }
                    output.push(char_byte as char);
                    addr = addr.wrapping_add(1);
                }
            } else if c == 2 {
                // Print single character in E
                let e = cpu.get_register("E");
                output.push(e as char);
            }
        }

        // Run one instruction (runs uncapped, as fast as possible)
        let step_cycles = cpu.step(&mut Bus {
            memory: &mut memory,
            io: &mut io,
        });
        cycles += step_cycles as u64;

        // The program ends when it jumps to 0x0000 (CP/M warm boot)
        if cpu.get_pc() == 0x0000 {
            break;
        }

        // Safety valve for cargo test
        if cycles > cycle_limit {
            output.push_str(
                "\n[TEST HARNESS ERROR]: Cycle limit exceeded. Infinite loop detected.\n",
            );
            break;
        }
    }

    output
}

#[test]
fn test_cpudiag() {
    let rom = include_bytes!("rom/cpudiag.bin");
    let output = run_cpu_test(rom);

    println!("{}", output);

    // cpudiag.bin prints this upon successful completion
    assert!(
        output.contains("CPU IS OPERATIONAL"),
        "cpudiag failed! Output was:\n{}",
        output
    );
}

#[test]
fn test_tst8080() {
    let rom = include_bytes!("rom/TST8080.COM");
    let output = run_cpu_test(rom);

    println!("{}", output);

    assert!(
        output.contains("CPU IS OPERATIONAL"),
        "TST8080 failed! Output was:\n{}",
        output
    );
}

#[test]
fn test_8080pre() {
    let rom = include_bytes!("rom/8080PRE.COM");
    let output = run_cpu_test(rom);

    println!("{}", output);

    assert!(
        !output.contains("ERROR"),
        "8080PRE failed! Output was:\n{}",
        output
    );
}

#[test]
fn test_cputest() {
    let rom = include_bytes!("rom/CPUTEST.COM");
    let output = run_cpu_test(rom);

    println!("{}", output);

    assert!(
        output.contains("CPU TESTS OK"),
        "CPUTEST failed! Output was:\n{}",
        output
    );
}

#[test]
fn test_8080exm() {
    // Note: 8080EXM takes billions of cycles. In release mode it takes a few seconds.
    // In debug mode (cargo test without --release), it might take a minute!
    let rom = include_bytes!("rom/8080EXM.COM");
    let output = run_cpu_test(rom);

    println!("{}", output);

    assert!(
        output.contains("8080 instruction exerciser") && !output.contains("ERROR"),
        "8080EXM failed! Output was:\n{}",
        output
    );
}
