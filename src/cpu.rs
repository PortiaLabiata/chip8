use std::num::Wrapping;

use super::framebuffer;
use super::ram;

pub struct Cpu {
    v: [Wrapping<u8>; 16],
    i: u16,
    sp: Wrapping<u8>,
    dt: u8,
    st: u8,
    pc: Wrapping<u16>,
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

            (0x6, x, k1, k2) => Ok(Opcode::Ld(x, k1 << 4 | k2)),

            (0x7, x, k1, k2) => Ok(Opcode::Add(x, k1 << 4 | k2)),

            _ => Err(()),
        };
    }
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            v: [Wrapping(0); 16],
            i: 0,
            sp: Wrapping(0),
            dt: 0,
            st: 0,
            pc: Wrapping(0),
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
                self.v[r as usize] += Wrapping(b);
            }
        }
    }

    // TODO: add better error handling
    pub fn tick(&mut self, memory: &mut ram::Ram, fb: &mut framebuffer::FrameBuffer) {
        let instruction = match memory.read16(self.pc.0) {
            Some(v) => v,
            None => return,
        };

        let opcode = match Opcode::try_from(instruction) {
            Ok(v) => v,
            Err(_) => return,
        };

        self.pc += 2;
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
        fn ld_not_implemented_in_decoder() {
            assert!(Opcode::try_from(0x6012).is_ok());
        }

        #[test]
        fn add_not_implemented_in_decoder() {
            assert!(Opcode::try_from(0x7012).is_ok());
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
