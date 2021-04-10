use crate::utils::bytes_to_hex;
use anyhow::Result;
use bitfield::bitfield;
use image::{ImageBuffer, Rgba};

const VISIBLE_WIDTH: usize = 160;
const VISIBLE_HEIGHT: usize = 144;
const WIDTH: usize = 256;
const HEIGHT: usize = 256;

bitfield! {
    struct LcdControl(u8);
    bg_win_enable, _: 0;
    sprite_enable, _: 1;
    sprite_size, _: 2;
    bg_tile_map_select, _: 3;
    tile_data_select, _: 4;
    window_display_enable, _: 5;
    window_tile_map_select, _: 6;
    lcd_display_enable, _: 7;
}

bitfield! {
    struct LcdStatus(u8);
    ppu_mode, _: 1, 0;
    coincidence_flag, _: 2;
    mode_0_stat_int_enable, _: 3;
    mode_1_stat_int_enable, _: 4;
    mode_2_stat_int_enable, _: 5;
    lyc_ly_stat_int_enable, _: 6;
}

bitfield! {
    #[derive(Default, Copy, Clone)]
    struct SpriteFlags(u8);
    palette_num, _: 4;
    x_flip, _: 5;
    y_flip, _: 6;
    priority, _: 7;
}

#[derive(Debug, Copy, Clone)]
struct Palette([u8; 4]);

impl From<u8> for Palette {
    fn from(val: u8) -> Self {
        Self([
            (val >> 6) & 0b00000011,
            (val >> 4) & 0b00000011,
            (val >> 2) & 0b00000011,
            val & 0b00000011,
        ])
    }
}

impl Into<u8> for Palette {
    fn into(self) -> u8 {
        self.0[0] << 6 | self.0[1] << 4 | self.0[2] << 2 | self.0[3]
    }
}

#[derive(Default, Copy, Clone)]
struct Oam {
    y_pos: u8,
    x_pos: u8,
    tile_num: u8,
    sprite_flag: SpriteFlags,
}

#[derive(PartialEq)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

pub struct Ppu {
    vram: [u8; 8 * 1024],

    mode: Mode,

    lcd_control: LcdControl,
    lcd_status: LcdStatus,
    window_x: u8,
    window_y: u8,
    scroll_x: u8,
    scroll_y: u8,

    cycles: u16,
    lines: u8,

    lines_compare: u8,

    bg_palette: Palette,
    object_palette_0: Palette,
    object_palette_1: Palette,

    pub int_v_blank: bool,
    pub int_lcd_stat: bool,

    x: u8,
    y: u8,

    oam: [Oam; 0xA0],

