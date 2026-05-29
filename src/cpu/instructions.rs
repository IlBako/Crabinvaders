use crate::cpu::ConditionBits;

pub fn undefined_instruction(instruction: u8, pc: u16) {
    panic!(
        "Undefined instruction {:#04x} at {:#06x}",
        instruction,
        pc - 1
    )
}

pub fn inr_instruction(current_val: u8, condition_bits: &mut ConditionBits) -> u8 {
    let res = current_val.wrapping_add(1);
    condition_bits.set_z(res == 0);
    condition_bits.set_p(res.count_ones() % 2 == 0);
    condition_bits.set_s(res & 0x80 == 0x80);
    condition_bits.set_ac((current_val & 0x0F) == 0x0F);
    res
}

pub fn dcr_instruction(current_val: u8, condition_bits: &mut ConditionBits) -> u8 {
    let res = current_val.wrapping_sub(1);
    condition_bits.set_z(res == 0);
    condition_bits.set_p(res.count_ones() % 2 == 0);
    condition_bits.set_s(res & 0x80 == 0x80);
    condition_bits.set_ac((current_val & 0x0F) == 0x00);
    res
}

pub fn add_instruction(
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

pub fn sub_instruction(
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

pub fn and_instruction(current_val: u8, value: u8, condition_bits: &mut ConditionBits) -> u8 {
    let res = current_val & value;
    condition_bits.set_z(res == 0);
    condition_bits.set_c(false);
    condition_bits.set_ac(false);
    condition_bits.set_p(res.count_ones() % 2 == 0);
    condition_bits.set_s(res & 0x80 == 0x80);
    res
}

pub fn xor_instruction(current_val: u8, value: u8, condition_bits: &mut ConditionBits) -> u8 {
    let res = current_val ^ value;
    condition_bits.set_z(res == 0);
    condition_bits.set_c(false);
    condition_bits.set_ac(false);
    condition_bits.set_p(res.count_ones() % 2 == 0);
    condition_bits.set_s(res & 0x80 == 0x80);
    res
}

pub fn or_instruction(current_val: u8, value: u8, condition_bits: &mut ConditionBits) -> u8 {
    let res = current_val | value;
    condition_bits.set_z(res == 0);
    condition_bits.set_c(false);
    condition_bits.set_ac(false);
    condition_bits.set_p(res.count_ones() % 2 == 0);
    condition_bits.set_s(res & 0x80 == 0x80);
    res
}

pub fn ret_instruction(pc: &mut u16, sp: &mut u16, memory: &mut [u8]) {
    let lower_nibble = memory[*sp as usize] as u16;
    *sp = sp.wrapping_add(1);
    let upper_nibble = memory[*sp as usize] as u16;
    *sp = sp.wrapping_add(1);
    *pc = (upper_nibble << 8) | lower_nibble;
}

pub fn call_instruction(
    pc: &mut u16,
    sp: &mut u16,
    memory: &mut [u8],
    address: u16,
    return_address: u16,
) {
    let lower_nibble = (return_address & 0x00FF) as u8;
    let upper_nibble = (return_address >> 8) as u8;
    *sp = sp.wrapping_sub(1);
    memory[*sp as usize] = upper_nibble;
    *sp = sp.wrapping_sub(1);
    memory[*sp as usize] = lower_nibble;
    *pc = address;
}
