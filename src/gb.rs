use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::debugger::Debugger;
use crate::mbc::RomOnly;
use crate::ppu::Ppu;
use crate::rom::Rom;
use anyhow::Result;

pub struct Gb {
    cpu: Cpu,
}

impl Gb {
    pub fn new(rom: Rom, debugger: Debugger) -> Self {
        let mbc = Box::new(RomOnly::new(rom));
        let ppu = Ppu::new();
        let bus = Bus::new(ppu, mbc);
        let cpu = Cpu::new(bus, debugger);

        Gb { cpu }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.cpu.reset()
    }

    pub fn tick(&mut self) -> Result<()> {
        self.cpu.tick()?;

        Ok(())
    }

    pub fn render(&mut self, frame: &mut [u8]) -> Result<()> {
        self.cpu.bus.ppu.render(frame)
    }
}
