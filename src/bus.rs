use crate::mbc::Mbc;
use crate::ppu::Ppu;
use anyhow::Result;

pub struct Bus {
    pub ppu: Ppu,
    // apu: Apu,
    ram: [u8; 0x8000],
    hram: [u8; 0x0080],
    // joyCon: JoyCon,
    mbc: Box<dyn Mbc>,
}

impl Bus {
    pub fn new(ppu: Ppu, mbc: Box<dyn Mbc>) -> Self {
        Bus {
            ram: [0; 0x8000],
            hram: [0; 0x0080],
            ppu,
            mbc,
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        self.ppu.tick()?;
        // self.apu.tick()?;

        Ok(())
    }

    pub fn read(&self, addr: u16) -> Result<u8> {
        match addr {
            0x0000..=0x7FFF => self.mbc.read_rom(addr),
            0x8000..=0x9FFF => self.ppu.read(addr - 0x8000),
            0xA000..=0xBFFF => self.mbc.read_ram(addr - 0xA000),
            0xC000..=0xDFFF => Ok(self.ram[(addr - 0xC000) as usize]),
            0xE000..=0xFDFF => Ok(self.ram[(addr - 0xE000) as usize]),
            0xFE00..=0xFE9F => self.ppu.read_oam(addr - 0xFE00),
            0xFEA0..=0xFEFF => Ok(0),
            0xFF00..=0xFF7F => {
                // TODO IOポートの実装
                // Ok(self.io.read(addr - 0xFF00))
                Ok(0)
            }
            0xFF80..=0xFFFE => Ok(self.hram[(addr - 0xFF80) as usize]),
            0xFFFF => {
                // TODO 割り込みの実装
                // Ok(self.interrupt)
                Ok(0)
            }
        }
    }

    pub fn read_word(&self, addr: u16) -> Result<u16> {
        let high = self.read(addr)?;
        let low = self.read(addr + 1)?;

        Ok(((high as u16) << 8) | (low as u16))
    }

    pub fn write(&mut self, addr: u16, val: u8) -> Result<()> {
        match addr {
            0x0000..=0x7FFF => self.mbc.write_rom(addr, val),
            0x8000..=0x9FFF => self.ppu.write(addr - 0x8000, val),
            0xA000..=0xBFFF => self.mbc.write_ram(addr - 0xA000, val),
            0xC000..=0xDFFF => {
                self.ram[(addr - 0xC000) as usize] = val;
                Ok(())
            }
            0xE000..=0xFDFF => {
                self.ram[(addr - 0xE000) as usize] = val;
                Ok(())
            }
            0xFE00..=0xFE9F => self.ppu.write_oam(addr - 0xFE00, val),
            0xFEA0..=0xFEFF => Ok(()),
            0xFF00..=0xFF7F => {
                // TODO IOポートの実装
                // self.io.write(addr - 0xFF00, val);
                Ok(())
            }
            0xFF80..=0xFFFE => {
                self.hram[(addr - 0xFF80) as usize] = val;
                Ok(())
            }
            0xFFFF => {
                // TODO 割り込みの実装
                // self.interrupt = val;
                Ok(())
            }
        }
    }

    pub fn write_word(&mut self, addr: u16, val: u16) -> Result<()> {
        let high = (val >> 8) as u8;
        let low = (val & 0x0F) as u8;

        self.write(addr, high)?;
        self.write(addr + 1, low)?;

        Ok(())
    }
}
