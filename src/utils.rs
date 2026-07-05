use std::time::{Duration, Instant};

/// Clocks and timers
pub mod clock {

    /// Internal step function
    #[inline(always)]
    fn step_internal(clock: &mut usize, mut cycles: usize, modulo: usize) -> usize {
        cycles += *clock;
        *clock = cycles % modulo;

        cycles / modulo
    }

    pub struct FixedClock<const M: usize> {
        pub cycles: usize,
    }

    impl<const M: usize> FixedClock<M> {
        pub fn new() -> Self {
            Self { cycles: 0 }
        }

        pub fn reset(&mut self) {
            self.cycles = 0;
        }

        pub fn step(&mut self, cycles: usize) -> usize {
            step_internal(&mut self.cycles, cycles, M)
        }
    }
}

pub fn real_time(f: impl FnOnce() -> usize) {
    let start = Instant::now();
    let cycles = f();

    let delta = start.elapsed();
    let real_delta = Duration::from_micros((cycles * 1_000_000 / crate::CYCLES_SECOND) as _);

    // If we executed less than the real delta, wait
    if delta < real_delta {
        std::thread::sleep(real_delta - delta);
    }
}
