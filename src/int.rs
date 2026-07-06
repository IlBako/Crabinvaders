#[derive(Debug, Clone)]
pub struct Int {
    if_: u8,
}

impl Int {
    pub const RESET: u8 = 0;
    pub const HALF_SCREEN: u8 = 1;
    pub const VBLANK: u8 = 2;

    pub fn new() -> Self {
        Self { if_: 0 }
    }

    #[inline(always)]
    fn mask(i: u8) -> u8 {
        (1 << i) & 0x07
    }

    #[inline(always)]
    fn reset_int(&mut self, i: u8) {
        self.if_ &= !Self::mask(i);
    }

    /// Are there any pending interrupts
    #[inline(always)]
    pub fn has_pending(&mut self) -> bool {
        (self.if_ & 0x07) != 0
    }

    /// Set a given interrupt
    #[inline(always)]
    pub fn set_int(&mut self, i: u8) {
        self.if_ |= Self::mask(i);
    }

    /// Take an interrupt, resetting it
    #[inline(always)]
    pub fn take_int(&mut self, i: u8) -> bool {
        let res = (self.if_ & Self::mask(i)) != 0;
        self.reset_int(i);
        res
    }
}
