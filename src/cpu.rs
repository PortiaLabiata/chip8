use std::num::Wrapping;

use super::framebuffer;
use super::ram;

#[derive(Debug)]
pub struct Cpu {
    v: [Wrapping<u8>; 16],
    i: Wrapping<u16>,
    sp: Wrapping<u8>,
    dt: Wrapping<u8>,
    st: Wrapping<u8>,
    pc: Wrapping<u16>,
}

#[derive(Debug)]
pub enum Opcode {
    Cls,

    Ret,
    Jp(u16),
    Call(u16),

    Se(u8, u8),
    Sne(u8, u8),
    Sexy(u8, u8),

    Ld(u8, u8),
    Ldxy(u8, u8),

    Add(u8, u8),
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    Addxy(u8, u8),
    Subxy(u8, u8),
    Shr(u8, u8),
    Subn(u8, u8),
    Shl(u8, u8),

    Snexy(u8, u8),
    Ldi(u16),
    Jpv0(u16),
    Rnd(u8, u8),

    Drw(u8, u8, u8),
    Skp(u8),
    Sknp(u8),

    Ldxd(u8),
    Ldxk(u8),
    Lddx(u8),
    Ldsx(u8),

    Addix(u8),
    Ldfx(u8),
    Ldbx(u8),
    Ldix(u8),
    Ldxi(u8),
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

            (0x6, x, k1, k2) => Ok(Opcode::Ld(x, k1 << 4 | k2)),

            (0x7, x, k1, k2) => Ok(Opcode::Add(x, k1 << 4 | k2)),

            (0x8, x, y, 0x0) => Ok(Opcode::Ldxy(x, y)),

            (0x8, x, y, 0x1) => Ok(Opcode::Or(x, y)),

            (0x8, x, y, 0x2) => Ok(Opcode::And(x, y)),

            (0x8, x, y, 0x3) => Ok(Opcode::Xor(x, y)),

            (0x8, x, y, 0x4) => Ok(Opcode::Addxy(x, y)),

            (0x8, x, y, 0x5) => Ok(Opcode::Subxy(x, y)),

            (0x8, x, y, 0x6) => Ok(Opcode::Shr(x, y)),

            (0x8, x, y, 0x7) => Ok(Opcode::Subn(x, y)),

            (0x8, x, y, 0xE) => Ok(Opcode::Shl(x, y)),

            (0x9, x, y, 0) => Ok(Opcode::Snexy(x, y)),

            (0xa, n1, n2, n3) => {
                let addr = ((n1 as u16) << 8) | ((n2 as u16) << 4) | n3 as u16;
                Ok(Opcode::Ldi(addr))
            }

            (0xb, n1, n2, n3) => {
                let addr = ((n1 as u16) << 8) | ((n2 as u16) << 4) | n3 as u16;
                Ok(Opcode::Jpv0(addr))
            }

            (0xc, x, k1, k2) => Ok(Opcode::Rnd(x, k1 << 4 | k2)),

            (0xd, x, y, n) => Ok(Opcode::Drw(x, y, n)),

            (0xe, x, 0x9, 0xe) => Ok(Opcode::Skp(x)),

            (0xe, x, 0xa, 0x1) => Ok(Opcode::Sknp(x)),

            (0xf, x, 0x0, 0x7) => Ok(Opcode::Ldxd(x)),

            (0xf, x, 0x0, 0xa) => Ok(Opcode::Ldxk(x)),

            (0xf, x, 0x1, 0x5) => Ok(Opcode::Lddx(x)),

            (0xf, x, 0x1, 0x8) => Ok(Opcode::Ldsx(x)),

            (0xf, x, 0x1, 0xe) => Ok(Opcode::Addix(x)),

            (0xf, x, 0x2, 0x9) => Ok(Opcode::Ldfx(x)),

            (0xf, x, 0x3, 0x3) => Ok(Opcode::Ldbx(x)),

            (0xf, x, 0x5, 0x5) => Ok(Opcode::Ldix(x)),

            (0xf, x, 0x6, 0x5) => Ok(Opcode::Ldxi(x)),

