use std::fs;
use std::io::Read;

pub const RAM_SIZE: usize = 4096;
pub const STACK_SIZE: usize = 64;
pub const FONT_SIZE: usize = 80;

pub struct Ram {
    stack: [u16; STACK_SIZE / 2],
    memory: [u8; RAM_SIZE],
}

pub struct Program {
    data: Vec<u8>,
}

impl Program {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let mut file = fs::File::open(path)?;
        let mut data = Vec::new();

        file.read_to_end(&mut data)?;
        Ok(Self { data })
    }
}

impl Ram {
    pub fn new() -> Self {
        let font = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        let mut memory = [0; 4096];
        for (font , ram) in font.iter().zip(memory.iter_mut()) {
            *ram = *font;
        }

        Ram {
            stack: [0; STACK_SIZE / 2],
            memory: [0; 4096],
        }
    }

    pub fn read(&self, addr: u16) -> Option<u8> {
        self.memory.get(addr as usize).copied()
    }

    pub fn write(&mut self, addr: u16, v: u8) -> Result<(), ()> {
        match self.memory.get_mut(addr as usize) {
            Some(r) => *r = v,
            None => return Err(()),
        }
        Ok(())
    }

    pub fn read16(&mut self, addr: u16) -> Option<u16> {
        let low = self.read(addr)? as u16;
        let high = self.read(addr + 1)? as u16;
        Some((low << 8) | high)
    }

    // TODO: add handling of RAM overflow
    pub fn load_program(&mut self, prog: Program) {
        for (ram_byte, rom_byte) in self.memory.iter_mut().zip(prog.data.iter()) {
            *ram_byte = *rom_byte;
        }
    }

    pub fn write_stack(&mut self, addr: u8, v: u16) -> Result<(), ()> {
        let ptr = match self.stack.get_mut(addr as usize) {
            Some(v) => v,
            None => return Err(()),
        };

        *ptr = v;
        Ok(())
    }

