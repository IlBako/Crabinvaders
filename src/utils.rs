use std::time::{Duration, Instant};

pub fn real_time(f: impl FnOnce() -> usize) {
    let start = Instant::now();
    let cycles = f();

    let delta = start.elapsed();
    let real_delta = Duration::from_micros( (cycles * 1_000_000 / crate::CYCLES_SECOND) as _ );
    
    // If we executed less than the real delta, wait
    if delta < real_delta {
        std::thread::sleep(real_delta - delta);
    }
}