            _ => Err(()),
        };
    }
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            v: [Wrapping(0); 16],
            i: Wrapping(0),
            sp: Wrapping(0),
            dt: Wrapping(0),
            st: Wrapping(0),
            pc: Wrapping(0x200),
        }
    }

    pub fn pc(&self) -> u16 {
        self.pc.0
    }

    fn cls(&self, fb: &mut framebuffer::FrameBuffer) {
        for line in 0..framebuffer::LINES {
            for col in 0..framebuffer::COLS {
                fb.reset(col as u8, line as u8);
            }
        }
    }

    fn push(&mut self, v: u16, memory: &mut ram::Ram) {
        if self.sp.0 as usize >= ram::STACK_SIZE / 2 {
            return;
        }

        // TODO: add invalid value handling
        match memory.write_stack(self.sp.0, v) {
            Ok(_) => (),
            Err(_) => (),
        }
        self.sp += 1;
    }

    fn pop(&mut self, memory: &mut ram::Ram) -> Option<u16> {
        self.sp -= 1;
        let v = memory.read_stack(self.sp.0);
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
                self.pc = Wrapping(match self.pop(memory) {
                    Some(v) => v,
                    None => return,
                });
            }

            Opcode::Jp(a) => self.pc = Wrapping(a),

            Opcode::Call(a) => {
                self.push(self.pc.0, memory);
                self.pc = Wrapping(a);
            }

            // TODO: add better invalid index handling
            Opcode::Se(r, b) => {
                if self.v[r as usize].0 == b {
                    self.pc += 2;
                }
            }

            Opcode::Sne(r, b) => {
                if self.v[r as usize].0 != b {
                    self.pc += 2;
                }
            }

            Opcode::Sexy(r1, r2) => {
                if self.v[r1 as usize] == self.v[r2 as usize] {
                    self.pc += 2;
                }
            }

            Opcode::Ld(r, b) => {
                self.v[r as usize] = Wrapping(b);
            }

            Opcode::Add(r, b) => {
                let v = self.v[r as usize];
                self.v[r as usize] += Wrapping(b);
                if v.0 as u16 + b as u16 > 255 {
                    self.v[15] = Wrapping(1);
                } else {
                    self.v[15] = Wrapping(0);
                }
            }

            Opcode::Ldxy(x, y) => {
                self.v[x as usize] = self.v[y as usize];
            }

            Opcode::Or(x, y) => {
                self.v[x as usize] = self.v[x as usize] | self.v[y as usize];
            }

            Opcode::And(x, y) => {
                self.v[x as usize] = self.v[x as usize] & self.v[y as usize];
            }

            Opcode::Xor(x, y) => {
                self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize];
            }

            Opcode::Addxy(x, y) => {
                let (vx, vy) = (self.v[x as usize], self.v[y as usize]);
                self.v[x as usize] = self.v[x as usize] + self.v[y as usize];
                if (vx.0 as u16) + (vy.0 as u16) > 255 {
                    self.v[15] = Wrapping(1);
                } else {
                    self.v[15] = Wrapping(0);
                }
            }

            Opcode::Subxy(x, y) => {
                let (vx, vy) = (self.v[x as usize], self.v[y as usize]);
                self.v[x as usize] -= self.v[y as usize];
                if (vx.0 as i16) - (vy.0 as i16) < 0 {
                    self.v[15] = Wrapping(0);
                } else {
                    self.v[15] = Wrapping(1);
                }
            }

            Opcode::Shr(x, _) => {
                let v = self.v[x as usize];
                let lsb = v.0 & 0x01;
                self.v[x as usize] = v >> 1;
                if lsb == 0x01 {
                    self.v[15] = Wrapping(1);
                } else {
                    self.v[15] = Wrapping(0);
                }
            }

            Opcode::Subn(x, y) => {
                let (vx, vy) = (self.v[x as usize], self.v[y as usize]);
                self.v[x as usize] = self.v[y as usize] - self.v[x as usize];
                if (vy.0 as i16) - (vx.0 as i16) < 0 {
                    self.v[15] = Wrapping(0);
                } else {
                    self.v[15] = Wrapping(1);
                }
            }

            Opcode::Shl(x, _) => {
                let v = self.v[x as usize];
                let msb = (v.0 & 0b10000000) >> 7;
                self.v[x as usize] = v << 1;
                if msb == 0x01 {
                    self.v[15] = Wrapping(1);
                } else {
                    self.v[15] = Wrapping(0);
                }
            }

            Opcode::Snexy(x, y) => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
            }

            Opcode::Ldi(a) => {
                self.i = Wrapping(a);
            }

            Opcode::Jpv0(a) => {
                self.pc = Wrapping::<u16>(self.v[0].0 as u16) + Wrapping(a);
            }

            Opcode::Rnd(x, k) => {
                let v: u8 = rand::random();
                self.v[x as usize] = Wrapping(v & k);
            }

            Opcode::Drw(x, y, n) => {
                let mut sprite = Vec::with_capacity(n as usize);
                for a in self.i.0..(self.i.0 + n as u16) {
                    sprite.push(memory.read(a).unwrap());
                }
                if fb.blit(&sprite, self.v[x as usize].0, self.v[y as usize].0) {
                    self.v[15] = Wrapping(1);
                } else {
                    self.v[15] = Wrapping(0);
                }
            }

            Opcode::Skp(x) => {
                if fb.key_pressed(self.v[x as usize].0) {
                    self.pc += 2;
                }
            }

            Opcode::Sknp(x) => {
                if !fb.key_pressed(self.v[x as usize].0) {
                    self.pc += 2;
                }
            }

            Opcode::Ldxd(x) => {
                self.v[x as usize] = self.dt;
            }

            Opcode::Ldxk(x) => {
                while !fb.key_pressed(x) {
                    fb.run();
                }
            }

            Opcode::Lddx(x) => {
                self.dt = self.v[x as usize];
            }

            Opcode::Ldsx(x) => {
                self.st = self.v[x as usize];
            }

            Opcode::Addix(x) => {
                self.i += Wrapping(self.v[x as usize].0 as u16);
            }

            Opcode::Ldfx(x) => {
                self.i = Wrapping((self.v[x as usize].0 * 5) as u16);
            }

            Opcode::Ldbx(x) => {
                let v = self.v[x as usize].0;
                let (v1, v2, v3) = (v / 100, (v / 10) % 10, v % 10);
                memory.write(self.i.0, v1).unwrap();
                memory.write((self.i + Wrapping(1)).0, v2).unwrap();
                memory.write((self.i + Wrapping(2)).0, v3).unwrap();
            }

            Opcode::Ldix(x) => {
                for i in 0..(x as usize + 1) {
                    memory.write((self.i + Wrapping(i as u16)).0, self.v[i].0).unwrap();
                }
                self.i += (x + 1) as u16;
            }

            // TODO: add better error handling
            Opcode::Ldxi(x) => {
                for i in 0..(x as usize + 1) {
                    let v = memory.read((self.i + Wrapping(i as u16)).0).unwrap();
                    self.v[i as usize] = Wrapping(v);
                }
                self.i += (x + 1) as u16;
            }
        }
    }

    // TODO: add better error handling
    pub fn tick(&mut self, memory: &mut ram::Ram, fb: &mut framebuffer::FrameBuffer) {
        let instruction = match memory.read16(self.pc.0) {
            Some(v) => v,
            None => panic!("Failed to fetch instruction"),
        };

        let opcode = match Opcode::try_from(instruction) {
            Ok(v) => v,
            Err(_) => return,
        };

        println!("pc: {} op: {:?}", self.pc, opcode);
        println!("{:?}", self);
        self.pc += 2;
        if self.dt.0 > 0 {
            self.dt -= 1;
        }

        if self.st.0 > 0 {
            self.st -= 1;
        }

        self.execute_opcode(opcode, memory, fb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer;

    // Хелпер для создания объектов. Замените `FrameBuffer::new(900, 900).unwrap()` на ваш конструктор, если нужно.
    fn setup() -> (Cpu, ram::Ram, framebuffer::FrameBuffer) {
        (
            Cpu::new(),
            ram::Ram::new(),
            framebuffer::FrameBuffer::new(900, 900).unwrap(),
        )
    }

    // ============================================================
    // Декодирование опкодов (TryFrom<u16>)
    // ============================================================
    mod opcode_decode {
        use super::Opcode;

        #[test]
        fn cls() {
            assert!(matches!(Opcode::try_from(0x00E0).unwrap(), Opcode::Cls));
        }

        #[test]
        fn ret() {
            assert!(matches!(Opcode::try_from(0x00EE).unwrap(), Opcode::Ret));
        }

        #[test]
        fn jp() {
            assert!(matches!(
                Opcode::try_from(0x1234).unwrap(),
                Opcode::Jp(0x234)
            ));
            assert!(matches!(
                Opcode::try_from(0x1FFF).unwrap(),
                Opcode::Jp(0xFFF)
            ));
        }

        #[test]
        fn call() {
            assert!(matches!(
                Opcode::try_from(0x2345).unwrap(),
                Opcode::Call(0x345)
            ));
            assert!(matches!(
                Opcode::try_from(0x2000).unwrap(),
                Opcode::Call(0x000)
            ));
        }

        #[test]
        fn se() {
            assert!(matches!(
                Opcode::try_from(0x30AB).unwrap(),
                Opcode::Se(0x0, 0xAB)
            ));
            assert!(matches!(
                Opcode::try_from(0x3FAB).unwrap(),
                Opcode::Se(0xF, 0xAB)
            ));
        }

        #[test]
        fn sne() {
            assert!(matches!(
                Opcode::try_from(0x40CD).unwrap(),
                Opcode::Sne(0x0, 0xCD)
            ));
            assert!(matches!(
                Opcode::try_from(0x4F12).unwrap(),
                Opcode::Sne(0xF, 0x12)
            ));
        }

        #[test]
        fn sexy() {
            assert!(matches!(
                Opcode::try_from(0x5120).unwrap(),
                Opcode::Sexy(0x1, 0x2)
            ));
            assert!(matches!(
                Opcode::try_from(0x5FE0).unwrap(),
                Opcode::Sexy(0xF, 0xE)
            ));
        }

        #[test]
        fn ld() {
            assert!(Opcode::try_from(0x6012).is_ok());
        }

        #[test]
        fn add() {
            assert!(Opcode::try_from(0x7012).is_ok());
        }

        #[test]
        fn ldxy() {
            assert!(matches!(Opcode::try_from(0x8010), Ok(Opcode::Ldxy(0, 1))));
        }

        #[test]
        fn or() {
            assert!(matches!(Opcode::try_from(0x8011), Ok(Opcode::Or(0, 1))));
        }

        #[test]
        fn and() {
            assert!(matches!(Opcode::try_from(0x8012), Ok(Opcode::And(0, 1))));
        }

        #[test]
        fn xor() {
            assert!(matches!(Opcode::try_from(0x8013), Ok(Opcode::Xor(0, 1))));
        }

        #[test]
        fn addxy() {
            assert!(matches!(Opcode::try_from(0x8014), Ok(Opcode::Addxy(0, 1))));
        }

        #[test]
        fn subxy() {
            assert!(matches!(Opcode::try_from(0x8015), Ok(Opcode::Subxy(0, 1))));
        }

        #[test]
        fn shr() {
            assert!(matches!(Opcode::try_from(0x8016), Ok(Opcode::Shr(0, 1))));
        }

        #[test]
        fn subn() {
            assert!(matches!(Opcode::try_from(0x8017), Ok(Opcode::Subn(0, 1))));
        }

        #[test]
        fn shl() {
            assert!(matches!(Opcode::try_from(0x801e), Ok(Opcode::Shl(0, 1))));
        }

        #[test]
        fn snexy() {
            assert!(matches!(Opcode::try_from(0x9010), Ok(Opcode::Snexy(0, 1))));
        }

        #[test]
        fn ldi() {
            assert!(matches!(Opcode::try_from(0xa123), Ok(Opcode::Ldi(0x123))));
        }

        #[test]
        fn jpv0() {
            assert!(matches!(Opcode::try_from(0xb123), Ok(Opcode::Jpv0(0x123))));
        }

        #[test]
        fn rand() {
            assert!(matches!(
                Opcode::try_from(0xc012),
                Ok(Opcode::Rnd(0x0, 0x12))
            ));
        }

        #[test]
        fn drw() {
            assert!(matches!(
                Opcode::try_from(0xd013),
                Ok(Opcode::Drw(0x0, 0x1, 0x3))
            ));
        }

        #[test]
        fn skp() {
            assert!(matches!(Opcode::try_from(0xe09e), Ok(Opcode::Skp(0x0))));
        }

        #[test]
        fn sknp() {
            assert!(matches!(Opcode::try_from(0xe0a1), Ok(Opcode::Sknp(0x0))));
        }

        #[test]
        fn ldxd() {
            assert!(matches!(Opcode::try_from(0xf107), Ok(Opcode::Ldxd(0x1))));
        }

        #[test]
        fn ldxk() {
            assert!(matches!(Opcode::try_from(0xf10a), Ok(Opcode::Ldxk(0x1))));
        }

        #[test]
        fn lddx() {
            assert!(matches!(Opcode::try_from(0xf115), Ok(Opcode::Lddx(0x1))));
        }

        #[test]
        fn ldsx() {
            assert!(matches!(Opcode::try_from(0xf118), Ok(Opcode::Ldsx(0x1))));
        }

        #[test]
        fn unknown_opcodes_return_err() {
            assert!(Opcode::try_from(0xFFFF).is_err());
            assert!(Opcode::try_from(0x0000).is_err());
            assert!(Opcode::try_from(0x9999).is_err());
        }
    }

    // ============================================================
    // Инициализация CPU
    // ============================================================
    mod cpu_init {
        use super::Cpu;
        use std::num::Wrapping;

        #[test]
        fn new_clears_all_registers() {
            let cpu = Cpu::new();
            assert_eq!(cpu.v, [Wrapping(0); 16]);
            assert_eq!(cpu.i, 0);
            assert_eq!(cpu.sp, Wrapping(0));
            assert_eq!(cpu.dt, 0);
            assert_eq!(cpu.st, 0);
            assert_eq!(cpu.pc, Wrapping(0));
        }
    }

    // ============================================================
    // Стек (push / pop)
    // ============================================================
    mod stack_ops {
        use super::{Cpu, ram};
        use std::num::Wrapping;

        #[test]
        fn push_increments_sp() {
            let mut cpu = Cpu::new();
            let mut ram = ram::Ram::new();
            cpu.push(0x1234, &mut ram);
            assert_eq!(cpu.sp.0, 1);
        }

        #[test]
        fn push_writes_to_ram_stack() {
            let mut cpu = Cpu::new();
            let mut ram = ram::Ram::new();
            cpu.push(0xABCD, &mut ram);
            assert_eq!(ram.read_stack(0), Some(0xABCD));
        }

        #[test]
        fn push_pop_roundtrip() {
            let mut cpu = Cpu::new();
            let mut ram = ram::Ram::new();
            cpu.push(0xDEAD, &mut ram);
            let val = cpu.pop(&mut ram);
            assert_eq!(val, Some(0xDEAD));
        }

        #[test]
        fn push_does_not_panic_when_full() {
            let mut cpu = Cpu::new();
            let mut ram = ram::Ram::new();
            let max = (ram::STACK_SIZE / 2) as u8;
            for _ in 0..max {
                cpu.push(0x1111, &mut ram);
            }
            // Стек полон, следующий push должен просто вернуться
            cpu.push(0x2222, &mut ram);
            assert_eq!(cpu.sp.0, max);
        }

        #[test]
        fn pop_underflow_does_not_panic() {
            let mut cpu = Cpu::new();
            let mut ram = ram::Ram::new();
            cpu.sp = Wrapping(0);
            let _ = cpu.pop(&mut ram);
            assert_eq!(cpu.sp.0, 255);
        }
    }

    // ============================================================
    // Выполнение опкодов (execute_opcode)
    // ============================================================
    mod opcode_exec {
        use super::{Opcode, setup};
        use std::num::Wrapping;

        #[test]
        fn cls_does_not_panic() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.execute_opcode(Opcode::Cls, &mut ram, &mut fb);
            // Детальная проверка требует API framebuffer; здесь проверяем отсутствие паники
        }

        #[test]
        fn jp_sets_pc() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.execute_opcode(Opcode::Jp(0x234), &mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x234);
        }

        #[test]
        fn call_pushes_pc_and_jumps() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.pc = Wrapping(0x200);
            cpu.execute_opcode(Opcode::Call(0x400), &mut ram, &mut fb);
            // push записал текущий pc в stack[0], sp стал 1
            assert_eq!(ram.read_stack(0), Some(0x200));
            assert_eq!(cpu.sp.0, 1);
            assert_eq!(cpu.pc.0, 0x400);
        }

        #[test]
        fn ret_pops_pc() {
            let (mut cpu, mut ram, mut fb) = setup();
            ram.write_stack(0, 0x200).unwrap();
            cpu.sp = Wrapping(1);
            cpu.execute_opcode(Opcode::Ret, &mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x200);
            assert_eq!(cpu.sp.0, 0);
        }

        #[test]
        fn call_followed_by_ret() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.pc = Wrapping(0x200);
            cpu.execute_opcode(Opcode::Call(0x400), &mut ram, &mut fb);
            cpu.execute_opcode(Opcode::Ret, &mut ram, &mut fb);
            assert_eq!(cpu.sp.0, 0);
            assert_eq!(cpu.pc.0, 0x200);
        }

        #[test]
        fn se_skips_when_equal() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[1] = Wrapping(0x42);
            cpu.pc = Wrapping(0x200);
            cpu.execute_opcode(Opcode::Se(1, 0x42), &mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x202);
        }

        #[test]
        fn se_no_skip_when_not_equal() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[1] = Wrapping(0x42);
            cpu.pc = Wrapping(0x200);
            cpu.execute_opcode(Opcode::Se(1, 0x43), &mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x200);
        }

        #[test]
        fn sne_skips_when_not_equal() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[2] = Wrapping(0x10);
            cpu.pc = Wrapping(0x200);
            cpu.execute_opcode(Opcode::Sne(2, 0x20), &mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x202);
        }

        #[test]
        fn sne_no_skip_when_equal() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[2] = Wrapping(0x10);
            cpu.pc = Wrapping(0x200);
            cpu.execute_opcode(Opcode::Sne(2, 0x10), &mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x200);
        }

        #[test]
        fn sexy_skips_when_equal() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[3] = Wrapping(0x55);
            cpu.v[4] = Wrapping(0x55);
            cpu.pc = Wrapping(0x200);
            cpu.execute_opcode(Opcode::Sexy(3, 4), &mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x202);
        }

        #[test]
        fn sexy_no_skip_when_not_equal() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[3] = Wrapping(0x55);
            cpu.v[4] = Wrapping(0x66);
            cpu.pc = Wrapping(0x200);
            cpu.execute_opcode(Opcode::Sexy(3, 4), &mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x200);
        }

        #[test]
        fn ld_sets_register() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.execute_opcode(Opcode::Ld(5, 0xAB), &mut ram, &mut fb);
            assert_eq!(cpu.v[5].0, 0xAB);
        }

        #[test]
        fn add_adds_to_register() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[6] = Wrapping(0x10);
            cpu.execute_opcode(Opcode::Add(6, 0x20), &mut ram, &mut fb);
            assert_eq!(cpu.v[6].0, 0x30);
        }

        #[test]
        fn add_wraps_on_overflow() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[7] = Wrapping(0xFF);
            cpu.execute_opcode(Opcode::Add(7, 0x01), &mut ram, &mut fb);
            assert_eq!(cpu.v[7].0, 0x00);
        }

        #[test]
        fn ldxy_loads() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0xDE);
            cpu.execute_opcode(Opcode::Ldxy(1, 0), &mut ram, &mut fb);
            assert_eq!(cpu.v[1].0, 0xDE);
            assert_eq!(cpu.v[0].0, 0xDE);
        }

        #[test]
        fn or_works() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0x01);
            cpu.v[1] = Wrapping(0x10);
            cpu.execute_opcode(Opcode::Or(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0x11);
            assert_eq!(cpu.v[1].0, 0x10);
        }

        #[test]
        fn and_works() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0x01);
            cpu.v[1] = Wrapping(0x11);
            cpu.execute_opcode(Opcode::And(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0x01);
            assert_eq!(cpu.v[1].0, 0x11);
        }

        #[test]
        fn xor_works() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0x01);
            cpu.v[1] = Wrapping(0x11);
            cpu.execute_opcode(Opcode::Xor(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0x10);
            assert_eq!(cpu.v[1].0, 0x11);
        }

        #[test]
        fn addxy_works() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0x02);
            cpu.v[1] = Wrapping(0x03);
            cpu.v[15] = Wrapping(0xDE);
            cpu.execute_opcode(Opcode::Addxy(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0x05);
            assert_eq!(cpu.v[1].0, 0x03);
            assert_eq!(cpu.v[15].0, 0);
        }

        #[test]
        fn addxy_vf() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0xff);
            cpu.v[1] = Wrapping(0x03);
            cpu.v[15] = Wrapping(0xDE);
            cpu.execute_opcode(Opcode::Addxy(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0x02);
            assert_eq!(cpu.v[1].0, 0x03);
            assert_eq!(cpu.v[15].0, 1);
        }

        #[test]
        fn subxy_works() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0x03);
            cpu.v[1] = Wrapping(0x02);
            cpu.v[15] = Wrapping(0xDE);
            cpu.execute_opcode(Opcode::Subxy(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0x01);
            assert_eq!(cpu.v[1].0, 0x02);
            assert_eq!(cpu.v[15].0, 0);
        }

        #[test]
        fn subxy_vf() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0x00);
            cpu.v[1] = Wrapping(0x03);
            cpu.v[15] = Wrapping(0xDE);
            cpu.execute_opcode(Opcode::Subxy(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0xfd);
            assert_eq!(cpu.v[1].0, 0x03);
            assert_eq!(cpu.v[15].0, 1);
        }

        #[test]
        fn shr_works() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0b10);
            cpu.v[1] = Wrapping(0x02);
            cpu.v[15] = Wrapping(0xDE);
            cpu.execute_opcode(Opcode::Shr(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0x01);
            assert_eq!(cpu.v[1].0, 0x02);
            assert_eq!(cpu.v[15].0, 0);
        }

        #[test]
        fn shr_vf() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0b11);
            cpu.v[1] = Wrapping(0x03);
            cpu.v[15] = Wrapping(0xDE);
            cpu.execute_opcode(Opcode::Shr(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0x01);
            assert_eq!(cpu.v[1].0, 0x03);
            assert_eq!(cpu.v[15].0, 1);
        }

        #[test]
        fn subn_works() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0x03);
            cpu.v[1] = Wrapping(0x00);
            cpu.v[15] = Wrapping(0xDE);
            cpu.execute_opcode(Opcode::Subn(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0xfd);
            assert_eq!(cpu.v[1].0, 0x00);
            assert_eq!(cpu.v[15].0, 1);
        }

        #[test]
        fn shl_works() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0b01);
            cpu.v[1] = Wrapping(0x02);
            cpu.v[15] = Wrapping(0xDE);
            cpu.execute_opcode(Opcode::Shl(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0b10);
            assert_eq!(cpu.v[1].0, 0x02);
            assert_eq!(cpu.v[15].0, 0);
        }

        #[test]
        fn shl_vf() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0b11000000);
            cpu.v[1] = Wrapping(0x02);
            cpu.v[15] = Wrapping(0xDE);
            cpu.execute_opcode(Opcode::Shl(0, 1), &mut ram, &mut fb);
            assert_eq!(cpu.v[0].0, 0b10000000);
            assert_eq!(cpu.v[1].0, 0x02);
            assert_eq!(cpu.v[15].0, 1);
        }

        #[test]
        fn snexy_skips_when_not_equal() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[3] = Wrapping(0x55);
            cpu.v[4] = Wrapping(0x56);
            cpu.pc = Wrapping(0x200);
            cpu.execute_opcode(Opcode::Snexy(3, 4), &mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x202);
        }

        #[test]
        fn snexy_no_skip_when_equal() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[3] = Wrapping(0x55);
            cpu.v[4] = Wrapping(0x55);
            cpu.pc = Wrapping(0x200);
            cpu.execute_opcode(Opcode::Snexy(3, 4), &mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x200);
        }

        #[test]
        fn ldi_works() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.execute_opcode(Opcode::Ldi(0x123), &mut ram, &mut fb);
            assert_eq!(cpu.i, 0x123);
        }

        #[test]
        fn jpv0_works() {
            let (mut cpu, mut ram, mut fb) = setup();
            cpu.v[0] = Wrapping(0xDE);
            cpu.execute_opcode(Opcode::Jpv0(0x123), &mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x123 + 0xDE);
        }
    }

    // ============================================================
    // Tick — полный цикл выборки-декодирования-выполнения
    // ============================================================
    mod tick {
        use super::{Cpu, framebuffer, ram};
        use std::num::Wrapping;

        #[test]
        fn tick_executes_jp_from_memory() {
            let mut cpu = Cpu::new();
            let mut ram = ram::Ram::new();
            let mut fb = framebuffer::FrameBuffer::new(900, 900).unwrap();

            // JP 0x400 = 0x1400 (big-endian)
            ram.write(0, 0x14).unwrap();
            ram.write(1, 0x00).unwrap();

            cpu.tick(&mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x400);
        }

        #[test]
        fn tick_advances_pc_by_one() {
            let mut cpu = Cpu::new();
            let mut ram = ram::Ram::new();
            let mut fb = framebuffer::FrameBuffer::new(900, 900).unwrap();

            ram.write(0, 0x30).unwrap();
            ram.write(1, 0x01).unwrap();

            cpu.tick(&mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 2);
        }

        #[test]
        fn tick_unknown_opcode_does_not_panic() {
            let mut cpu = Cpu::new();
            let mut ram = ram::Ram::new();
            let mut fb = framebuffer::FrameBuffer::new(900, 900).unwrap();

            // 0xFFFF — неизвестный опкод
            ram.write(0, 0xFF).unwrap();
            ram.write(1, 0xFF).unwrap();

            cpu.tick(&mut ram, &mut fb);
            // При ошибке декодирования PC не меняется — возможен бесконечный цикл
            assert_eq!(cpu.pc.0, 0);
        }

        #[test]
        fn tick_out_of_bounds_does_not_panic() {
            let mut cpu = Cpu::new();
            let mut ram = ram::Ram::new();
            let mut fb = framebuffer::FrameBuffer::new(900, 900).unwrap();

            cpu.pc = Wrapping((ram::RAM_SIZE - 1) as u16);
            cpu.tick(&mut ram, &mut fb);
            // read16 вернул None, PC остался без изменений
            assert_eq!(cpu.pc.0, (ram::RAM_SIZE - 1) as u16);
        }

        #[test]
        fn tick_call_and_return_sequence() {
            let mut cpu = Cpu::new();
            let mut ram = ram::Ram::new();
            let mut fb = framebuffer::FrameBuffer::new(900, 900).unwrap();

            // Адрес 0x200: CALL 0x400 (0x2400)
            ram.write(0x200, 0x24).unwrap();
            ram.write(0x201, 0x00).unwrap();

            // Адрес 0x400: RET (0x00EE)
            ram.write(0x400, 0x00).unwrap();
            ram.write(0x401, 0xEE).unwrap();

            cpu.pc = Wrapping(0x200);

            // CALL
            cpu.tick(&mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x400);
            assert_eq!(cpu.sp.0, 1);

            cpu.tick(&mut ram, &mut fb);
            assert_eq!(cpu.pc.0, 0x202);
            assert_eq!(cpu.sp.0, 0);
        }
    }
}
