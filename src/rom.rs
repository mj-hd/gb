use crate::utils::*;
use anyhow::{bail, Context, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

#[derive(FromPrimitive, Debug)]
pub enum MbcType {
    RomOnly = 0x00,
    Mbc1 = 0x01,
    Mbc1Ram = 0x02,
    Mbc1RamBattery = 0x03,
    Mbc2 = 0x05,
    Mbc2Battery = 0x06,
    RomRam = 0x08,
    RomRamBattery = 0x09,
    Mmm01 = 0x0b,
    Mmm01Ram = 0x0c,
    Mmm01RamBattery = 0x0d,
    Mbc3 = 0x11,
    Mbc3Ram = 0x12,
    Mbc3RamBattery = 0x13,
}

impl Default for MbcType {
    fn default() -> Self {
        MbcType::RomOnly
    }
}

#[derive(FromPrimitive, Debug)]
pub enum DestinationCode {
    Japanese = 0x00,
    NonJapanese = 0x01,
}

impl Default for DestinationCode {
    fn default() -> Self {
        DestinationCode::Japanese
    }
}

pub struct Rom {
    pub entry_point: [u8; 4],
    pub logo: [u8; 0x0030],
    pub title: [u8; 0x0010],
    pub new_licensee_code: [u8; 2],
    pub sgb_flag: bool,
    pub mbc_type: MbcType,
    pub rom_size: usize,
    pub ram_size: usize,
    pub destination_code: DestinationCode,
    pub old_licensee_code: u8,
    pub mask_rom_version_number: u8,
    pub header_checksum: u8,
    pub global_checksum: [u8; 2],
    pub data: Vec<u8>,
}

impl Default for Rom {
    fn default() -> Self {
        Rom {
            entry_point: Default::default(),
            logo: [0; 0x0030],
            title: Default::default(),
            new_licensee_code: Default::default(),
            sgb_flag: Default::default(),
            mbc_type: Default::default(),
            rom_size: Default::default(),
            ram_size: Default::default(),
            destination_code: Default::default(),
            old_licensee_code: Default::default(),
            mask_rom_version_number: Default::default(),
            header_checksum: Default::default(),
            global_checksum: Default::default(),
            data: Vec::new(),
        }
    }
}

impl fmt::Debug for Rom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rom")
            .field("entry_point", &bytes_to_hex(&self.entry_point[..]))
            .field("logo", &bytes_to_hex(&self.logo[..]))
            .field("title", &bytes_to_hex(&self.title[..]))
            .field("new_licensee_code", &bytes_to_hex(&self.new_licensee_code))
            .field("sgb_flag", &self.sgb_flag)
            .field("mbc_type", &self.mbc_type)
            .field("rom_size", &self.rom_size)
            .field("ram_size", &self.ram_size)
            .field("destination_code", &self.destination_code)
            .field("old_licensee_code", &self.old_licensee_code)
            .field("mask_rom_version_number", &self.mask_rom_version_number)
            .field("header_checksum", &self.header_checksum)
            .field("global_checksum", &bytes_to_hex(&self.global_checksum))
            .field("data", &self.data.len())
            .finish()
    }
}

