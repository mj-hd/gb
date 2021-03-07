use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::mbc::RomOnly;
use crate::rom::Rom;
use anyhow::Result;

pub struct Gb {
    cpu: Cpu,
}

impl Gb {
    pub fn new(rom: Rom) -> Self {
        let mbc = Box::new(RomOnly::new(rom));
        let bus = Bus::new(mbc);
        let cpu = Cpu::new(bus);

        Gb { cpu }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.cpu.reset()
    }

    pub fn tick(&mut self) -> Result<()> {
        self.cpu.tick()?;

        Ok(())
    }
}
