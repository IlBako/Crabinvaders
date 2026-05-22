use super::instructions::*;

#[allow(non_snake_case, unused)]
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
