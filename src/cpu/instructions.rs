use super::*;

pub fn undefined_instruction(instruction: u8, pc: u16) {
    panic!(
        "Undefined instruction {:#04x} at {:#06x}",
        instruction,
        pc - 1
    )
}

fn inr_instruction(current_val: u8, condition_bits: &mut ConditionBits) -> u8 {
    let res = current_val.wrapping_add(1);
    condition_bits.set_z(res == 0);
    condition_bits.set_p(res.count_ones() % 2 == 0);
    condition_bits.set_s(res & 0x80 == 0x80);
    condition_bits.set_ac((current_val & 0x0F) == 0x0F);
    res
}

fn dcr_instruction(current_val: u8, condition_bits: &mut ConditionBits) -> u8 {
    let res = current_val.wrapping_sub(1);
    condition_bits.set_z(res == 0);
    condition_bits.set_p(res.count_ones() % 2 == 0);
    condition_bits.set_s(res & 0x80 == 0x80);
    condition_bits.set_ac((current_val & 0x0F) == 0x00);
    res
}

fn add_instruction(
    current_val: u8,
    value: u8,
    carry_in: bool,
    condition_bits: &mut ConditionBits,
) -> u8 {
    let sum: u16 = (current_val as u16) + (value as u16) + (carry_in as u16);
    let res = sum as u8;
    condition_bits.set_z(res == 0);
    condition_bits.set_c(sum > 0xFF);
    condition_bits.set_p(res.count_ones() % 2 == 0);
    condition_bits.set_s(res & 0x80 == 0x80);
    condition_bits.set_ac(((current_val & 0x0F) + (value & 0x0F) + (carry_in as u8)) > 0x0F);
    res
}

fn sub_instruction(
    current_val: u8,
    value: u8,
    borrow_in: bool,
    condition_bits: &mut ConditionBits,
) -> u8 {
    let subtrahend = (value as u16) + (borrow_in as u16);
    let (res, borrow) = (current_val as u16).overflowing_sub(subtrahend);

    condition_bits.set_z(res as u8 == 0);
    condition_bits.set_c(borrow); // carry = borrow for SUB/SBB
    condition_bits.set_p((res as u8).count_ones() % 2 == 0);
    condition_bits.set_s((res as u8) & 0x80 == 0x80);

    let ac = ((current_val & 0x0F) as i16) - ((value & 0x0F) as i16) - (borrow_in as i16) < 0;
    condition_bits.set_ac(ac);

    res as u8
}

fn and_instruction(current_val: u8, value: u8, condition_bits: &mut ConditionBits) -> u8 {
    let res = current_val & value;
    condition_bits.set_z(res == 0);
    condition_bits.set_c(false);
    condition_bits.set_ac(false);
    condition_bits.set_p(res.count_ones() % 2 == 0);
    condition_bits.set_s(res & 0x80 == 0x80);
    res
}

fn xor_instruction(current_val: u8, value: u8, condition_bits: &mut ConditionBits) -> u8 {
    let res = current_val ^ value;
    condition_bits.set_z(res == 0);
    condition_bits.set_c(false);
    condition_bits.set_ac(false);
    condition_bits.set_p(res.count_ones() % 2 == 0);
    condition_bits.set_s(res & 0x80 == 0x80);
    res
}

fn or_instruction(current_val: u8, value: u8, condition_bits: &mut ConditionBits) -> u8 {
    let res = current_val | value;
    condition_bits.set_z(res == 0);
    condition_bits.set_c(false);
    condition_bits.set_ac(false);
    condition_bits.set_p(res.count_ones() % 2 == 0);
    condition_bits.set_s(res & 0x80 == 0x80);
    res
}

impl Cpu {
    #[inline(always)]
    fn push(&mut self, bus: &mut Bus, val: u16) {
        self.sp = self.sp.wrapping_sub(2);
        bus.write_u16(self.sp, val);
    }

    #[inline(always)]
    fn pop(&mut self, bus: &mut Bus) -> u16 {
        let val = bus.read_u16(self.sp);
        self.sp = self.sp.wrapping_add(2);
        val
    }

    #[inline(always)]
    fn call(&mut self, bus: &mut Bus, off: u16) {
        self.push(bus, self.pc);
        self.pc = off;
    }

    #[inline(always)]
    fn ret(&mut self, bus: &mut Bus) {
        self.pc = self.pop(bus);
    }

    #[inline(always)]
    fn jp(&mut self, off: u16) {
        self.pc = off;
    }
}

macro_rules! get {
    ($self:expr, bc) => {$self.registers.get_bc()};
    ($self:expr, de) => {$self.registers.get_de()};
    ($self:expr, hl) => {$self.registers.get_hl()};

    ($self:expr, $reg:ident) => {
      $self.registers.$reg  
    };
}

macro_rules! set {
    ($self:expr, bc, $val: expr) => {$self.registers.set_bc($val)};
    ($self:expr, de, $val: expr) => {$self.registers.set_de($val)};
    ($self:expr, hl, $val: expr) => {$self.registers.set_hl($val)};

    ($self:expr, $reg:ident, $val:expr) => {
      $self.registers.$reg = $val
    };
}

