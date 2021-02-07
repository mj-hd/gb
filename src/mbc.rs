use crate::rom::Rom;
use anyhow::Result;

pub trait Mbc {
    fn read_rom(&self, addr: u16) -> Result<u8>;
    fn read_ram(&self, addr: u16) -> Result<u8>;
    fn write_rom(&mut self, addr: u16, val: u8) -> Result<()>;
    fn write_ram(&mut self, addr: u16, val: u8) -> Result<()>;
}

pub struct RomOnly {
    rom: Rom,
}

impl RomOnly {
    pub fn new(rom: Rom) -> Self {
        RomOnly { rom }
    }
}

impl Mbc for RomOnly {
    fn read_rom(&self, addr: u16) -> Result<u8> {
        Ok(self.rom.data[addr as usize])
    }

    fn read_ram(&self, addr: u16) -> Result<u8> {
        Ok(0)
    }

    fn write_rom(&mut self, addr: u16, val: u8) -> Result<()> {
        Ok(())
    }

    fn write_ram(&mut self, addr: u16, val: u8) -> Result<()> {
        Ok(())
    }
}
