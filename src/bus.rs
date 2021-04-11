use crate::joypad::Joypad;
use crate::mbc::Mbc;
use crate::ppu::Ppu;
use crate::timer::Timer;
use anyhow::Result;
use bitfield::bitfield;
use bitmatch::bitmatch;

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
    pub joypad: Joypad,
    pub timer: Timer,
    // apu: Apu,
    ram: [u8; 0x8000],
    hram: [u8; 0x0080],
    mbc: Box<dyn Mbc + Send>,

    pub ie: Ie,

    prev_serial: bool,
    int_serial: bool,
}

impl Bus {
    pub fn new(ppu: Ppu, mbc: Box<dyn Mbc + Send>) -> Self {
        Bus {
            ram: [0; 0x8000],
            hram: [0; 0x0080],
            ie: Default::default(),
            int_serial: false,
            prev_serial: false,
            ppu,
            mbc,
            joypad: Default::default(),
            timer: Default::default(),
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        self.ppu.tick()?;
        self.ppu.tick()?;
        self.timer.tick();
        self.timer.tick();
        self.timer.tick();
        self.timer.tick();
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

    pub fn irq_timer(&self) -> bool {
        self.timer.int
    }

    pub fn set_irq_timer(&mut self, val: bool) {
        self.timer.int = val;
    }

    pub fn irq_serial(&self) -> bool {
        self.int_serial
    }

    pub fn set_irq_serial(&mut self, val: bool) {
        self.int_serial = val;
    }

    pub fn irq_joypad(&self) -> bool {
        self.joypad.int
    }

    pub fn set_irq_joypad(&mut self, val: bool) {
        self.joypad.int = val;
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
            0xFF00 => Ok(self.joypad.read()),
            0xFF01 => self.read_serial(),
            0xFF02 => self.read_serial_ctrl(),
            0xFF04 => Ok(self.timer.read_div()),
            0xFF05 => Ok(self.timer.read_tima()),
            0xFF06 => Ok(self.timer.read_tma()),
            0xFF07 => Ok(self.timer.read_tac()),
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

    #[bitmatch]
    #[allow(clippy::many_single_char_names)]
    pub fn read_irq(&self) -> Result<u8> {
        let v = self.ppu.int_v_blank;
        let l = self.ppu.int_lcd_stat;
        let t = self.timer.int;
        let s = self.int_serial;
        let j = self.joypad.int;

        // let res = bitpack!("000jstlv");

        // println!("IRQ READ: {:#08b}", res);

        Ok(bitpack!("000jstlv"))
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
            0xFF00 => {
                self.joypad.write(val);
                Ok(())
            }
            0xFF01 => self.write_serial(val),
            0xFF02 => self.write_serial_ctrl(val),
            0xFF04 => {
                self.timer.write_div(val);
                Ok(())
            }
            0xFF05 => {
                self.timer.write_tima(val);
                Ok(())
            }
            0xFF06 => {
                self.timer.write_tma(val);
                Ok(())
            }
            0xFF07 => {
                self.timer.write_tac(val);
                Ok(())
            }
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

    #[bitmatch]
    #[allow(clippy::many_single_char_names)]
    pub fn write_irq(&mut self, val: u8) -> Result<()> {
        #[bitmatch]
        let "???jstlv" = val;

        self.ppu.int_v_blank = v > 0;
        self.ppu.int_lcd_stat = l > 0;
        self.timer.int = t > 0;
        self.int_serial = s > 0;
        self.joypad.int = j > 0;

        // println!("IRQ WRITE: {:#08b}", val);

        Ok(())
    }

    pub fn write_serial(&mut self, val: u8) -> Result<()> {
        eprintln!("SERIAL: {:#02X}", val);

        Ok(())
    }

    #[bitmatch]
    pub fn write_serial_ctrl(&mut self, val: u8) -> Result<()> {
        #[bitmatch]
        let "s??????i" = val;

        if i > 0 {
            eprintln!("SERIAL CTRL: INTERNAL CLOCK");
        } else {
            eprintln!("SERIAL CTRL: EXTERNAL CLOCK");
        }

        let cur = if s > 0 {
            eprintln!("SERIAL CTRL: START TRANSFER");

            true
        } else {
            eprintln!("SERIAL CTRL: NO TRANSFER");

            false
        };

        if self.prev_serial && !cur {
            self.int_serial = true;
        }

        self.prev_serial = cur;

        Ok(())
    }
}
