use std::fmt::{self, Binary};

use super::instructions::*;

#[allow(non_snake_case)]
#[derive(Default)]
struct Registers {
    A: u8,
    B: u8,
    C: u8,
    D: u8,
    E: u8,
    H: u8,
    L: u8,
}

impl Registers {
    fn get_bc(&self) -> u16 {
        ((self.B as u16) << 8) | self.C as u16
    }
    fn get_de(&self) -> u16 {
        ((self.D as u16) << 8) | self.E as u16
    }
    fn get_hl(&self) -> u16 {
        ((self.H as u16) << 8) | self.L as u16
    }

    fn set_bc(&mut self, val: u16) {
        self.B = (val >> 8) as u8;
        self.C = val as u8;
    }
    fn set_de(&mut self, val: u16) {
        self.D = (val >> 8) as u8;
        self.E = val as u8;
    }
    fn set_hl(&mut self, val: u16) {
        self.H = (val >> 8) as u8;
        self.L = val as u8;
    }
}

pub struct ConditionBits {
    bits: u8,
}

pub struct State {
    registers: Registers,
    memory: Vec<u8>,
    pub pc: u16,
    sp: u16,
    pub(crate) condition_bits: ConditionBits,
    pub cpu_state: CPUState,
    // interrupt_enabled: bool,
}

pub enum CPUState {
    RUNNING,
    HALTED,
}

impl State {
    fn new() -> State {
        State {
            registers: Registers::default(),
            memory: vec![0; 0x10000],
            pc: 0,
            sp: 0,
            condition_bits: ConditionBits::default(),
            cpu_state: CPUState::RUNNING,
            // interrupted: false,
        }
    }
}

impl Default for ConditionBits {
    fn default() -> Self {
        Self { bits: 0b0000_0010 }
    }
}

impl fmt::Display for ConditionBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "sign: {},\nzero: {},\nac: {},\nparity: {},\ncarry: {}",
            self.s(),
            self.z(),
            self.ac(),
            self.p(),
            self.c()
        )
    }
}

impl Binary for ConditionBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "0b{}{}0{}0{}1{}",
            self.s() as u8,
            self.z() as u8,
            self.ac() as u8,
            self.p() as u8,
            self.c() as u8
        )
    }
}

#[allow(unused)]
impl ConditionBits {
    // Bit 7: Sign (s)
    // Bit 6: Zero (z)
    // Bit 5: 0
    // Bit 4: Auxiliary carry (ac)
    // Bit 3: 0
    // Bit 2: Parity (p)
    // Bit 1: 1
    // Bit 0: Carry (c)

    // Getters
    fn c(&self) -> bool {
        self.bits & 0x01 != 0
    }
    fn p(&self) -> bool {
        self.bits & (0x01 << 2) != 0
    }
    fn ac(&self) -> bool {
        self.bits & (0x01 << 4) != 0
    }
    fn z(&self) -> bool {
        self.bits & (0x01 << 6) != 0
    }
    fn s(&self) -> bool {
        self.bits & (0x01 << 7) != 0
    }

    // Setters
    pub(crate) fn set_c(&mut self, value: bool) {
        if value {
            self.bits |= 0x01;
        } else {
            self.bits &= !0x01;
        }
    }
    pub(crate) fn set_p(&mut self, value: bool) {
        if value {
            self.bits |= 0x01 << 2;
        } else {
            self.bits &= !(0x01 << 2);
        }
    }
    pub(crate) fn set_ac(&mut self, value: bool) {
        if value {
            self.bits |= 0x01 << 4;
        } else {
            self.bits &= !(0x01 << 4);
        }
    }
    pub(crate) fn set_z(&mut self, value: bool) {
        if value {
            self.bits |= 0x01 << 6;
        } else {
            self.bits &= !(0x01 << 6);
        }
    }
    pub(crate) fn set_s(&mut self, value: bool) {
        if value {
            self.bits |= 0x01 << 7;
        } else {
            self.bits &= !(0x01 << 7);
        }
    }