    pixels: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            vram: [0; 8 * 1024],
            mode: Mode::OamScan,
            lcd_control: LcdControl(0),
            lcd_status: LcdStatus(0),
            window_x: 0,
            window_y: 0,
            scroll_x: 0,
            scroll_y: 0,
            cycles: 0,
            lines: 0x99,
            lines_compare: 0,
            bg_palette: Palette::from(0x00),
            object_palette_0: Palette::from(0x00),
            object_palette_1: Palette::from(0x00),
            x: 0,
            y: 0,
            int_v_blank: false,
            int_lcd_stat: false,
            oam: [Oam::default(); 0xA0],
            pixels: ImageBuffer::new(VISIBLE_WIDTH as u32, VISIBLE_HEIGHT as u32),
        }
    }

    fn color_to_pixel(&self, color: u8) -> Rgba<u8> {
        match color {
            0 => Rgba([0x00, 0x00, 0x00, 0xFF]),
            1 => Rgba([0x55, 0x55, 0x55, 0xFF]),
            2 => Rgba([0xAA, 0xAA, 0xAA, 0xFF]),
            3 => Rgba([0xFF, 0xFF, 0xFF, 0xFF]),
            _ => Rgba([0xFF, 0xFF, 0xFF, 0xFF]),
        }
    }

    fn tile_to_pixel_line(&self, tile_num: u8, row: u8, palette: &Palette) -> [Rgba<u8>; 8] {
        let base_addr = if self.lcd_control.tile_data_select() {
            0x0000u16
        } else {
            0x9000u16 - 0x8000u16
        };

        let index_addr = if self.lcd_control.tile_data_select() {
            (row as u16) * 2 + (tile_num as u16) * 16
        } else {
            ((row as i16) * 2 + (tile_num as i8 as i16) * 16) as u16
        };

        let addr = base_addr.wrapping_add(index_addr);

        let bit = self.vram[addr as usize];
        let color = self.vram[(addr + 1) as usize];

        let mut pixels = [Rgba([0xFF, 0xFF, 0xFF, 0xFF]); 8];

        for i in 1..=8 {
            let palette_num =
                (((bit >> (8 - i)) & 0b00000001) << 1) | (color >> (8 - i) & 0b00000001);

            pixels[i - 1] = self.color_to_pixel(palette.0[palette_num as usize]);
        }

        pixels
    }

    fn bg_tile_map_to_pixel_line(
        &self,
        tile_x: u8,
        tile_y: u8,
        row: u8,
        palette: &Palette,
    ) -> [Rgba<u8>; 8] {
        let base_addr = if self.lcd_control.bg_tile_map_select() {
            0x9C00u16 - 0x8000u16
        } else {
            0x9800u16 - 0x8000u16
        };

        let index_addr = tile_x as u16 + (tile_y as u16) * 32;

        let addr = base_addr.wrapping_add(index_addr);

        let tile_num = self.vram[addr as usize];

        self.tile_to_pixel_line(tile_num, row, &palette)
    }

    pub fn tick(&mut self) -> Result<()> {
        self.cycles += 1;

        if self.cycles >= 456 {
            self.cycles = 0;
            self.lines += 1;
        }

        if self.lines >= 154 {
            self.lines = 0;
        }

        if self.cycles == 80 {
            self.x = 0;
        }

        if self.lines == 0 {
            self.y = 0;
        }

        if self.lines < 144 {
            self.y = self.lines;
            match self.cycles {
                0..=79 => {
                    self.mode = Mode::OamScan;
                }
                80 => {
                    self.mode = Mode::Drawing;
                }
                81..=239 => {
                    self.x += 1;
                }
                240..=455 => {
                    self.mode = Mode::HBlank;
                }
                _ => {}
            }
        }

        if self.lines == 144 {
            self.mode = Mode::VBlank;
            self.int_v_blank = true;
        }

        if self.mode == Mode::Drawing {
            if self.x % 8 == 0 {
                let tile_x = self.x / 8;
                let tile_y = self.y / 8;
                let row = self.y % 8;

                for (i, &pixel) in self
                    .bg_tile_map_to_pixel_line(tile_x, tile_y, row, &self.bg_palette)
                    .into_iter()
                    .enumerate()
                {
                    self.pixels
                        .put_pixel((self.x + i as u8) as u32, self.y as u32, pixel);
                }
            }
        }

        Ok(())
    }

    pub fn read(&self, addr: u16) -> Result<u8> {
        Ok(self.vram[(addr - 0x8000) as usize])
    }

    pub fn write(&mut self, addr: u16, val: u8) -> Result<()> {
        // println!("PPU WRITE: {:#02X}={:#02X}", addr, val);
        self.vram[(addr - 0x8000) as usize] = val;
        Ok(())
    }

    pub fn read_oam(&self, addr: u16) -> Result<u8> {
        Ok(0)
    }

    pub fn write_oam(&mut self, addr: u16, val: u8) -> Result<()> {
        Ok(())
    }

    pub fn read_lcd_control(&self) -> Result<u8> {
        Ok(self.lcd_control.0)
    }

    pub fn write_lcd_control(&mut self, val: u8) -> Result<()> {
        self.lcd_control = LcdControl(val);
        Ok(())
    }

    pub fn read_lcd_status(&self) -> Result<u8> {
        Ok(self.lcd_status.0)
    }

    pub fn write_lcd_status(&mut self, val: u8) -> Result<()> {
        self.lcd_status = LcdStatus(val);
        Ok(())
    }

    pub fn read_scroll_y(&self) -> Result<u8> {
        Ok(self.scroll_y)
    }

    pub fn write_scroll_y(&mut self, val: u8) -> Result<()> {
        self.scroll_y = val;
        Ok(())
    }

    pub fn read_scroll_x(&self) -> Result<u8> {
        Ok(self.scroll_x)
    }

    pub fn write_scroll_x(&mut self, val: u8) -> Result<()> {
        self.scroll_x = val;
        Ok(())
    }

    pub fn read_lines(&self) -> Result<u8> {
        Ok(self.lines)
    }

    pub fn read_line_compare(&self) -> Result<u8> {
        Ok(self.lines_compare)
    }

    pub fn write_line_compare(&mut self, val: u8) -> Result<()> {
        self.lines_compare = val;
        Ok(())
    }

    pub fn read_window_x(&self) -> Result<u8> {
        Ok(self.window_x)
    }

    pub fn write_window_x(&mut self, val: u8) -> Result<()> {
        self.window_x = val;
        Ok(())
    }

    pub fn read_window_y(&self) -> Result<u8> {
        Ok(self.window_y)
    }

    pub fn write_window_y(&mut self, val: u8) -> Result<()> {
        self.window_y = val;
        Ok(())
    }

    pub fn read_bg_palette(&self) -> Result<u8> {
        Ok(self.bg_palette.into())
    }

    pub fn write_bg_palette(&mut self, val: u8) -> Result<()> {
        self.bg_palette = Palette::from(val);
        Ok(())
    }

    pub fn read_object_palette_0(&self) -> Result<u8> {
        Ok(self.object_palette_0.into())
    }

    pub fn write_object_palette_0(&mut self, val: u8) -> Result<()> {
        self.object_palette_0 = Palette::from(val);
        Ok(())
    }

    pub fn read_object_palette_1(&self) -> Result<u8> {
        Ok(self.object_palette_1.into())
    }

    pub fn write_object_palette_1(&mut self, val: u8) -> Result<()> {
        self.object_palette_1 = Palette::from(val);
        Ok(())
    }

    pub fn render(&mut self, frame: &mut [u8]) -> Result<()> {
        frame.copy_from_slice(&self.pixels.clone().into_raw());
        Ok(())
    }
}
