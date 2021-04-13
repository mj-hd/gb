use crate::rom::{MbcType, Rom};
use anyhow::Result;
use std::cmp::max;

pub trait Mbc {
    fn read(&self, addr: u16) -> Result<u8>;
    fn write(&mut self, addr: u16, val: u8) -> Result<()>;
}

pub fn new_mbc(rom: Rom) -> Box<dyn Mbc + Send> {
    match rom.mbc_type {
        MbcType::RomOnly => Box::new(RomOnly::new(rom)),
        MbcType::Mbc1 | MbcType::Mbc1Ram | MbcType::Mbc1RamBattery => Box::new(Mbc1::new(rom)),
        t => {
            unimplemented!("unimplemented mbc: {:?}", t);
        }
    }
}

pub struct RomOnly {
    rom: Rom,
    ram: [u8; 8 * 1024],
}

impl RomOnly {
    pub fn new(rom: Rom) -> Self {
        RomOnly {
            rom,
            ram: [0; 8 * 1024],
        }
    }
}

impl Mbc for RomOnly {
    fn read(&self, addr: u16) -> Result<u8> {
        if addr >= 0xA000 {
            return Ok(self.ram[(addr - 0xA000) as usize]);
        }

        Ok(self.rom.data[addr as usize])
    }

    fn write(&mut self, addr: u16, val: u8) -> Result<()> {
        if addr >= 0xA000 {
            self.ram[(addr - 0xA000) as usize] = val;

            return Ok(());
        }

        Ok(())
    }
}

enum Mbc1SelectMode {
    ROM,
    RAM,
}

pub struct Mbc1 {
    rom: Rom,
    ram: [u8; 32 * 1024],
    rom_bank: u8,
    ram_bank: u8,

    enable_ram: bool,
    select_mode: Mbc1SelectMode,
}

impl Mbc1 {
    pub fn new(rom: Rom) -> Self {
        Mbc1 {
            rom,
            ram: [0; 32 * 1024],
            rom_bank: 1,
            ram_bank: 0,
            enable_ram: true,
            select_mode: Mbc1SelectMode::ROM,
        }
    }

    fn read_rom_from_bank(&self, addr: u16) -> Result<u8> {
        let base_addr = ((self.rom_bank as u64) * 16 * 1024) as usize;
        let index_addr = (addr - 0x4000) as usize;
        Ok(self.rom.data[base_addr + index_addr])
    }

    fn read_ram_from_bank(&self, addr: u16) -> Result<u8> {
        if !self.enable_ram {
            eprintln!("disabled ram read");

            return Ok(0);
        }

        let base_addr = ((self.ram_bank as u64) * 8 * 1024) as usize;
        let index_addr = (addr - 0xA000) as usize;
        Ok(self.ram[base_addr + index_addr])
    }

    fn write_ram_into_bank(&mut self, addr: u16, val: u8) -> Result<()> {
        if !self.enable_ram {
            eprintln!("disabled ram write");

            return Ok(());
        }

        let base_addr = ((self.ram_bank as u16) * 8 * 1024) as usize;
        let index_addr = (addr - 0xA000) as usize;

        self.ram[base_addr + index_addr] = val;

        Ok(())
    }
}

impl Mbc for Mbc1 {
    fn read(&self, addr: u16) -> Result<u8> {
        match addr {
            0x0000..=0x3FFF => Ok(self.rom.data[addr as usize]),
            0x4000..=0x7FFF => self.read_rom_from_bank(addr),
            0xA000..=0xBFFF => self.read_ram_from_bank(addr),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> Result<()> {
        match addr {
            0x0000..=0x1FFF => match val {
                v if (v & 0x0F) == 0x0A => {
                    self.enable_ram = true;

                    Ok(())
                }
                _ => {
                    self.enable_ram = false;

                    Ok(())
                }
            },
            0x2000..=0x3FFF => {
                let bank = val & 0b00011111;

                self.rom_bank = max(bank, 1);

                Ok(())
            }
            0x4000..=0x5FFF => match self.select_mode {
                Mbc1SelectMode::ROM => {
                    let bank_high = max(val & 0b00000011, 1);

                    self.rom_bank |= bank_high << 5;

                    Ok(())
                }
                Mbc1SelectMode::RAM => {
                    let bank = val & 0b00000011;

                    self.ram_bank = bank;

                    Ok(())
                }
            },
            0x6000..=0x7FFF => {
                self.select_mode = match val {
                    0x01 => Mbc1SelectMode::RAM,
                    _ => Mbc1SelectMode::ROM,
                };

                Ok(())
            }
            addr => self.write_ram_into_bank(addr, val),
        }
    }
}
