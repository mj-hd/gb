use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::mbc::new_mbc;
use crate::ppu::Ppu;
use crate::rom::Rom;
use anyhow::Result;
use rustyline::Editor;

pub struct Gb {
    cpu: Cpu,
}

impl Gb {
    pub fn new(rom: Rom, rl: Editor<()>) -> Self {
        let mbc = new_mbc(rom);
        let ppu = Ppu::new();
        let bus = Bus::new(ppu, mbc);
        let cpu = Cpu::new(bus, rl);

        Gb { cpu }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.cpu.reset()
    }

    pub fn debug_break(&mut self) -> Result<()> {
        self.cpu.debug_break();

        Ok(())
    }

    pub fn tick(&mut self) -> Result<()> {
        self.cpu.tick()?;
        self.cpu.bus.tick()?;

        Ok(())
    }

    pub fn render(&mut self, frame: &mut [u8]) -> Result<()> {
        self.cpu.bus.ppu.render(frame)
    }
}
