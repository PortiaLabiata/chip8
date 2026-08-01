mod cpu;
mod framebuffer;
mod ram;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut framebuffer = framebuffer::FrameBuffer::new(900, 900)?;
    let mut cpu = cpu::Cpu::new();
    let mut memory = ram::Ram::new();

    let args: Vec<String> = std::env::args().collect();
    let filename = match args.get(1) {
        Some(v) => v,
        None => {
            let e = std::io::ErrorKind::NotFound;
            return Err(Box::new(std::io::Error::new(e, "File not found")));
        }
    };

    let program = ram::Program::new(filename)?;
    memory.load_program(program);

    while framebuffer.run() {
        cpu.tick(&mut memory, &mut framebuffer);
    }

    Ok(())
}
