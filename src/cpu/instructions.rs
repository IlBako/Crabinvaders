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
