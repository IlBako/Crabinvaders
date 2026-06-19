use std::fmt::{self, Binary};

#[allow(non_snake_case)]
#[derive(Default)]
pub struct Registers {
    pub A: u8,
    pub B: u8,
    pub C: u8,
    pub D: u8,
    pub E: u8,
    pub H: u8,
    pub L: u8,
}

impl Registers {
    pub fn get_bc(&self) -> u16 {
        ((self.B as u16) << 8) | self.C as u16
    }
    pub fn get_de(&self) -> u16 {
        ((self.D as u16) << 8) | self.E as u16
    }
    pub fn get_hl(&self) -> u16 {
        ((self.H as u16) << 8) | self.L as u16
    }

    pub fn set_bc(&mut self, val: u16) {
        self.B = (val >> 8) as u8;
        self.C = val as u8;
    }
    pub fn set_de(&mut self, val: u16) {
        self.D = (val >> 8) as u8;
        self.E = val as u8;
    }
    pub fn set_hl(&mut self, val: u16) {
        self.H = (val >> 8) as u8;
        self.L = val as u8;
    }
}

pub struct ConditionBits {
    pub bits: u8,
}

impl ConditionBits {
    /// Condition Bits saved as a single u8
    /// Bit 7: Sign (s)
    /// Bit 6: Zero (z)
    /// Bit 5: 0
    /// Bit 4: Auxiliary carry (ac)
    /// Bit 3: 0
    /// Bit 2: Parity (p)
    /// Bit 1: 1
    /// Bit 0: Carry (c)

    // Getters
    pub(crate) fn c(&self) -> bool {
        self.bits & 0x01 != 0
    }
    pub(crate) fn p(&self) -> bool {
        self.bits & (0x01 << 2) != 0
    }
    pub(crate) fn ac(&self) -> bool {
        self.bits & (0x01 << 4) != 0
    }
    pub(crate) fn z(&self) -> bool {
        self.bits & (0x01 << 6) != 0
    }
    pub(crate) fn s(&self) -> bool {
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

    pub(crate) fn set_bits(&mut self, value: u8) {
        // Allows to set all the bits together while guaranteeing that bit 3 and 5 remain 0 and bit 1 remains 1
        self.bits = (value & 0b1101_0111) | 0b0000_0010;
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
