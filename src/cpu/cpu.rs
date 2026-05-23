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

pub struct ConditionBits {
    bits: u8,
}

pub struct State {
    registers: Registers,
    memory: Vec<u8>,
    pub pc: u16,
    sp: u16,
    pub(crate) condition_bits: ConditionBits,
    // interrupted: bool,
}

impl State {
    fn new() -> State {
        State {
            registers: Registers::default(),
            memory: vec![0; 0x10000],
            pc: 0,
            sp: 0,
            condition_bits: ConditionBits::default(),
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
            let address =
                (current_state.registers.B as u16) << 8 | current_state.registers.C as u16;
            current_state.memory[address as usize] = current_state.registers.A;
        }
        0x03 => {
            // INX B
            let bc = (current_state.registers.B as u16) << 8 | current_state.registers.C as u16;
            let res = bc.wrapping_add(1);
            current_state.registers.B = (res >> 8) as u8;
            current_state.registers.C = res as u8;
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
            let hl = (current_state.registers.H as u16) << 8 | current_state.registers.L as u16;
            let bc = (current_state.registers.B as u16) << 8 | current_state.registers.C as u16;
            let (res, carry) = hl.overflowing_add(bc);
            current_state.registers.H = (res >> 8) as u8;
            current_state.registers.L = res as u8;
            current_state.condition_bits.set_c(carry);
        }
        0x0a => {
            // LDAX B
            let address =
                (current_state.registers.B as u16) << 8 | current_state.registers.C as u16;
            current_state.registers.A = current_state.memory[address as usize];
        }
        0x0b => {
            // DCX B
            let bc = (current_state.registers.B as u16) << 8 | current_state.registers.C as u16;
            let res = bc.wrapping_sub(1);
            current_state.registers.B = (res >> 8) as u8;
            current_state.registers.C = res as u8;
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
            let address =
                (current_state.registers.D as u16) << 8 | current_state.registers.E as u16;
            current_state.memory[address as usize] = current_state.registers.A;
        }
        0x13 => {
            // INX D
            let de = (current_state.registers.D as u16) << 8 | current_state.registers.E as u16;
            let res = de.wrapping_add(1);
            current_state.registers.D = (res >> 8) as u8;
            current_state.registers.E = res as u8;
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
            let hl = (current_state.registers.H as u16) << 8 | current_state.registers.L as u16;
            let de = (current_state.registers.D as u16) << 8 | current_state.registers.E as u16;
            let (res, carry) = hl.overflowing_add(de);
            current_state.registers.H = (res >> 8) as u8;
            current_state.registers.L = res as u8;
            current_state.condition_bits.set_c(carry);
        }
        0x1a => {
            // LDAX D
            let address =
                (current_state.registers.D as u16) << 8 | current_state.registers.E as u16;
            current_state.registers.A = current_state.memory[address as usize];
        }
        0x1b => {
            // DCX D
            let de = (current_state.registers.D as u16) << 8 | current_state.registers.E as u16;
            let res = de.wrapping_sub(1);
            current_state.registers.D = (res >> 8) as u8;
            current_state.registers.E = res as u8;
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
            current_state.memory[address as usize + 1] = current_state.registers.H;
            pc += 2
        }
        0x23 => {
            // INX H
            let hl = (current_state.registers.H as u16) << 8 | current_state.registers.L as u16;
            let res = hl.wrapping_add(1);
            current_state.registers.H = (res >> 8) as u8;
            current_state.registers.L = res as u8;
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
            if accumulator > 0x99 || old_c || step1_carry {
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

        0x31 => {
            current_state.sp = (code[2] as u16) << 8 | code[1] as u16;
            pc += 2
        }
        0xc3 => pc = (((code[2] as u16) << 8) | code[1] as u16) as usize,
        _ => undefined_instruction(code[0], pc as u16),
    }

    current_state.pc = pc as u16;
    current_state
}
