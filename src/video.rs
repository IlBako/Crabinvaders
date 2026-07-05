use crate::{clock::FixedClock, int::Int};

pub struct Video {
    clock: FixedClock<128>,
    scanline: u16,
}

impl Video {
    pub fn new() -> Self {
        Self {
            clock: FixedClock::new(),
            scanline: 0,
        }
    }

    pub fn step(&mut self, int: &mut Int, cycles: usize) {
        // Step the internal clock
        for _ in 0..self.clock.step(cycles) {
            self.scanline += 1;
            self.scanline %= 262;

            if self.scanline == 96 {
                // Trigger half-screen interrupt
                int.set_int(Int::HALF_SCREEN);
            } else if self.scanline == 224 {
                // Trigger VBlank interrupt
                int.set_int(Int::VBLANK);
                self.render();
            }
        }
    }

    fn render(&mut self) {
        unimplemented!()
    }
}
