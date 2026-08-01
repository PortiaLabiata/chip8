use minifb::{Window, WindowOptions};

pub const LINES: usize = 64;
pub const COLS: usize = 32;

pub struct FrameBuffer {
    buffer: [u32; LINES * COLS],
    window: Window,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Result<Self, minifb::Error> {
        let buffer = [0; LINES * COLS];
        let window = Window::new("CHIP-8", width, height, WindowOptions::default())?;

        Ok(FrameBuffer { buffer, window })
    }

    pub fn run(&mut self) -> bool {
        if !self.window.is_open() {
            return false;
        }

        return match self.window.update_with_buffer(&self.buffer, 64, 32) {
            Ok(_) => true,
            Err(_) => false,
        };
    }

    pub fn reset(&mut self, x: u8, y: u8) {
        if x as usize > COLS || y as usize > LINES {
            return;
        }

        self.buffer[x as usize * y as usize] = 0x00000000;
    }

    pub fn blit(&mut self, sprite: &[u8], x: u8, y: u8) -> bool {
        let mut collision = false;

        for (row, &byte) in sprite.iter().enumerate() {
            for bit in 0..8 {
                // CHIP-8: старший бит (7) — левый пиксель строки
                if (byte >> (7 - bit)) & 0x1 == 0x1 {
                    let px = (x as usize + bit) % COLS;
                    let py = (y as usize + row) % LINES;
                    let idx = py * COLS + px;

                    let old = self.buffer[idx];
                    let new = old ^ u32::MAX;
                    self.buffer[idx] = new;

                    // Коллизия: пиксель был включён (u32::MAX) и стал выключенным (0)
                    if old == u32::MAX && new == 0 {
                        collision = true;
                    }
                }
            }
        }

        collision
    }
}
