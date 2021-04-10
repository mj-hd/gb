use crate::mbc::Mbc;
use crate::ppu::Ppu;
use anyhow::Result;
use bitfield::bitfield;

bitfield! {
    #[derive(Default)]
    pub struct Ie(u8);
    impl Debug;
    pub v_blank, set_v_blank: 0;
    pub lcd_stat, set_lcd_stat: 1;
    pub timer, set_timer: 2;
    pub serial, set_serial: 3;
    pub joypad, set_joypad: 4;
}

pub struct Bus {
    pub ppu: Ppu,
    // apu: Apu,
    ram: [u8; 0x8000],
    hram: [u8; 0x0080],
    // joyCon: JoyCon,
    mbc: Box<dyn Mbc + Send>,

    pub ie: Ie,

    int_serial: bool,
}

impl Bus {
    pub fn new(ppu: Ppu, mbc: Box<dyn Mbc + Send>) -> Self {
        Bus {
            ram: [0; 0x8000],
            hram: [0; 0x0080],
            ie: Default::default(),
            int_serial: false,
            ppu,
            mbc,
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        self.ppu.tick()?;
        // self.apu.tick()?;

        Ok(())
    }

    pub fn irq_v_blank(&self) -> bool {
        self.ppu.int_v_blank
    }

    pub fn set_irq_v_blank(&mut self, val: bool) {
        self.ppu.int_v_blank = val;
    }

    pub fn irq_lcd_stat(&self) -> bool {
        self.ppu.int_lcd_stat
    }

    pub fn set_irq_lcd_stat(&mut self, val: bool) {
        self.ppu.int_lcd_stat = val;
    }

    pub fn irq_serial(&self) -> bool {
        self.int_serial
    }

    pub fn set_irq_serial(&mut self, val: bool) {
        self.int_serial = val;
    }

    pub fn read(&self, addr: u16) -> Result<u8> {
        match addr {
            0x0000..=0x7FFF => self.mbc.read(addr),
            0x8000..=0x9FFF => self.ppu.read(addr),
            0xA000..=0xBFFF => self.mbc.read(addr),
            0xC000..=0xDFFF => Ok(self.ram[(addr - 0xC000) as usize]),
            0xE000..=0xFDFF => Ok(self.ram[(addr - 0xE000) as usize]),
            0xFE00..=0xFE9F => self.ppu.read_oam(addr),
            0xFEA0..=0xFEFF => Ok(0),
            0xFF01 => self.read_serial(),
            0xFF02 => self.read_serial_ctrl(),
            0xFF0F => self.read_irq(),
            0xFF40 => self.ppu.read_lcd_control(),
            0xFF41 => self.ppu.read_lcd_status(),
            0xFF42 => self.ppu.read_scroll_y(),
            0xFF43 => self.ppu.read_scroll_x(),
            0xFF44 => self.ppu.read_lines(),
            0xFF45 => self.ppu.read_line_compare(),
            0xFF47 => self.ppu.read_bg_palette(),
            0xFF48 => self.ppu.read_object_palette_0(),
            0xFF49 => self.ppu.read_object_palette_1(),
            0xFF4A => self.ppu.read_window_y(),
            0xFF4B => self.ppu.read_window_x(),
            0xFF80..=0xFFFE => Ok(self.hram[(addr - 0xFF80) as usize]),
            0xFFFF => Ok(self.ie.0),
            _ => Ok(0),
        }
    }

    pub fn read_word(&self, addr: u16) -> Result<u16> {
        let low = self.read(addr)?;
        let high = self.read(addr + 1)?;

        Ok(((high as u16) << 8) | (low as u16))
    }

    pub fn read_irq(&self) -> Result<u8> {
        let mut irq = 0;

        if self.ppu.int_v_blank {
            irq &= 0b00000001;
        }

        if self.ppu.int_lcd_stat {
            irq &= 0b00000010;
        }

        if self.int_serial {
            irq &= 0b00001000;
        }

        Ok(irq)
    }

    pub fn read_serial(&self) -> Result<u8> {
        // シリアル通信は一旦実装せず、デバッグ用途にだけ使う
        Ok(0)
    }

    pub fn read_serial_ctrl(&self) -> Result<u8> {
        // シリアル通信は一旦実装せず、デバッグ用途にだけ使う
        Ok(0)
    }

    pub fn write(&mut self, addr: u16, val: u8) -> Result<()> {
        match addr {
            0x0000..=0x7FFF => self.mbc.write(addr, val),
            0x8000..=0x9FFF => self.ppu.write(addr, val),
            0xA000..=0xBFFF => self.mbc.write(addr, val),
            0xC000..=0xDFFF => {
                self.ram[(addr - 0xC000) as usize] = val;
                Ok(())
            }
            0xE000..=0xFDFF => {
                self.ram[(addr - 0xE000) as usize] = val;
                Ok(())
            }
            0xFE00..=0xFE9F => self.ppu.write_oam(addr, val),
            0xFEA0..=0xFEFF => Ok(()),
            0xFF01 => self.write_serial(val),
            0xFF02 => self.write_serial_ctrl(val),
            0xFF0F => self.write_irq(val),
            0xFF40 => self.ppu.write_lcd_control(val),
            0xFF41 => self.ppu.write_lcd_status(val),
            0xFF42 => self.ppu.write_scroll_y(val),
            0xFF43 => self.ppu.write_scroll_x(val),
            0xFF45 => self.ppu.write_line_compare(val),
            0xFF47 => self.ppu.write_bg_palette(val),
            0xFF48 => self.ppu.write_object_palette_0(val),
            0xFF49 => self.ppu.write_object_palette_1(val),
            0xFF4A => self.ppu.write_window_y(val),
            0xFF4B => self.ppu.write_window_x(val),
            0xFF80..=0xFFFE => {
                self.hram[(addr - 0xFF80) as usize] = val;
                Ok(())
            }
            0xFFFF => {
                self.ie.0 = val;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn write_word(&mut self, addr: u16, val: u16) -> Result<()> {
        let low = (val & 0x00FF) as u8;
        let high = (val >> 8) as u8;

        self.write(addr, low)?;
        self.write(addr + 1, high)?;

        Ok(())
    }

    pub fn write_irq(&mut self, val: u8) -> Result<()> {
        self.ppu.int_v_blank = val & 0b00000001 > 0;
        self.ppu.int_lcd_stat = val & 0b00000010 > 0;
        self.int_serial = val & 0b00001000 > 0;

        Ok(())
    }

    pub fn write_serial(&mut self, val: u8) -> Result<()> {
        eprintln!("SERIAL: {:#02X}", val);

        Ok(())
    }

    pub fn write_serial_ctrl(&mut self, val: u8) -> Result<()> {
        if val & 0b00000001 > 0 {
            eprintln!("SERIAL CTRL: INTERNAL CLOCK");
        } else {
            eprintln!("SERIAL CTRL: EXTERNAL CLOCK");
        }

        if val & 0b10000000 > 0 {
            eprintln!("SERIAL CTRL: START TRANSFER");
        } else {
            eprintln!("SERIAL CTRL: NO TRANSFER");
        }

        Ok(())
    }
}
