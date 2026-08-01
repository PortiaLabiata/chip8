mod cpu;
mod framebuffer;
mod ram;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut framebuffer = framebuffer::FrameBuffer::new(900, 900)?;
    let mut cpu = cpu::Cpu::new();
    let mut memory = ram::Ram::new();

    let program = ram::Program::new("3-corax+.ch8")?;
    memory.load_program(program);

    while framebuffer.run() {
        cpu.tick(&mut memory, &mut framebuffer);
    }

    Ok(())
}
