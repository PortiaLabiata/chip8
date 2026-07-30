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

    pub fn set(&mut self, x: u8, y: u8) {
        if (x as usize > COLS || y as usize > LINES) {
            return;
        }

        self.buffer[(x * y) as usize] = 0xFFFFFFFF;
    }

    pub fn reset(&mut self, x: u8, y: u8) {
        if (x as usize > COLS || y as usize > LINES) {
            return;
        }

        self.buffer[(x * y) as usize] = 0x00000000;
    }
}