    fn set_bits(&mut self, value: u8) {
        // Allows to set all the bits together while guaranteeing that bit 3 and 5 remain 0 and bit 1 remains 1
        self.bits = (value & 0b1101_0111) | 0b0000_0010;
    }
}

pub fn init(rom: Vec<u8>) -> State {
    let mut state = State::new();
    // Load ROM into a full 64KB memory buffer. If ROM is smaller, leave the
    // remaining memory zeroed. If larger, truncate to 0x10000 bytes.
    let max = state.memory.len();
    let len = rom.len().min(max);
    state.memory[..len].copy_from_slice(&rom[..len]);
    state
}

#[allow(dead_code)]
pub fn read_instruction(mut current_state: State) -> State {
    let mut pc: usize = current_state.pc as usize;
    let code = &current_state.memory[pc..];

    pc += 1;

    match code[0] {
        0x00 => { /* NOP */ }
        0x01 => {
            // LXI B, D16
            current_state.registers.C = code[1];
            current_state.registers.B = code[2];
            pc += 2
        }
        0x02 => {
            // STAX B
            let address = current_state.registers.get_bc();
            current_state.memory[address as usize] = current_state.registers.A;
        }
        0x03 => {
            // INX B
            let res = current_state.registers.get_bc().wrapping_add(1);
            current_state.registers.set_bc(res);
        }
        0x04 => {
            // INR B
            current_state.registers.B =
                inr_instruction(current_state.registers.B, &mut current_state.condition_bits);
        }
        0x05 => {
            // DCR B
            current_state.registers.B =
                dcr_instruction(current_state.registers.B, &mut current_state.condition_bits);
        }
        0x06 => {
            // MVI B, D8
            current_state.registers.B = code[1];
            pc += 1
        }
        0x07 => {
            // RLC
            let val = current_state.registers.A;
            current_state.condition_bits.set_c(val >> 7 == 1);
            current_state.registers.A = (val << 1) | (val >> 7);
        }
        0x08 => { /* NOP */ }
        0x09 => {
            // DAD B
            let hl = current_state.registers.get_hl();
            let bc = current_state.registers.get_bc();
            let (res, carry) = hl.overflowing_add(bc);
            current_state.registers.set_hl(res);
            current_state.condition_bits.set_c(carry);
        }
        0x0a => {
            // LDAX B
            let address = current_state.registers.get_bc();
            current_state.registers.A = current_state.memory[address as usize];
        }
        0x0b => {
            // DCX B
            let res = current_state.registers.get_bc().wrapping_sub(1);
            current_state.registers.set_bc(res);
        }
        0x0c => {
            // INR C
            current_state.registers.C =
                inr_instruction(current_state.registers.C, &mut current_state.condition_bits);
        }
        0x0d => {
            // DCR C
            current_state.registers.C =
                dcr_instruction(current_state.registers.C, &mut current_state.condition_bits);
        }
        0x0e => {
            // MVI C, D8
            current_state.registers.C = code[1];
            pc += 1
        }
        0x0f => {
            // RRC
            let val = current_state.registers.A;
            current_state.condition_bits.set_c(val & 0x01 == 1);
            current_state.registers.A = (val >> 1) | (val << 7);
        }
        0x10 => { /* NOP */ }
        0x11 => {
            // LXI D, D16
            current_state.registers.E = code[1];
            current_state.registers.D = code[2];
            pc += 2
        }
        0x12 => {
            // STAX D
            let address = current_state.registers.get_de();
            current_state.memory[address as usize] = current_state.registers.A;
        }
        0x13 => {
            // INX D
            let res = current_state.registers.get_de().wrapping_add(1);
            current_state.registers.set_de(res);
        }
        0x14 => {
            // INR D
            current_state.registers.D =
                inr_instruction(current_state.registers.D, &mut current_state.condition_bits);
        }
        0x15 => {
            // DCR D
            current_state.registers.D =
                dcr_instruction(current_state.registers.D, &mut current_state.condition_bits);
        }
        0x16 => {
            // MVI D, D8
            current_state.registers.D = code[1];
            pc += 1
        }
        0x17 => {
            // RAL
            let val = current_state.registers.A;
            current_state.registers.A = (val << 1) | (current_state.condition_bits.c() as u8);
            current_state.condition_bits.set_c(val >> 7 == 1);
        }
        0x18 => { /* NOP */ }
        0x19 => {
            // DAD D
            let hl = current_state.registers.get_hl();
            let de = current_state.registers.get_de();
            let (res, carry) = hl.overflowing_add(de);
            current_state.registers.set_hl(res);
            current_state.condition_bits.set_c(carry);
        }
        0x1a => {
            // LDAX D
            let address = current_state.registers.get_de();
            current_state.registers.A = current_state.memory[address as usize];
        }
        0x1b => {
            // DCX D
            let res = current_state.registers.get_de().wrapping_sub(1);
            current_state.registers.set_de(res);
        }
        0x1c => {
            // INR E
            current_state.registers.E =
                inr_instruction(current_state.registers.E, &mut current_state.condition_bits);
        }
        0x1d => {
            // DCR E
            current_state.registers.E =
                dcr_instruction(current_state.registers.E, &mut current_state.condition_bits);
        }
        0x1e => {
            // MVI E, D8
            current_state.registers.E = code[1];
            pc += 1
        }
        0x1f => {
            // RAR
            let val = current_state.registers.A;
            current_state.registers.A =
                (val >> 1) | ((current_state.condition_bits.c() as u8) << 7);
            current_state.condition_bits.set_c(val & 0x01 == 1);
        }
        0x20 => { /* NOP */ }
        0x21 => {
            // LXI H, D16
            current_state.registers.L = code[1];
            current_state.registers.H = code[2];
            pc += 2
        }
        0x22 => {
            // SHLD adr
            let address = ((code[2] as u16) << 8) | code[1] as u16;
            current_state.memory[address as usize] = current_state.registers.L;
            current_state.memory[address.wrapping_add(1) as usize] = current_state.registers.H;
            pc += 2
        }
        0x23 => {
            // INX H
            let res = current_state.registers.get_hl().wrapping_add(1);
            current_state.registers.set_hl(res);
        }
        0x24 => {
            // INR H
            current_state.registers.H =
                inr_instruction(current_state.registers.H, &mut current_state.condition_bits);
        }
        0x25 => {
            // DCR H
            current_state.registers.H =
                dcr_instruction(current_state.registers.H, &mut current_state.condition_bits);
        }
        0x26 => {
            // MVI H, D8
            current_state.registers.H = code[1];
            pc += 1
        }
        0x27 => {
            // DAA
            let mut accumulator = current_state.registers.A;
            let ac = current_state.condition_bits.ac();
            let old_c = current_state.condition_bits.c();
            let mut step1_carry = false;
            let mut step2_carry = false;

            let lower_nibble = accumulator & 0x0F;
            if lower_nibble > 9 || ac {
                current_state
                    .condition_bits
                    .set_ac((lower_nibble + 0x06) > 0x0F);
                (accumulator, step1_carry) = accumulator.overflowing_add(6);
            } else {
                current_state.condition_bits.set_ac(false);
            }
            let upper_nibble = accumulator >> 4;
            if upper_nibble > 9 || old_c || step1_carry {
                (accumulator, step2_carry) = accumulator.overflowing_add(0x60);
            }
            current_state
                .condition_bits
                .set_c(step1_carry | step2_carry | old_c);
            current_state.condition_bits.set_z(accumulator == 0);
            current_state
                .condition_bits
                .set_p(accumulator.count_ones() % 2 == 0);
            current_state
                .condition_bits
                .set_s(accumulator & 0x80 == 0x80);
            current_state.registers.A = accumulator;
        }
        0x28 => { /* NOP */ }
        0x29 => {
            // DAD H
            let hl = current_state.registers.get_hl();
            let res = hl << 1;
            current_state.registers.set_hl(res);
            current_state.condition_bits.set_c(res < hl);
        }
        0x2a => {
            // LHLD adr
            let address = ((code[2] as u16) << 8) | code[1] as u16;
            current_state.registers.L = current_state.memory[address as usize];
            current_state.registers.H = current_state.memory[address.wrapping_add(1) as usize];
            pc += 2
        }
        0x2b => {
            // DCX H
            let res = current_state.registers.get_hl().wrapping_sub(1);
            current_state.registers.set_hl(res);
        }
        0x2c => {
            // INR L
            current_state.registers.L =
                inr_instruction(current_state.registers.L, &mut current_state.condition_bits);
        }
        0x2d => {
            // DCR L
            current_state.registers.L =
                dcr_instruction(current_state.registers.L, &mut current_state.condition_bits);
        }
        0x2e => {
            // MVI L, D8
            current_state.registers.L = code[1];
            pc += 1
        }
        0x2f => {
            // CMA
            current_state.registers.A = !current_state.registers.A;
        }
        0x30 => { /* NOP */ }
        0x31 => {
            // LXI SP, D16
            current_state.sp = (code[2] as u16) << 8 | code[1] as u16;
            pc += 2
        }
        0x32 => {
            // STA adr
            let address = (code[2] as u16) << 8 | code[1] as u16;
            current_state.memory[address as usize] = current_state.registers.A;
            pc += 2;
        }
        0x33 => {
            // INX SP
            current_state.sp = current_state.sp.wrapping_add(1);
        }
        0x34 => {
            // INR M
            let address = current_state.registers.get_hl();
            let val = current_state.memory[address as usize];
            let res = inr_instruction(val, &mut current_state.condition_bits);
            current_state.memory[address as usize] = res;
        }
        0x35 => {
            // DCR M
            let address = current_state.registers.get_hl();
            let val = current_state.memory[address as usize];
            let res = dcr_instruction(val, &mut current_state.condition_bits);
            current_state.memory[address as usize] = res;
        }
        0x36 => {
            // MVI M, D8
            let address = current_state.registers.get_hl();
            current_state.memory[address as usize] = code[1];
            pc += 1
        }
        0x37 => {
            // STC
            current_state.condition_bits.set_c(true);
        }
        0x38 => { /* NOP */ }
        0x39 => {
            // DAD SP
            let hl = current_state.registers.get_hl();
            let sp = current_state.sp;
            let (res, carry) = hl.overflowing_add(sp);
            current_state.registers.set_hl(res);
            current_state.condition_bits.set_c(carry);
        }
        0x3a => {
            // LDA adr
            let address = (code[2] as u16) << 8 | code[1] as u16;
            current_state.registers.A = current_state.memory[address as usize];
            pc += 2;
        }
        0x3b => {
            // DCX SP
            current_state.sp = current_state.sp.wrapping_sub(1);
        }
        0x3c => {
            // INR A
            current_state.registers.A =
                inr_instruction(current_state.registers.A, &mut current_state.condition_bits);
        }
        0x3d => {
            // DCR A
            current_state.registers.A =
                dcr_instruction(current_state.registers.A, &mut current_state.condition_bits);
        }
        0x3e => {
            // MVI A, D8
            current_state.registers.A = code[1];
            pc += 1
        }
        0x3f => {
            // CMC
            current_state
                .condition_bits
                .set_c(!current_state.condition_bits.c());
        }
        0x40 => {
            // MOV B, B
            current_state.registers.B = current_state.registers.B;
        }
        0x41 => {
            // MOV B, C
            current_state.registers.B = current_state.registers.C;
        }
        0x42 => {
            // MOV B, D
            current_state.registers.B = current_state.registers.D;
        }
        0x43 => {
            // MOV B, E
            current_state.registers.B = current_state.registers.E;
        }
        0x44 => {
            // MOV B, H
            current_state.registers.B = current_state.registers.H;
        }
        0x45 => {
            // MOV B, L
            current_state.registers.B = current_state.registers.L;
        }
        0x46 => {
            // MOV B, M
            let address = current_state.registers.get_hl();
            current_state.registers.B = current_state.memory[address as usize];
        }
        0x47 => {
            // MOV B, A
            current_state.registers.B = current_state.registers.A;
        }
        0x48 => {
            // MOV C, B
            current_state.registers.C = current_state.registers.B;
        }
        0x49 => {
            // MOV C, C
            current_state.registers.C = current_state.registers.C;
        }
        0x4a => {
            // MOV C, D
            current_state.registers.C = current_state.registers.D;
        }
        0x4b => {
            // MOV C, E
            current_state.registers.C = current_state.registers.E;
        }
        0x4c => {
            // MOV C, H
            current_state.registers.C = current_state.registers.H;
        }
        0x4d => {
            // MOV C, L
            current_state.registers.C = current_state.registers.L;
        }
        0x4e => {
            // MOV C, M
            let address = current_state.registers.get_hl();
            current_state.registers.C = current_state.memory[address as usize];
        }
        0x4f => {
            // MOV C, A
            current_state.registers.C = current_state.registers.A;
        }
        0x50 => {
            // MOV D, B
            current_state.registers.D = current_state.registers.B;
        }
        0x51 => {
            // MOV D, C
            current_state.registers.D = current_state.registers.C;
        }
        0x52 => {
            // MOV D, D
            current_state.registers.D = current_state.registers.D;
        }
        0x53 => {
            // MOV D, E
            current_state.registers.D = current_state.registers.E;
        }
        0x54 => {
            // MOV D, H
            current_state.registers.D = current_state.registers.H;
        }
        0x55 => {
            // MOV D, L
            current_state.registers.D = current_state.registers.L;
        }
        0x56 => {
            // MOV D, M
            let address = current_state.registers.get_hl();
            current_state.registers.D = current_state.memory[address as usize];
        }
        0x57 => {
            // MOV D, A
            current_state.registers.D = current_state.registers.A;
        }
        0x58 => {
            // MOV E, B
            current_state.registers.E = current_state.registers.B;
        }
        0x59 => {
            // MOV E, C
            current_state.registers.E = current_state.registers.C;
        }
        0x5a => {
            // MOV E, D
            current_state.registers.E = current_state.registers.D;
        }
        0x5b => {
            // MOV E, E
            current_state.registers.E = current_state.registers.E;
        }
        0x5c => {
            // MOV E, H
            current_state.registers.E = current_state.registers.H;
        }
        0x5d => {
            // MOV E, L
            current_state.registers.E = current_state.registers.L;
        }
        0x5e => {
            // MOV E, M
            let address = current_state.registers.get_hl();
            current_state.registers.E = current_state.memory[address as usize];
        }
        0x5f => {
            // MOV E, A
            current_state.registers.E = current_state.registers.A;
        }
        0x60 => {
            // MOV H, B
            current_state.registers.H = current_state.registers.B;
        }
        0x61 => {
            // MOV H, C
            current_state.registers.H = current_state.registers.C;
        }
        0x62 => {
            // MOV H, D
            current_state.registers.H = current_state.registers.D;
        }
        0x63 => {
            // MOV H, E
            current_state.registers.H = current_state.registers.E;
        }
        0x64 => {
            // MOV H, H
            current_state.registers.H = current_state.registers.H;
        }
        0x65 => {
            // MOV H, L
            current_state.registers.H = current_state.registers.L;
        }
        0x66 => {
            // MOV H, M
            let address = current_state.registers.get_hl();
            current_state.registers.H = current_state.memory[address as usize];
        }
        0x67 => {
            // MOV H, A
            current_state.registers.H = current_state.registers.A;
        }
        0x68 => {
            // MOV L, B
            current_state.registers.L = current_state.registers.B;
        }
        0x69 => {
            // MOV L, C
            current_state.registers.L = current_state.registers.C;
        }
        0x6a => {
            // MOV L, D
            current_state.registers.L = current_state.registers.D;
        }
        0x6b => {
            // MOV L, E
            current_state.registers.L = current_state.registers.E;
        }
        0x6c => {
            // MOV L, H
            current_state.registers.L = current_state.registers.H;
        }
        0x6d => {
            // MOV L, L
            current_state.registers.L = current_state.registers.L;
        }
        0x6e => {
            // MOV L, M
            let address = current_state.registers.get_hl();
            current_state.registers.L = current_state.memory[address as usize];
        }
        0x6f => {
            // MOV L, A
            current_state.registers.L = current_state.registers.A;
        }
        0x70 => {
            // MOV M, B
            let address = current_state.registers.get_hl();
            current_state.memory[address as usize] = current_state.registers.B;
        }
        0x71 => {
            // MOV M, C
            let address = current_state.registers.get_hl();
            current_state.memory[address as usize] = current_state.registers.C;
        }
        0x72 => {
            // MOV M, D
            let address = current_state.registers.get_hl();
            current_state.memory[address as usize] = current_state.registers.D;
        }
        0x73 => {
            // MOV M, E
            let address = current_state.registers.get_hl();
            current_state.memory[address as usize] = current_state.registers.E;
        }
        0x74 => {
            // MOV M, H
            let address = current_state.registers.get_hl();
            current_state.memory[address as usize] = current_state.registers.H;
        }
        0x75 => {
            // MOV M, L
            let address = current_state.registers.get_hl();
            current_state.memory[address as usize] = current_state.registers.L;
        }
        0x76 => {
            // HLT
            current_state.cpu_state = CPUState::HALTED;
        }
        0x77 => {
            // MOV M, A
            let address = current_state.registers.get_hl();
            current_state.memory[address as usize] = current_state.registers.A;
        }
        0x78 => {
            // MOV A, B
            current_state.registers.A = current_state.registers.B;
        }
        0x79 => {
            // MOV A, C
            current_state.registers.A = current_state.registers.C;
        }
        0x7a => {
            // MOV A, D
            current_state.registers.A = current_state.registers.D;
        }
        0x7b => {
            // MOV A, E
            current_state.registers.A = current_state.registers.E;
        }
        0x7c => {
            // MOV A, H
            current_state.registers.A = current_state.registers.H;
        }
        0x7d => {
            // MOV A, L
            current_state.registers.A = current_state.registers.L;
        }
        0x7e => {
            // MOV A, M
            let address = current_state.registers.get_hl();
            current_state.registers.A = current_state.memory[address as usize];
        }
        0x7f => {
            // MOV A, A
            current_state.registers.A = current_state.registers.A;
        }
        0x80 => {
            // ADD B
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.B,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x81 => {
            // ADD C
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.C,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x82 => {
            // ADD D
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.D,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x83 => {
            // ADD E
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.E,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x84 => {
            // ADD H
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.H,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x85 => {
            // ADD L
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.L,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x86 => {
            // ADD M
            let address = current_state.registers.get_hl();
            let value = current_state.memory[address as usize];
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                value,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x87 => {
            // ADD A
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.A,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x88 => {
            // ADC B
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.B,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x89 => {
            // ADC C
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.C,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x8a => {
            // ADC D
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.D,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x8b => {
            // ADC E
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.E,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x8c => {
            // ADC H
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.H,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x8d => {
            // ADC L
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.L,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x8e => {
            // ADC M
            let address = current_state.registers.get_hl();
            let value = current_state.memory[address as usize];
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                value,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x8f => {
            // ADC A
            current_state.registers.A = add_instruction(
                current_state.registers.A,
                current_state.registers.A,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x90 => {
            // SUB B
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.B,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x91 => {
            // SUB C
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.C,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x92 => {
            // SUB D
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.D,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x93 => {
            // SUB E
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.E,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x94 => {
            // SUB H
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.H,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x95 => {
            // SUB L
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.L,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x96 => {
            // SUB M
            let address = current_state.registers.get_hl();
            let value = current_state.memory[address as usize];
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                value,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x97 => {
            // SUB A
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.A,
                false,
                &mut current_state.condition_bits,
            );
        }
        0x98 => {
            // SBB B
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.B,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x99 => {
            // SBB C
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.C,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x9a => {
            // SBB D
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.D,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x9b => {
            // SBB E
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.E,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x9c => {
            // SBB H
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.H,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x9d => {
            // SBB L
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.L,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x9e => {
            // SBB M
            let address = current_state.registers.get_hl();
            let value = current_state.memory[address as usize];
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                value,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0x9f => {
            // SBB A
            current_state.registers.A = sub_instruction(
                current_state.registers.A,
                current_state.registers.A,
                current_state.condition_bits.c(),
                &mut current_state.condition_bits,
            );
        }
        0xa0 => {
            // ANA B
            current_state.registers.A = and_instruction(
                current_state.registers.A,
                current_state.registers.B,
                &mut current_state.condition_bits,
            );
        }
        0xa1 => {
            // ANA C
            current_state.registers.A = and_instruction(
                current_state.registers.A,
                current_state.registers.C,
                &mut current_state.condition_bits,
            );
        }
        0xa2 => {
            // ANA D
            current_state.registers.A = and_instruction(
                current_state.registers.A,
                current_state.registers.D,
                &mut current_state.condition_bits,
            );
        }
        0xa3 => {
            // ANA E
            current_state.registers.A = and_instruction(
                current_state.registers.A,
                current_state.registers.E,
                &mut current_state.condition_bits,
            );
        }
        0xa4 => {
            // ANA H
            current_state.registers.A = and_instruction(
                current_state.registers.A,
                current_state.registers.H,
                &mut current_state.condition_bits,
            );
        }
        0xa5 => {
            // ANA L
            current_state.registers.A = and_instruction(
                current_state.registers.A,
                current_state.registers.L,
                &mut current_state.condition_bits,
            );
        }
        0xa6 => {
            // ANA M
            let address = current_state.registers.get_hl();
            let value = current_state.memory[address as usize];
            current_state.registers.A = and_instruction(
                current_state.registers.A,
                value,
                &mut current_state.condition_bits,
            );
        }
        0xa7 => {
            // ANA A
            current_state.registers.A = and_instruction(
                current_state.registers.A,
                current_state.registers.A,
                &mut current_state.condition_bits,
            );
        }
        0xa8 => {
            // XRA B
            current_state.registers.A = xor_instruction(
                current_state.registers.A,
                current_state.registers.B,
                &mut current_state.condition_bits,
            );
        }
        0xa9 => {
            // XRA C
            current_state.registers.A = xor_instruction(
                current_state.registers.A,
                current_state.registers.C,
                &mut current_state.condition_bits,
            );
        }
        0xaa => {
            // XRA D
            current_state.registers.A = xor_instruction(
                current_state.registers.A,
                current_state.registers.D,
                &mut current_state.condition_bits,
            );
        }
        0xab => {
            // XRA E
            current_state.registers.A = xor_instruction(
                current_state.registers.A,
                current_state.registers.E,
                &mut current_state.condition_bits,
            );
        }
        0xac => {
            // XRA H
            current_state.registers.A = xor_instruction(
                current_state.registers.A,
                current_state.registers.H,
                &mut current_state.condition_bits,
            );
        }
        0xad => {
            // XRA L
            current_state.registers.A = xor_instruction(
                current_state.registers.A,
                current_state.registers.L,
                &mut current_state.condition_bits,
            );
        }
        0xae => {
            // XRA M
            let address = current_state.registers.get_hl();
            let value = current_state.memory[address as usize];
            current_state.registers.A = xor_instruction(
                current_state.registers.A,
                value,
                &mut current_state.condition_bits,
            );
        }
        0xaf => {
            // XRA A
            current_state.registers.A = xor_instruction(
                current_state.registers.A,
                current_state.registers.A,
                &mut current_state.condition_bits,
            );
        }
        0xb0 => {
            // ORA B
            current_state.registers.A = or_instruction(
                current_state.registers.A,
                current_state.registers.B,
                &mut current_state.condition_bits,
            );
        }
        0xb1 => {
            // ORA C
            current_state.registers.A = or_instruction(
                current_state.registers.A,
                current_state.registers.C,
                &mut current_state.condition_bits,
            );
        }
        0xb2 => {
            // ORA D
            current_state.registers.A = or_instruction(
                current_state.registers.A,
                current_state.registers.D,
                &mut current_state.condition_bits,
            );
        }
        0xb3 => {
            // ORA E
            current_state.registers.A = or_instruction(
                current_state.registers.A,
                current_state.registers.E,
                &mut current_state.condition_bits,
            );
        }
        0xb4 => {
            // ORA H
            current_state.registers.A = or_instruction(
                current_state.registers.A,
                current_state.registers.H,
                &mut current_state.condition_bits,
            );
        }
        0xb5 => {
            // ORA L
            current_state.registers.A = or_instruction(
                current_state.registers.A,
                current_state.registers.L,
                &mut current_state.condition_bits,
            );
        }
        0xb6 => {
            // ORA M
            let address = current_state.registers.get_hl();
            let value = current_state.memory[address as usize];
            current_state.registers.A = or_instruction(
                current_state.registers.A,
                value,
                &mut current_state.condition_bits,
            );
        }
        0xb7 => {
            // ORA A
            current_state.registers.A = or_instruction(
                current_state.registers.A,
                current_state.registers.A,
                &mut current_state.condition_bits,
            );
        }
        0xb8 => {
            // CMP B - Performed as a subtraction that does not modify registers, only updates condition bits
            sub_instruction(
                current_state.registers.A,
                current_state.registers.B,
                false,
                &mut current_state.condition_bits,
            );
        }
        0xb9 => {
            // CMP C
            sub_instruction(
                current_state.registers.A,
                current_state.registers.C,
                false,
                &mut current_state.condition_bits,
            );
        }
        0xba => {
            // CMP D
            sub_instruction(
                current_state.registers.A,
                current_state.registers.D,
                false,
                &mut current_state.condition_bits,
            );
        }
        0xbb => {
            // CMP E
            sub_instruction(
                current_state.registers.A,
                current_state.registers.E,
                false,
                &mut current_state.condition_bits,
            );
        }
        0xbc => {
            // CMP H
            sub_instruction(
                current_state.registers.A,
                current_state.registers.H,
                false,
                &mut current_state.condition_bits,
            );
        }
        0xbd => {
            // CMP L
            sub_instruction(
                current_state.registers.A,
                current_state.registers.L,
                false,
                &mut current_state.condition_bits,
            );
        }
        0xbe => {
            // CMP M
            let address = current_state.registers.get_hl();
            let value = current_state.memory[address as usize];
            sub_instruction(
                current_state.registers.A,
                value,
                false,
                &mut current_state.condition_bits,
            );
        }
        0xbf => {
            // CMP A
            sub_instruction(
                current_state.registers.A,
                current_state.registers.A,
                false,
                &mut current_state.condition_bits,
            );
        }

        0xc3 => pc = (((code[2] as u16) << 8) | code[1] as u16) as usize,
        _ => undefined_instruction(code[0], pc as u16),
    }

    current_state.pc = pc as u16;
    current_state
}