impl Cpu {
    pub(super) fn run_instr(&mut self, bus: &mut Bus) {
        self.cycles += 4;
        match self.fetch_u8(bus) {
            0x00 => {/* NOP */},
            0x01 => {
                // LXI B, D16    
                let val = self.fetch_u16(bus);
                set!(self, bc, val);
                self.cycles += 6;
            },
            0x02 => {
                // STAX B
                bus.write_u8(get!(self, bc), get!(self, A));
                self.cycles += 3;
            }
            0x03 => {
                // INX B
                set!(self, bc, get!(self, bc).wrapping_add(1));
                self.cycles += 1;
            }
            0x04 => {
                //INR B
                set!(self, B, inr_instruction(get!(self, B), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x05 => {
                //DCR B
                set!(self, B, dcr_instruction(get!(self, B), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x06 => {
                // MVI B, D8
                set!(self, B, self.fetch_u8(bus));
                self.cycles += 3;
            }
            0x07 => {
                // RLC
                let val = get!(self, A);
                self.condition_bits.set_c(val >> 7 == 1);
                set!(self, A, (val << 1) | (val >> 7));
            }
            0x08 => { /* NOP */}
            0x09 => {
                // DAD B
                let hl = get!(self, hl);
                let bc = get!(self, bc);
                let (res, carry) = hl.overflowing_add(bc);
                set!(self, hl, res);
                self.condition_bits.set_c(carry);
                self.cycles += 6;
            }
            0x0A => {
                // LDAX B
                set!(self, A, bus.read_u8(get!(self, bc)));
                self.cycles += 3;
            }
            0x0B => {
                // DCX B
                set!(self, bc, get!(self, bc).wrapping_sub(1));
                self.cycles += 1;
            }
            0x0C => {
                // INR C
                set!(self, C, inr_instruction(get!(self, C), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x0D => {
                // DCR C
                set!(self, C, dcr_instruction(get!(self, C), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x0E => {
                // MVI C, D8
                set!(self, C, self.fetch_u8(bus));
                self.cycles += 3;
            }
            0x0F => {
                // RRC
                let val = get!(self, A);
                self.condition_bits.set_c(val & 0x01 == 1);
                set!(self, A, (val >> 1) | (val << 7));
            }
            0x10 => { /* NOP */}
            0x11 => {
                // LXI D, D16
                let val = self.fetch_u16(bus);
                set!(self, de, val);
                self.cycles += 6;
            }
            0x12 => {
                // STAX D
                bus.write_u8(get!(self, de), get!(self, A));
                self.cycles += 3;
            }
            0x13 => {
                // INX D
                set!(self, de, get!(self, de).wrapping_add(1));
                self.cycles += 1;
            }
            0x14 => {
                // INR D
                set!(self, D, inr_instruction(get!(self, D), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x15 => {
                // DCR D
                set!(self, D, dcr_instruction(get!(self, D), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x16 => {
                // MVI D, D8
                set!(self, D, self.fetch_u8(bus));
                self.cycles += 3;
            }
            0x17 => {
                // RAL
                let val = get!(self, A);
                set!(self, A, (val << 1) | (self.condition_bits.c() as u8));
                self.condition_bits.set_c(val >> 7 == 1);
            }
            0x18 => { /* NOP */}
            0x19 => {
                // DAD D
                let hl = get!(self, hl);
                let de = get!(self, de);
                let (res, carry) = hl.overflowing_add(de);
                set!(self, hl, res);
                self.condition_bits.set_c(carry);
                self.cycles += 6;
            }
            0x1A => {
                // LDAX D
                set!(self, A, bus.read_u8(get!(self, de)));
                self.cycles += 3;
            }
            0x1B => {
                // DCX D
                set!(self, de, get!(self, de).wrapping_sub(1));
                self.cycles += 1;
            }
            0x1C => {
                // INR E
                set!(self, E, inr_instruction(get!(self, E), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x1D => {
                // DCR E
                set!(self, E, dcr_instruction(get!(self, E), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x1E => {
                // MVI E, D8
                set!(self, E, self.fetch_u8(bus));
                self.cycles += 3;
            }
            0x1F => {
                // RAR
                let val = get!(self, A);
                set!(self, A, (val >> 1) | ((self.condition_bits.c() as u8) << 7));
                self.condition_bits.set_c(val & 0x01 == 1);
            }
            0x20 => { /* NOP */}
            0x21 => {
                // LXI H, D16
                let val = self.fetch_u16(bus);
                set!(self, hl, val);
                self.cycles += 6;
            }
            0x22 => {
                // SHLD addr
                let addr = self.fetch_u16(bus);
                bus.write_u16(addr, get!(self, hl));
                self.cycles += 12
            }
            0x23 => {
                // INX H
                set!(self, hl, get!(self, hl).wrapping_add(1));
                self.cycles += 1;
            }
            0x24 => {
                // INR H
                set!(self, H, inr_instruction(get!(self, H), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x25 => {
                // DCR H
                set!(self, H, dcr_instruction(get!(self, H), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x26 => {
                // MVI H, D8
                set!(self, H, self.fetch_u8(bus));
                self.cycles += 3;
            }
            0x27 => {
                // DAA
                let mut accumulator = get!(self, A);
                let ac = self.condition_bits.ac();
                let old_c = self.condition_bits.c();
                let mut step1_carry = false;
                let mut step2_carry = false;

                let lower_nibble = accumulator & 0x0F;
                if lower_nibble > 9 || ac {
                    self.condition_bits
                        .set_ac((lower_nibble + 0x06) > 0x0F);
                    (accumulator, step1_carry) = accumulator.overflowing_add(6);
                } else {
                    self.condition_bits.set_ac(false);
                }
                let upper_nibble = accumulator >> 4;
                if upper_nibble > 9 || old_c || step1_carry {
                    (accumulator, step2_carry) = accumulator.overflowing_add(0x60);
                }
                self
                    .condition_bits
                    .set_c(step1_carry | step2_carry | old_c);
                self.condition_bits.set_z(accumulator == 0);
                self
                    .condition_bits
                    .set_p(accumulator.count_ones() % 2 == 0);
                self
                    .condition_bits
                    .set_s(accumulator & 0x80 == 0x80);
                set!(self, A, accumulator);
            }
            0x28 => { /* NOP */}
            0x29 => {
                // DAD H
                let hl = get!(self, hl);
                let res = hl << 1;
                set!(self, hl, res);
                self.condition_bits.set_c(res < hl);
                self.cycles += 6;
            }
            0x2A => {
                // LHLD adr
                let addr = self.fetch_u16(bus);
                set!(self, hl, bus.read_u16(addr));
                self.cycles += 12;
            }
            0x2B => {
                // DCX H
                set!(self, hl, get!(self, hl).wrapping_sub(1));
                self.cycles += 1;
            }
            0x2C => {
                // INR L
                set!(self, L, inr_instruction(get!(self, L), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x2D => {
                // DCR L
                set!(self, L, dcr_instruction(get!(self, L), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x2E => {
                // MVI L, D8
                set!(self, L, self.fetch_u8(bus));
                self.cycles += 3;
            }
            0x2F => {
                // CMA
                set!(self, A, !(get!(self, A)));
            }
            0x30 => { /* NOP */ }
            0x31 => {
                // LXI SP, D16
                self.sp = self.fetch_u16(bus);  
                self.cycles += 6;
            }
            0x32 => {
                // STA  adr
                let addr = self.fetch_u16(bus);
                bus.write_u8(addr, get!(self, A));
                self.cycles += 9;
            }
            0x33 => {
                // INX SP
                self.sp = self.sp.wrapping_add(1);
                self.cycles += 1;
            }
            0x34 => {
                // INR M
                let addr = get!(self, hl);
                let val = bus.read_u8(addr);
                bus.write_u8(addr, inr_instruction(val, &mut self.condition_bits));
                self.cycles += 6;
            }
            0x35 => {
                // DCR M
                let addr = get!(self, hl);
                let val = bus.read_u8(addr);
                bus.write_u8(addr, dcr_instruction(val, &mut self.condition_bits));
                self.cycles += 6;
            }
            0x36 => {
                // MVI M, D8
                bus.write_u8(get!(self, hl), self.fetch_u8(bus));
                self.cycles += 6;
            }
            0x37 => {
                // STC
                self.condition_bits.set_c(true);
            }
            0x38 => { /* NOP */ }
            0x39 => {
                // DAD SP
                let hl = get!(self, hl);
                let (res, carry) = hl.overflowing_add(self.sp);
                set!(self, hl, res);
                self.condition_bits.set_c(carry);
                self.cycles += 6;
            }
            0x3A => {
                // LDA adr
                let addr = self.fetch_u16(bus);
                set!(self, A, bus.read_u8(addr));
                self.cycles += 9;
            }
            0x3B => {
                // DCX SP
                self.sp = self.sp.wrapping_sub(1);
                self.cycles += 1;
            }
            0x3C => {
                // INR A
                set!(self, A, inr_instruction(get!(self, A), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x3D => {
                // DCR A
                set!(self, A, dcr_instruction(get!(self, A), &mut self.condition_bits));
                self.cycles += 1;
            }
            0x3E => {
                // MVI A, D8
                set!(self, A, self.fetch_u8(bus));
                self.cycles += 3;
            }
            0x3F => {
                self.condition_bits.set_c(!self.condition_bits.c());
            }
            0x40 => {
                // MOV B, B
                set!(self, B, get!(self, B));
                self.cycles += 1;
            }
            0x41 => {
                // MOV B, C
                set!(self, B, get!(self, C));
                self.cycles += 1;
            }
            0x42 => {
                // MOV B, D
                set!(self, B, get!(self, D));
                self.cycles += 1;
            }
            0x43 => {
                // MOV B, E
                set!(self, B, get!(self, E));
                self.cycles += 1;
            }
            0x44 => {
                // MOV B, H
                set!(self, B, get!(self, H));
                self.cycles += 1;
            }
            0x45 => {
                // MOV B, L
                set!(self, B, get!(self, L));
                self.cycles += 1;
            }
            0x46 => {
                // MOV B, M
                set!(self, B, bus.read_u8(get!(self, hl)));
                self.cycles += 3;
            }
            0x47 => {
                // MOV B, A
                set!(self, B, get!(self, A));
                self.cycles += 1;
            }
            0x48 => {
                // MOV C, B
                set!(self, C, get!(self, B));
                self.cycles += 1;
            }
            0x49 => {
                // MOV C, C
                set!(self, C, get!(self, C));
                self.cycles += 1;
            }
            0x4A => {
                // MOV C, D
                set!(self, C, get!(self, D));
                self.cycles += 1;
            }
            0x4B => {
                // MOV C, E
                set!(self, C, get!(self, E));
                self.cycles += 1;
            }
            0x4C => {
                // MOV C, H
                set!(self, C, get!(self, H));
                self.cycles += 1;
            }
            0x4D => {
                // MOV C, L
                set!(self, C, get!(self, L));
                self.cycles += 1;
            }
            0x4E => {
                // MOV C, M
                set!(self, C, bus.read_u8(get!(self, hl)));
                self.cycles += 3;
            }
            0x4F => {
                // MOV C, A
                set!(self, C, get!(self, A));
                self.cycles += 1;
            }
            0x50 => {
                // MOV D, B
                set!(self, D, get!(self, B));
                self.cycles += 1;
            }
            0x51 => {
                // MOV D, C
                set!(self, D, get!(self, C));
                self.cycles += 1;
            }
            0x52 => {
                // MOV D, D
                set!(self, D, get!(self, D));
                self.cycles += 1;
            }
            0x53 => {
                // MOV D, E
                set!(self, D, get!(self, E));
                self.cycles += 1;
            }
            0x54 => {
                // MOV D, H
                set!(self, D, get!(self, H));
                self.cycles += 1;
            }
            0x55 => {
                // MOV D, L
                set!(self, D, get!(self, L));
                self.cycles += 1;
            }
            0x56 => {
                // MOV D, M
                set!(self, D, bus.read_u8(get!(self, hl)));
                self.cycles += 3;
            }
            0x57 => {
                // MOV D, A
                set!(self, D, get!(self, A));
                self.cycles += 1;
            }
            0x58 => {
                // MOV E, B
                set!(self, E, get!(self, B));
                self.cycles += 1;
            }
            0x59 => {
                // MOV E, C
                set!(self, E, get!(self, C));
                self.cycles += 1;
            }
            0x5A => {
                // MOV E, D
                set!(self, E, get!(self, D));
                self.cycles += 1;
            }
            0x5B => {
                // MOV E, E
                set!(self, E, get!(self, E));
                self.cycles += 1;
            }
            0x5C => {
                // MOV E, H
                set!(self, E, get!(self, H));
                self.cycles += 1;
            }
            0x5D => {
                // MOV E, L
                set!(self, E, get!(self, L));
                self.cycles += 1;
            }
            0x5E => {
                // MOV E, M
                set!(self, E, bus.read_u8(get!(self, hl)));
                self.cycles += 3;
            }
            0x5F => {
                // MOV E, A
                set!(self, E, get!(self, A));
                self.cycles += 1;
            }
            0x60 => {
                // MOV H, B
                set!(self, H, get!(self, B));
                self.cycles += 1;
            }
            0x61 => {
                // MOV H, C
                set!(self, H, get!(self, C));
                self.cycles += 1;
            }
            0x62 => {
                // MOV H, D
                set!(self, H, get!(self, D));
                self.cycles += 1;
            }
            0x63 => {
                // MOV H, E
                set!(self, H, get!(self, E));
                self.cycles += 1;
            }
            0x64 => {
                // MOV H, H
                set!(self, H, get!(self, H));
                self.cycles += 1;
            }
            0x65 => {
                // MOV H, L
                set!(self, H, get!(self, L));
                self.cycles += 1;
            }
            0x66 => {
                // MOV H, M
                set!(self, H, bus.read_u8(get!(self, hl)));
                self.cycles += 3;
            }
            0x67 => {
                // MOV H, A
                set!(self, H, get!(self, A));
                self.cycles += 1;
            }
            0x68 => {
                // MOV L, B
                set!(self, L, get!(self, B));
                self.cycles += 1;
            }
            0x69 => {
                // MOV L, C
                set!(self, L, get!(self, C));
                self.cycles += 1;
            }
            0x6A => {
                // MOV L, D
                set!(self, L, get!(self, D));
                self.cycles += 1;
            }
            0x6B => {
                // MOV L, E
                set!(self, L, get!(self, E));
                self.cycles += 1;
            }
            0x6C => {
                // MOV L, H
                set!(self, L, get!(self, H));
                self.cycles += 1;
            }
            0x6D => {
                // MOV L, L
                set!(self, L, get!(self, L));
                self.cycles += 1;
            }
            0x6E => {
                // MOV L, M
                set!(self, L, bus.read_u8(get!(self, hl)));
                self.cycles += 3;
            }
            0x6F => {
                // MOV L, A
                set!(self, L, get!(self, A));
                self.cycles += 1;
            }
            0x70 => {
                // MOV M, B
                bus.write_u8(get!(self, hl), get!(self, B));
                self.cycles += 3;
            }
            0x71 => {
                // MOV M, C
                bus.write_u8(get!(self, hl), get!(self, C));
                self.cycles += 3;
            }
            0x72 => {
                // MOV M, D
                bus.write_u8(get!(self, hl), get!(self, D));
                self.cycles += 3;
            }
            0x73 => {
                // MOV M, E
                bus.write_u8(get!(self, hl), get!(self, E));
                self.cycles += 3;
            }
            0x74 => {
                // MOV M, H
                bus.write_u8(get!(self, hl), get!(self, H));
                self.cycles += 3;
            }
            0x75 => {
                // MOV M, L
                bus.write_u8(get!(self, hl), get!(self, L));
                self.cycles += 3;
            }
            0x76 => {
                // HLT
                if self.ime {
                    self.halt = true;
                } else {
                    // Interrupts are disable, do not halt
                }
            }
            0x77 => {
                // MOV M, A
                bus.write_u8(get!(self, hl), get!(self, A));
                self.cycles += 3;
            }
            0x78 => {
                // MOV A, B
                set!(self, A, get!(self, B));
                self.cycles += 1;
            }
            0x79 => {
                // MOV A, C
                set!(self, A, get!(self, C));
                self.cycles += 1;
            }
            0x7A => {
                // MOV A, D
                set!(self, A, get!(self, D));
                self.cycles += 1;
            }
            0x7B => {
                // MOV A, E
                set!(self, A, get!(self, E));
                self.cycles += 1;
            }
            0x7C => {
                // MOV A, H
                set!(self, A, get!(self, H));
                self.cycles += 1;
            }
            0x7D => {
                // MOV A, L
                set!(self, A, get!(self, L));
                self.cycles += 1;
            }
            0x7E => {
                // MOV A, M
                set!(self, A, bus.read_u8(get!(self, hl)));
                self.cycles += 3;
            }
            0x7F => {
                // MOV A, A
                set!(self, A, get!(self, A));
                self.cycles += 1;
            }
            0x80 => {
                // ADD B
                set!(self, A, add_instruction(get!(self, A), get!(self, B), false, &mut self.condition_bits));
            }
            0x81 => {
                // ADD C
                set!(self, A, add_instruction(get!(self, A), get!(self, C), false, &mut self.condition_bits));
            }
            0x82 => {
                // ADD D
                set!(self, A, add_instruction(get!(self, A), get!(self, D), false, &mut self.condition_bits));
            }
            0x83 => {
                // ADD E
                set!(self, A, add_instruction(get!(self, A), get!(self, E), false, &mut self.condition_bits));
            }
            0x84 => {
                // ADD H
                set!(self, A, add_instruction(get!(self, A), get!(self, H), false, &mut self.condition_bits));
            }
            0x85 => {
                // ADD L
                set!(self, A, add_instruction(get!(self, A), get!(self, L), false, &mut self.condition_bits));
            }
            0x86 => {
                // ADD M
                set!(self, A, add_instruction(get!(self, A), bus.read_u8(get!(self, hl)), false, &mut self.condition_bits));
                self.cycles += 3;
            }
            0x87 => {
                // ADD A
                set!(self, A, add_instruction(get!(self, A), get!(self, A), false, &mut self.condition_bits));
            }
            0x88 => {
                // ADC B
                set!(self, A, add_instruction(get!(self, A), get!(self, B), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x89 => {
                // ADC C
                set!(self, A, add_instruction(get!(self, A), get!(self, C), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x8A => {
                // ADC D
                set!(self, A, add_instruction(get!(self, A), get!(self, D), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x8B => {
                // ADC E
                set!(self, A, add_instruction(get!(self, A), get!(self, E), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x8C => {
                // ADC H
                set!(self, A, add_instruction(get!(self, A), get!(self, H), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x8D => {
                // ADC L
                set!(self, A, add_instruction(get!(self, A), get!(self, L), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x8E => {
                // ADC M
                set!(self, A, add_instruction(get!(self, A), bus.read_u8(get!(self, hl)), self.condition_bits.c(), &mut self.condition_bits));
                self.cycles += 3;
            }
            0x8F => {
                // ADC A
                set!(self, A, add_instruction(get!(self, A), get!(self, A), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x90 => {
                // SUB B
                set!(self, A, sub_instruction(get!(self, A), get!(self, B), false, &mut self.condition_bits));
            }
            0x91 => {
                // SUB C
                set!(self, A, sub_instruction(get!(self, A), get!(self, C), false, &mut self.condition_bits));
            }
            0x92 => {
                // SUB D
                set!(self, A, sub_instruction(get!(self, A), get!(self, D), false, &mut self.condition_bits));
            }
            0x93 => {
                // SUB E
                set!(self, A, sub_instruction(get!(self, A), get!(self, E), false, &mut self.condition_bits));
            }
            0x94 => {
                // SUB H
                set!(self, A, sub_instruction(get!(self, A), get!(self, H), false, &mut self.condition_bits));
            }
            0x95 => {
                // SUB L
                set!(self, A, sub_instruction(get!(self, A), get!(self, L), false, &mut self.condition_bits));
            }
            0x96 => {
                // SUB M
                set!(self, A, sub_instruction(get!(self, A), bus.read_u8(get!(self, hl)), false, &mut self.condition_bits));
                self.cycles += 3;
            }
            0x97 => {
                // SUB A
                set!(self, A, sub_instruction(get!(self, A), get!(self, A), false, &mut self.condition_bits));
            }
            0x98 => {
                // SBB B
                set!(self, A, sub_instruction(get!(self, A), get!(self, B), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x99 => {
                // SBB C
                set!(self, A, sub_instruction(get!(self, A), get!(self, C), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x9A => {
                // SBB D
                set!(self, A, sub_instruction(get!(self, A), get!(self, D), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x9B => {
                // SBB E
                set!(self, A, sub_instruction(get!(self, A), get!(self, E), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x9C => {
                // SBB H
                set!(self, A, sub_instruction(get!(self, A), get!(self, H), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x9D => {
                // SBB L
                set!(self, A, sub_instruction(get!(self, A), get!(self, L), self.condition_bits.c(), &mut self.condition_bits));
            }
            0x9E => {
                // SBB M
                set!(self, A, sub_instruction(get!(self, A), bus.read_u8(get!(self, hl)), self.condition_bits.c(), &mut self.condition_bits));
                self.cycles += 3;
            }
            0x9F => {
                // SBB A
                set!(self, A, sub_instruction(get!(self, A), get!(self, A), self.condition_bits.c(), &mut self.condition_bits));
            }
            0xA0 => {
                // ANA B
                set!(self, A, and_instruction(get!(self, A), get!(self, B), &mut self.condition_bits));
            }
            0xA1 => {
                // ANA C
                set!(self, A, and_instruction(get!(self, A), get!(self, C), &mut self.condition_bits));
            }
            0xA2 => {
                // ANA D
                set!(self, A, and_instruction(get!(self, A), get!(self, D), &mut self.condition_bits));
            }
            0xA3 => {
                // ANA E
                set!(self, A, and_instruction(get!(self, A), get!(self, E), &mut self.condition_bits));
            }
            0xA4 => {
                // ANA H
                set!(self, A, and_instruction(get!(self, A), get!(self, H), &mut self.condition_bits));
            }
            0xA5 => {
                // ANA L
                set!(self, A, and_instruction(get!(self, A), get!(self, L), &mut self.condition_bits));
            }
            0xA6 => {
                // ANA M
                set!(self, A, and_instruction(get!(self, A), bus.read_u8(get!(self, hl)), &mut self.condition_bits));
                self.cycles += 3;
            }
            0xA7 => {
                // ANA A
                set!(self, A, and_instruction(get!(self, A), get!(self, A), &mut self.condition_bits));
            }
            0xA8 => {
                // XRA B
                set!(self, A, xor_instruction(get!(self, A), get!(self, B), &mut self.condition_bits));
            }
            0xA9 => {
                // XRA C
                set!(self, A, xor_instruction(get!(self, A), get!(self, C), &mut self.condition_bits));
            }
            0xAA => {
                // XRA D
                set!(self, A, xor_instruction(get!(self, A), get!(self, D), &mut self.condition_bits));
            }
            0xAB => {
                // XRA E
                set!(self, A, xor_instruction(get!(self, A), get!(self, E), &mut self.condition_bits));
            }
            0xAC => {
                // XRA H
                set!(self, A, xor_instruction(get!(self, A), get!(self, H), &mut self.condition_bits));
            }
            0xAD => {
                // XRA L
                set!(self, A, xor_instruction(get!(self, A), get!(self, L), &mut self.condition_bits));
            }
            0xAE => {
                // XRA M
                set!(self, A, xor_instruction(get!(self, A), bus.read_u8(get!(self, hl)), &mut self.condition_bits));
                self.cycles += 3;
            }
            0xAF => {
                // XRA A
                set!(self, A, xor_instruction(get!(self, A), get!(self, A), &mut self.condition_bits));
            }
            0xB0 => {
                // ORA B
                set!(self, A, or_instruction(get!(self, A), get!(self, B), &mut self.condition_bits));
            }
            0xB1 => {
                // ORA C
                set!(self, A, or_instruction(get!(self, A), get!(self, C), &mut self.condition_bits));
            }
            0xB2 => {
                // ORA D
                set!(self, A, or_instruction(get!(self, A), get!(self, D), &mut self.condition_bits));
            }
            0xB3 => {
                // ORA E
                set!(self, A, or_instruction(get!(self, A), get!(self, E), &mut self.condition_bits));
            }
            0xB4 => {
                // ORA H
                set!(self, A, or_instruction(get!(self, A), get!(self, H), &mut self.condition_bits));
            }
            0xB5 => {
                // ORA L
                set!(self, A, or_instruction(get!(self, A), get!(self, L), &mut self.condition_bits));
            }
            0xB6 => {
                // ORA M
                set!(self, A, or_instruction(get!(self, A), bus.read_u8(get!(self, hl)), &mut self.condition_bits));
                self.cycles += 3;
            }
            0xB7 => {
                // ORA A
                set!(self, A, or_instruction(get!(self, A), get!(self, A), &mut self.condition_bits));
            }
            0xB8 => {
                // CMP B
                sub_instruction(get!(self, A), get!(self, B), false, &mut self.condition_bits);
            }
            0xB9 => {
                // CMP C
                sub_instruction(get!(self, A), get!(self, C), false, &mut self.condition_bits);
            }
            0xBA => {
                // CMP D
                sub_instruction(get!(self, A), get!(self, D), false, &mut self.condition_bits);
            }
            0xBB => {
                // CMP E
                sub_instruction(get!(self, A), get!(self, E), false, &mut self.condition_bits);
            }
            0xBC => {
                // CMP H
                sub_instruction(get!(self, A), get!(self, H), false, &mut self.condition_bits);
            }
            0xBD => {
                // CMP L
                sub_instruction(get!(self, A), get!(self, L), false, &mut self.condition_bits);
            }
            0xBE => {
                // CMP M
                sub_instruction(get!(self, A), bus.read_u8(get!(self, hl)), false, &mut self.condition_bits);
                self.cycles += 3;
            }
            0xBF => {
                // CMP A
                sub_instruction(get!(self, A), get!(self, A), false, &mut self.condition_bits);
            }
            0xC0 => {
                // RNZ
                if !self.condition_bits.z() {
                    self.ret(bus);
                    self.cycles += 6; 
                }
                self.cycles += 1;
            }
            0xC1 => {
                // POP B
                let val = self.pop(bus);
                set!(self, bc, val);
                self.cycles += 6;
            }
            0xC2 => {
                // JNZ adr
                let off = self.fetch_u16(bus);
                if !self.condition_bits.z() {
                    self.jp(off);
                }
                self.cycles += 6;
            }
            0xC3 => {
                // JMP adr
                let off = self.fetch_u16(bus);
                self.jp(off);
                self.cycles += 6;
            }
            0xC4 => {
                // CNZ adr
                let off = self.fetch_u16(bus);
                if !self.condition_bits.z() {
                    self.call(bus, off);
                    self.cycles += 6;
                }
                self.cycles += 7;
            }
            0xC5 => {
                // PUSH B
                self.push(bus, get!(self, bc));
                self.cycles += 7;
            }
            0xC6 => {
                // ADI d8
                set!(self, A, add_instruction(get!(self, A), self.fetch_u8(bus), false, &mut self.condition_bits));
                self.cycles += 3;
            }
            0xC7 => {
                // RST 0
                let off = 0x0000;
                self.call(bus, off);
                self.cycles += 7;
            }
            0xC8 => {
                // RZ
                if self.condition_bits.z() {
                    self.ret(bus);
                    self.cycles += 6; 
                }
                self.cycles += 1;
            }
            0xC9 => {
                // RET
                self.ret(bus);
                self.cycles += 6;
            }
            0xCA => {
                // JZ adr
                let off = self.fetch_u16(bus);
                if self.condition_bits.z() {
                    self.jp(off);
                }
                self.cycles += 6;
            }
            0xCB => {
                // JMP adr (alternate)
                let off = self.fetch_u16(bus);
                self.jp(off);
                self.cycles += 6;
            }
            0xCC => {
                // CZ adr
                let off = self.fetch_u16(bus);
                if self.condition_bits.z() {
                    self.call(bus, off);
                    self.cycles += 6;
                }
                self.cycles += 7;
            }
            0xCD => {
                // CALL adr
                let off = self.fetch_u16(bus);
                self.call(bus, off);
                self.cycles += 13;
            }
            0xCE => {
                // ACI d8
                set!(self, A, add_instruction(get!(self, A), self.fetch_u8(bus), self.condition_bits.c(), &mut self.condition_bits));
                self.cycles += 3;
            }
            0xCF => {
                // RST 1
                let off = 0x0008;
                self.call(bus, off);
                self.cycles += 7;
            }
            0xD0 => {
                // RNC
                if !self.condition_bits.c() {
                    self.ret(bus);
                    self.cycles += 6;
                }
                self.cycles += 1;
            }
            0xD1 => {
                // POP D
                let val = self.pop(bus);
                set!(self, de, val);
                self.cycles += 6;
            }
            0xD2 => {
                // JNC adr
                let off = self.fetch_u16(bus);
                if !self.condition_bits.c() {
                    self.jp(off);
                }
                self.cycles += 6;
            }
            0xD3 => {
                // OUT d8 
                undefined_instruction(0xD3, self.pc);
            }
            0xD4 => {
                // CNC adr
                let off = self.fetch_u16(bus);
                if !self.condition_bits.c() {
                    self.call(bus, off);
                    self.cycles += 6;
                }
                self.cycles += 7;
            }
            0xD5 => {
                // PUSH D
                self.push(bus, get!(self, de));
                self.cycles += 7;
            }
            0xD6 => {
                // SUI d8
                set!(self, A, sub_instruction(get!(self, A), self.fetch_u8(bus), false, &mut self.condition_bits));
                self.cycles += 3;
            }
            0xD7 => {
                // RST 2
                let off = 0x0010;
                self.call(bus, off);
                self.cycles += 7;
            }
            0xD8 => {
                // RC
                if self.condition_bits.c() {
                    self.ret(bus);
                    self.cycles += 6;
                }
                self.cycles += 1;
            }
            0xD9 => {
                // RET
                self.ret(bus);
                self.cycles += 6;
            }
            0xDA => {
                // JC adr
                let off = self.fetch_u16(bus);
                if self.condition_bits.c() {
                    self.jp(off);
                }
                self.cycles += 6;
            }
            0xDB => {
                // IN d8
                undefined_instruction(0xDB, self.pc);
                self.cycles += 3;
            }
            0xDC => {
                // CC adr
                let off = self.fetch_u16(bus);
                if self.condition_bits.c() {
                    self.call(bus, off);
                    self.cycles += 6;
                }
                self.cycles += 7;
            }
            0xDD => {
                // CALL adr (alternate)
                let off = self.fetch_u16(bus);
                self.call(bus, off);
                self.cycles += 13;
            }
            0xDE => {
                // SBI d8
                set!(self, A, sub_instruction(get!(self, A), self.fetch_u8(bus), self.condition_bits.c(), &mut self.condition_bits));
                self.cycles += 3;
            }
            0xDF => {
                // RST 3
                let off = 0x0018;
                self.call(bus, off);
                self.cycles += 7;
            }
            0xE0 => {
                // RPO
                if !self.condition_bits.p() {
                    self.ret(bus);
                    self.cycles += 6;
                }
                self.cycles += 1;
            }
            0xE1 => {
                // POP H
                let val = self.pop(bus);
                set!(self, hl, val);
                self.cycles += 6;
            }
            0xE2 => {
                // JPO adr
                let off = self.fetch_u16(bus);
                if !self.condition_bits.p() {
                    self.jp(off);
                }
                self.cycles += 6;
            }
            0xE3 => {
                // XTHL (unimplemented)
                undefined_instruction(0xE3, self.pc);
            }
            0xE4 => {
                // CPO adr
                let off = self.fetch_u16(bus);
                if !self.condition_bits.p() {
                    self.call(bus, off);
                    self.cycles += 6;
                }
                self.cycles += 7;
            }
            0xE5 => {
                // PUSH H
                self.push(bus, get!(self, hl));
                self.cycles += 7;
            }
            0xE6 => {
                // ANI d8
                set!(self, A, and_instruction(get!(self, A), self.fetch_u8(bus), &mut self.condition_bits));
                self.cycles += 3;
            }
            0xE7 => {
                // RST 4
                let off = 0x0020;
                self.call(bus, off);
                self.cycles += 7;
            }
            0xE8 => {
                // RPE
                if self.condition_bits.p() {
                    self.ret(bus);
                    self.cycles += 6;
                }
                self.cycles += 1;
            }
            0xE9 => {
                // PCHL
                self.pc = get!(self, hl);
            }
            0xEA => {
                // JPE adr
                let off = self.fetch_u16(bus);
                if self.condition_bits.p() {
                    self.jp(off);
                }
                self.cycles += 6;
            }
            0xEB => {
                // XCHG (unimplemented)
                undefined_instruction(0xEB, self.pc);
            }
            0xEC => {
                // CPE adr
                let off = self.fetch_u16(bus);
                if self.condition_bits.p() {
                    self.call(bus, off);
                    self.cycles += 6;
                }
                self.cycles += 7;
            }
            0xED => {
                // CALL adr (alternate)
                let off = self.fetch_u16(bus);
                self.call(bus, off);
                self.cycles += 13;
            }
            0xEE => {
                // XRI d8
                set!(self, A, xor_instruction(get!(self, A), self.fetch_u8(bus), &mut self.condition_bits));
                self.cycles += 3;
            }
            0xEF => {
                // RST 5
                let off = 0x0028;
                self.call(bus, off);
                self.cycles += 7;
            }
            0xF0 => {
                // RP
                if !self.condition_bits.s() {
                    self.ret(bus);
                    self.cycles += 6;
                }
                self.cycles += 1;
            }
            0xF1 => {
                // POP PSW
                let val = self.pop(bus);
                self.registers.A = (val >> 8) as u8;
                self.condition_bits.set_bits(val as u8);
                self.cycles += 6;
            }
            0xF2 => {
                // JP adr
                let off = self.fetch_u16(bus);
                if !self.condition_bits.s() {
                    self.jp(off);
                }
                self.cycles += 6;
            }
            0xF3 => {
                // DI (unimplemented)
                undefined_instruction(0xF3, self.pc);
            }
            0xF4 => {
                // CP adr
                let off = self.fetch_u16(bus);
                if !self.condition_bits.s() {
                    self.call(bus, off);
                    self.cycles += 6;
                }
                self.cycles += 7;
            }
            0xF5 => {
                // PUSH PSW
                let val = ((get!(self, A) as u16) << 8) | (self.condition_bits.bits as u16);
                self.push(bus, val);
                self.cycles += 7;
            }
            0xF6 => {
                // ORI d8
                set!(self, A, or_instruction(get!(self, A), self.fetch_u8(bus), &mut self.condition_bits));
                self.cycles += 3;
            }
            0xF7 => {
                // RST 6
                let off = 0x0030;
                self.call(bus, off);
                self.cycles += 7;
            }
            0xF8 => {
                // RM
                if self.condition_bits.s() {
                    self.ret(bus);
                    self.cycles += 6;
                }
                self.cycles += 1;
            }
            0xF9 => {
                // SPHL (unimplemented)
                undefined_instruction(0xF9, self.pc);
            }
            0xFA => {
                // JM adr
                let off = self.fetch_u16(bus);
                if self.condition_bits.s() {
                    self.jp(off);
                }
                self.cycles += 6;
            }
            0xFB => {
                // EI (unimplemented)
                undefined_instruction(0xFB, self.pc);
            }
            0xFC => {
                // CM adr
                let off = self.fetch_u16(bus);
                if self.condition_bits.s() {
                    self.call(bus, off);
                    self.cycles += 6;
                }
                self.cycles += 7;
            }
            0xFD => {
                // CALL adr (alternate)
                let off = self.fetch_u16(bus);
                self.call(bus, off);
                self.cycles += 13;
            }
            0xFE => {
                // CPI d8
                sub_instruction(get!(self, A), self.fetch_u8(bus), false, &mut self.condition_bits);
                self.cycles += 3;
            }
            0xFF => {
                // RST 7
                let off = 0x0038;
                self.call(bus, off);
                self.cycles += 7;
            }
        }
    }
}

