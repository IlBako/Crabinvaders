use crate::{clock::FixedClock, int::Int};

const VRAM_SIZE: usize = 7168;
pub const WIDTH: usize = 224;
pub const HEIGHT: usize = 256;
const PIX_BUF_LEN: usize = WIDTH * HEIGHT * 3;

pub struct Video {
    clock: FixedClock<128>,
    vram: Box<[u8; VRAM_SIZE]>,
    pub pixel_buffer: Box<[u8; PIX_BUF_LEN]>,
    pub img_ready: bool,
    scanline: u16,
}

impl Video {
    pub fn new() -> Self {
        Self {
            clock: FixedClock::new(),
            vram: Box::new([0; VRAM_SIZE]),
            pixel_buffer: Box::new([0; PIX_BUF_LEN]),
            img_ready: false,
            scanline: 0,
        }
    }

    pub fn step(&mut self, int: &mut Int, cycles: usize) {
        self.img_ready = false;
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
        for i in 0..VRAM_SIZE {
            let byte = self.vram[i];

            // Calculate unrotated x and y
            let unrotated_y = i / 32;
            let unrotated_x_base = (i % 32) * 8;

            // Each bit corresponds to 1 pixel
            for bit in 0..8 {
                let pixel = (byte >> bit) & 1 == 1;
                let unrotated_x = unrotated_x_base + bit;

                // Apply 90-degree counter-clockwise rotation
                let rotated_x = unrotated_y;
                let rotated_y = 255 - unrotated_x;

                // Calculate index in 1D RGB buffer
                let buffer_index = (rotated_y * WIDTH + rotated_x) * 3;

                // Determine color (white for on, black for off)
                let color = if pixel { 255 } else { 0 };

                // Write RGB values
                self.pixel_buffer[buffer_index] = color; // R
                self.pixel_buffer[buffer_index + 1] = color; // G
                self.pixel_buffer[buffer_index + 2] = color; // B
            }
        }
        self.img_ready = true;
    }

    /// Read from VRAM 2400 - 3FFF
    pub fn read_vram(&self, addr: u16) -> u8 {
        self.vram[addr as usize]
    }

    /// Write to VRAM 2400 - 3FFF
    pub fn write_vram(&mut self, addr: u16, val: u8) {
        self.vram[addr as usize] = val;
    }
}
