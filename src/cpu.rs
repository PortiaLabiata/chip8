use crate::framebuffer;

use super::ram;

pub struct Cpu {
    v: [u8; 16],
    i: u16,
    sp: u8,
    dt: u8,
    st: u8,
    pc: u16,
}

pub enum Opcode {
    Cls,
    Ret,
    Jp(u16),
    Call(u16),
    Se(u8, u8),
    Sne(u8, u8),
    Sexy(u8, u8),
    Ld(u8, u8),
    Add(u8, u8),
}

impl TryFrom<u16> for Opcode {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let a = ((value >> 12) & 0x0F) as u8;
        let b = ((value >> 8) & 0x0F) as u8;
        let c = ((value >> 4) & 0x0F) as u8;
        let d = ((value >> 0) & 0x0F) as u8;

        return match (a, b, c, d) {
            (0x0, 0x0, 0xE, 0x0) => Ok(Opcode::Cls),

            (0x0, 0x0, 0xE, 0xE) => Ok(Opcode::Ret),

            (0x1, n1, n2, n3) => {
                let addr = ((n1 as u16) << 8) | ((n2 as u16) << 4) | n3 as u16;
                Ok(Opcode::Jp(addr))
            }

            (0x2, n1, n2, n3) => {
                let addr = ((n1 as u16) << 8) | ((n2 as u16) << 4) | n3 as u16;
                Ok(Opcode::Call(addr))
            }

            (0x3, x, k1, k2) => {
                let addr = (k1 << 4) | k2;
                Ok(Opcode::Se(x, addr))
            }

            (0x4, x, k1, k2) => {
                let addr = (k1 << 4) | k2;
                Ok(Opcode::Sne(x, addr))
            }

            (0x5, x, y, 0x0) => Ok(Opcode::Sexy(x, y)),

            _ => Err(()),
        };
    }
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            v: [0; 16],
            i: 0,
            sp: 0,
            dt: 0,
            st: 0,
            pc: 0,
        }
    }

    fn cls(&self, fb: &mut framebuffer::FrameBuffer) {
        for line in 0..framebuffer::LINES {
            for col in 0..framebuffer::COLS {
                fb.reset(col as u8, line as u8);
            }
        }
    }

    fn push(&mut self, v: u16, memory: &mut ram::Ram) {
        if self.sp as usize >= ram::STACK_SIZE {
            return;
        }

        // TODO: add invalid value handling
        match memory.write_stack(self.sp, v) {
            Ok(_) => (),
            Err(_) => (),
        }
        self.sp += 1;
    }

    fn pop(&mut self, memory: &mut ram::Ram) -> Option<u16> {
        let v = memory.read_stack(self.sp);
        self.sp -= 1;
        v
    }

    fn execute_opcode(
        &mut self,
        op: Opcode,
        memory: &mut ram::Ram,
        fb: &mut framebuffer::FrameBuffer,
    ) {
        match op {
            Opcode::Cls => self.cls(fb),

            // TODO: add pop fail handling
            Opcode::Ret => {
                self.pc = match self.pop(memory) {
                    Some(v) => v,
                    None => return,
                };
            }

            Opcode::Jp(a) => self.pc = a,

            Opcode::Call(a) => {
                self.push(self.pc, memory);
                self.pc = a;
            }

            // TODO: add better invalid index handling
            Opcode::Se(r, b) => {
                if self.v[r as usize] == b {
                    self.pc += 1;
                }
            }

            Opcode::Sne(r, b) => {
                if self.v[r as usize] != b {
                    self.pc += 1;
                }
            }

            Opcode::Sexy(r1, r2) => {
                if self.v[r1 as usize] == self.v[r2 as usize] {
                    self.pc += 1;
                }
            }

            Opcode::Ld(r, b) => {
                self.v[r as usize] = b;
            }

            Opcode::Add(r, b) => {
                self.v[r as usize] += b;
            }
        }
    }

    // TODO: add better error handling
    pub fn tick(&mut self, memory: &mut ram::Ram, fb: &mut framebuffer::FrameBuffer) {
        let instruction = match memory.read16(self.pc) {
            Some(v) => v,
            None => return,
        };

        let opcode = match Opcode::try_from(instruction) {
            Ok(v) => v,
            Err(_) => return,
        };

        self.execute_opcode(opcode, memory, fb);
    }
}