impl Rom {
    pub fn new(reader: &mut BufReader<File>) -> Result<Rom> {
        let mut rom = Rom::default();

        // @see https://gbdev.io/pandocs/#the-cartridge-header

        reader.seek(SeekFrom::Start(0x0100))?;

        // 0100-0103 - Entry Point
        reader.read_exact(&mut rom.entry_point[..])?;

        // 0104-0133 - Nintendo Logo
        reader.read_exact(&mut rom.logo[..])?;

        // 0134-0143 - Title
        // NOTE: Manufacturer Code, CGB Flagを含む
        reader.read_exact(&mut rom.title[..])?;

        // 0144-0145 - New Licensee Code
        reader.read_exact(&mut rom.new_licensee_code[..])?;

        // 0146 - SGB Flag
        rom.sgb_flag = match reader.take(1).bytes().next() {
            Some(Ok(0x00)) => false,
            Some(Ok(0x03)) => true,
            Some(Ok(unknown)) => bail!("unknown SGB Flag {:#X}", unknown),
            Some(Err(e)) => bail!("error occured while reading the SGB Flag {}", e),
            None => bail!("unexpected EOF while reading the SGB Flag"),
        };

        // 0147 - Cartridge Type
        if let Some(Ok(typ)) = reader.take(1).bytes().next() {
            rom.mbc_type = FromPrimitive::from_u8(typ).context("unknown mbc type")?;
        } else {
            bail!("failed to parse the Cardridge Type");
        }

        // 0148 - ROM Size
        // NOTE: バンク数を読み込んでいない
        rom.rom_size = match reader.take(1).bytes().next() {
            Some(Ok(n @ 0x00..=0x08)) => ((32 * 1024) << n) as usize,
            Some(Ok(0x52)) => (1.1 * 1024.0 * 1024.0) as usize,
            Some(Ok(0x53)) => (1.2 * 1024.0 * 1024.0) as usize,
            Some(Ok(0x54)) => (1.5 * 1024.0 * 1024.0) as usize,
            Some(Ok(unknown)) => bail!("unknown ROM Size {:#X}", unknown),
            Some(Err(e)) => bail!("error occured while reading the ROM Size {}", e),
            None => bail!("unexpected EOF while reading the ROM Size"),
        };

        // 0149 - RAM Size
        rom.ram_size = match reader.take(1).bytes().next() {
            Some(Ok(0x00)) => 0_usize,
            Some(Ok(0x01)) => 2 * 1024 * 1024_usize,
            Some(Ok(0x02)) => 8 * 1024 * 1024_usize,
            Some(Ok(0x03)) => 32 * 1024 * 1024_usize,
            Some(Ok(0x04)) => 128 * 1024 * 1024_usize,
            Some(Ok(0x05)) => 64 * 1024 * 1024_usize,
            Some(Ok(unknown)) => bail!("unknown RAM Size {:#X}", unknown),
            Some(Err(e)) => bail!("error occured while reading the RAM Size {}", e),
            None => bail!("unexpected EOF while reading the RAM Size"),
        };

        // 014A - Destination Code
        if let Some(Ok(code)) = reader.take(1).bytes().next() {
            rom.destination_code =
                FromPrimitive::from_u8(code).context("unknown destination code")?;
        } else {
            bail!("failed to parse the Destination Code");
        }

        // 014B - Old Licensee Code
        rom.old_licensee_code = reader
            .take(1)
            .bytes()
            .next()
            .context("failed to parse the Old Licensee Code")??;

        // 014C - Mask ROM Version number
        rom.mask_rom_version_number = reader
            .take(1)
            .bytes()
            .next()
            .context("failed to parse the Mask ROM Version number")??;

        // 014D - Header Checksum
        rom.header_checksum = reader
            .take(1)
            .bytes()
            .next()
            .context("failed to parse the Header Checksum")??;

        // 014E-014F - Global Checksum
        reader.read_exact(&mut rom.global_checksum[..])?;

        reader.seek(SeekFrom::Start(0x0134))?;

        let mut chksum: u8 = 0;

        for _ in 0x0134..=0x014C {
            if let Some(Ok(b)) = reader.take(1).bytes().next() {
                chksum = chksum.wrapping_sub(b).wrapping_sub(1);
            } else {
                bail!("error occured while checking header chksum");
            }
        }

        if rom.header_checksum != chksum {
            bail!(
                "invalid checksum expected: {}, actual: {}",
                rom.header_checksum,
                chksum
            );
        }

        // TODO 先にraed_to_endしてから読み込んだほうがシンプル
        reader.seek(SeekFrom::Start(0))?;

        reader.read_to_end(&mut rom.data)?;

        if rom.rom_size != rom.data.len() {
            bail!(
                "invalid rom size expected: {}, actual: {}",
                rom.rom_size,
                rom.data.len(),
            );
        }

        Ok(rom)
    }
}