    pub fn read_stack(&self, addr: u8) -> Option<u16> {
        self.stack.get(addr as usize).map(|v| *v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;
    use std::io::Write;

    // ============================================================
    // Инициализация
    // ============================================================

    #[test]
    fn new_initializes_memory_to_zero() {
        let ram = Ram::new();
        for i in 0..RAM_SIZE as u16 {
            assert_eq!(ram.read(i), Some(0), "memory[{}] должна быть 0", i);
        }
    }

    #[test]
    fn new_initializes_stack_to_zero() {
        let ram = Ram::new();
        let stack_len = STACK_SIZE / 2;
        for i in 0..stack_len as u8 {
            assert_eq!(ram.read_stack(i), Some(0), "stack[{}] должен быть 0", i);
        }
    }

    // ============================================================
    // Чтение/запись памяти (memory)
    // ============================================================

    #[test]
    fn read_valid_address() {
        let mut ram = Ram::new();
        ram.memory[0x200] = 0xAB;
        assert_eq!(ram.read(0x200), Some(0xAB));
    }

    #[test]
    fn read_first_byte() {
        let ram = Ram::new();
        assert_eq!(ram.read(0x0000), Some(0));
    }

    #[test]
    fn read_last_valid_byte() {
        let mut ram = Ram::new();
        ram.memory[RAM_SIZE - 1] = 0xFF;
        assert_eq!(ram.read((RAM_SIZE - 1) as u16), Some(0xFF));
    }

    #[test]
    fn read_out_of_bounds_returns_none() {
        let ram = Ram::new();
        assert_eq!(ram.read(RAM_SIZE as u16), None);
        assert_eq!(ram.read(0xFFFF), None);
    }

    #[test]
    fn write_valid_address() {
        let mut ram = Ram::new();
        assert!(ram.write(0x200, 0xAB).is_ok());
        assert_eq!(ram.memory[0x200], 0xAB);
    }

    #[test]
    fn write_last_valid_address() {
        let mut ram = Ram::new();
        assert!(ram.write((RAM_SIZE - 1) as u16, 0xFF).is_ok());
        assert_eq!(ram.memory[RAM_SIZE - 1], 0xFF);
    }

    #[test]
    fn write_out_of_bounds_returns_err() {
        let mut ram = Ram::new();
        assert_eq!(ram.write(RAM_SIZE as u16, 0xAB), Err(()));
        assert_eq!(ram.write(0xFFFF, 0xAB), Err(()));
    }

    // ============================================================
    // 16-битное чтение (big-endian)
    // ============================================================

    #[test]
    fn read16_valid_big_endian() {
        let mut ram = Ram::new();
        ram.memory[0x200] = 0x12;
        ram.memory[0x201] = 0x34;
        assert_eq!(ram.read16(0x200), Some(0x1234));
    }

    #[test]
    fn read16_zero() {
        let mut ram = Ram::new();
        assert_eq!(ram.read16(0x200), Some(0x0000));
    }

    #[test]
    fn read16_max_value() {
        let mut ram = Ram::new();
        ram.memory[0x300] = 0xFF;
        ram.memory[0x301] = 0xFF;
        assert_eq!(ram.read16(0x300), Some(0xFFFF));
    }

    #[test]
    fn read16_at_upper_boundary() {
        let mut ram = Ram::new();
        ram.memory[RAM_SIZE - 2] = 0xDE;
        ram.memory[RAM_SIZE - 1] = 0xAD;
        assert_eq!(ram.read16((RAM_SIZE - 2) as u16), Some(0xDEAD));
    }

    #[test]
    fn read16_second_byte_out_of_bounds() {
        let mut ram = Ram::new();
        // addr = RAM_SIZE - 1 → addr + 1 выходит за границу
        assert_eq!(ram.read16((RAM_SIZE - 1) as u16), None);
    }

    #[test]
    fn read16_first_byte_out_of_bounds() {
        let mut ram = Ram::new();
        assert_eq!(ram.read16(RAM_SIZE as u16), None);
    }

    // ============================================================
    // Загрузка программы
    // ============================================================

    #[test]
    fn load_program_starts_at_zero() {
        let mut ram = Ram::new();
        let prog = Program {
            data: vec![0x00, 0xE0, 0x12, 0x34],
        };
        ram.load_program(prog);
        assert_eq!(ram.memory[0], 0x00);
        assert_eq!(ram.memory[1], 0xE0);
        assert_eq!(ram.memory[2], 0x12);
        assert_eq!(ram.memory[3], 0x34);
    }

    #[test]
    fn load_program_partial_load() {
        let mut ram = Ram::new();
        let prog = Program {
            data: vec![0xAB; 100],
        };
        ram.load_program(prog);
        assert_eq!(ram.memory[0], 0xAB);
        assert_eq!(ram.memory[99], 0xAB);
        assert_eq!(ram.memory[100], 0x00); // остальное не тронуто
    }

    #[test]
    fn load_program_does_not_exceed_ram() {
        let mut ram = Ram::new();
        let prog = Program {
            data: vec![0xFF; RAM_SIZE + 100],
        };
        ram.load_program(prog);
        for i in 0..RAM_SIZE {
            assert_eq!(ram.memory[i], 0xFF, "memory[{}] должна быть 0xFF", i);
        }
    }

    // ============================================================
    // Стек
    // ============================================================

    #[test]
    fn stack_write_read_roundtrip() {
        let mut ram = Ram::new();
        assert!(ram.write_stack(0, 0x1234).is_ok());
        assert_eq!(ram.read_stack(0), Some(0x1234));
    }

    #[test]
    fn stack_multiple_values() {
        let mut ram = Ram::new();
        ram.write_stack(0, 0x0000).unwrap();
        ram.write_stack(1, 0x1234).unwrap();
        ram.write_stack(2, 0xFFFF).unwrap();
        assert_eq!(ram.read_stack(0), Some(0x0000));
        assert_eq!(ram.read_stack(1), Some(0x1234));
        assert_eq!(ram.read_stack(2), Some(0xFFFF));
    }

    #[test]
    fn stack_last_valid_address() {
        let mut ram = Ram::new();
        let last = (STACK_SIZE / 2 - 1) as u8;
        assert!(ram.write_stack(last, 0xABCD).is_ok());
        assert_eq!(ram.read_stack(last), Some(0xABCD));
    }

    #[test]
    fn stack_write_out_of_bounds() {
        let mut ram = Ram::new();
        assert_eq!(ram.write_stack((STACK_SIZE / 2) as u8, 0x1234), Err(()));
        assert_eq!(ram.write_stack(0xFF, 0x1234), Err(()));
    }

    #[test]
    fn stack_read_out_of_bounds() {
        let ram = Ram::new();
        assert_eq!(ram.read_stack((STACK_SIZE / 2) as u8), None);
        assert_eq!(ram.read_stack(0xFF), None);
    }

    // ============================================================
    // Program::new (файловый ввод-вывод)
    // ============================================================

    #[test]
    fn program_new_reads_file_correctly() {
        let path = "test_chip8_program.ch8";
        let expected = vec![0x00, 0xE0, 0x12, 0x34, 0xFF, 0xFF];
        {
            let mut file = std::fs::File::create(path).unwrap();
            file.write_all(&expected).unwrap();
        }
        let prog = Program::new(path).unwrap();
        assert_eq!(prog.data, expected);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn program_new_file_not_found() {
        let result = Program::new("nonexistent_file.ch8");
        assert!(result.is_err());
    }

    #[test]
    fn correct_endianness() {
        let mut ram = Ram::new();
        assert_matches!(ram.write(0x0000, 0x01), Ok(_));
        assert_matches!(ram.write(0x0001, 0x02), Ok(_));
        let v = match ram.read16(0x0000) {
            Some(v) => v,
            None => {
                assert!(false);
                0
            }
        };

        assert_eq!(v, 0x0102);
    }
}
