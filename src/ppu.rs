use anyhow::Result;
use bitfield::bitfield;
use bitmatch::bitmatch;
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
    impl Debug;
    palette_num, _: 4;
    x_flip, _: 5;
    y_flip, _: 6;
    priority, _: 7;
}

#[derive(Debug, Copy, Clone)]
struct Palette([u8; 4]);

impl From<u8> for Palette {
    #[bitmatch]
    fn from(val: u8) -> Self {
        #[bitmatch]
        let "ddccbbaa" = val;

        Self([a, b, c, d])
    }
}

impl From<Palette> for u8 {
    #[bitmatch]
    #[allow(clippy::many_single_char_names)]
    fn from(p: Palette) -> Self {
        let a = p.0[0];
        let b = p.0[1];
        let c = p.0[2];
        let d = p.0[3];

        bitpack!("ddccbbaa")
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct Oam {
    y_pos: u8,
    x_pos: u8,
    tile_num: u8,
    sprite_flag: SpriteFlags,
}

#[derive(Debug, PartialEq)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

type ColorIndex = u8;

#[derive(Debug, Copy, Clone)]
struct OamColor {
    index: ColorIndex,
    color: u8,
    blend: bool,
}

impl Default for OamColor {
    fn default() -> Self {
        Self {
            index: 0,
            blend: false,
            color: 0,
        }
    }
}

impl OamColor {
    fn from_indexes(indexes: [ColorIndex; 8], blend: bool, palette: &Palette) -> [OamColor; 8] {
        let mut colors: [OamColor; 8] = [Default::default(); 8];

        for (j, &index) in indexes.iter().enumerate() {
            colors[j] = OamColor {
                index,
                blend,
                color: palette.0[index as usize],
            }
        }

        colors
    }
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
    buffer: Vec<Oam>,

    bg_line: [ColorIndex; WIDTH],
    oam_line: [OamColor; WIDTH],
    cur_bg: [ColorIndex; 8],
    drawing_window: bool,

    pixels: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            vram: [0; 8 * 1024],
            mode: Mode::VBlank,
            lcd_control: LcdControl(0),
            lcd_status: LcdStatus(0),
            window_x: 0,
            window_y: 0,
            scroll_x: 0,
            scroll_y: 0,
            cycles: 0,
            lines: 0,
            lines_compare: 0,
            bg_palette: Palette::from(0x00),
            object_palette_0: Palette::from(0x00),
            object_palette_1: Palette::from(0x00),
            x: 0,
            y: 0,
            int_v_blank: false,
            int_lcd_stat: false,
            oam: [Oam::default(); 0xA0],
            bg_line: [0; WIDTH],
            oam_line: [Default::default(); WIDTH],
            cur_bg: [0; 8],
            drawing_window: false,
            buffer: Vec::new(),
            pixels: ImageBuffer::new(VISIBLE_WIDTH as u32, VISIBLE_HEIGHT as u32),
        }
    }

    fn color_to_pixel(&self, color: u8) -> Rgba<u8> {
        match color {
            0 => Rgba([0xD8, 0xF7, 0xD7, 0xFF]),
            1 => Rgba([0x6C, 0xA6, 0x6B, 0xFF]),
            2 => Rgba([0x20, 0x59, 0x4A, 0xFF]),
            3 => Rgba([0x00, 0x14, 0x1B, 0xFF]),
            _ => Rgba([0xFF, 0xFF, 0xFF, 0xFF]),
        }
    }

    #[bitmatch]
    #[allow(clippy::many_single_char_names)]
    fn tile_to_indexes(&self, tile_num: u8, row: u8, signed: bool) -> [ColorIndex; 8] {
        let base_addr = if signed {
            0x9000u16 - 0x8000u16
        } else {
            0x0000u16
        };

        let index_addr = if signed {
            ((row as i16) * 2 + (tile_num as i8 as i16) * 16) as u16
        } else {
            (row as u16) * 2 + (tile_num as u16) * 16
        };

        let addr = base_addr.wrapping_add(index_addr);

        let bit = self.vram[addr as usize];
        let color = self.vram[(addr + 1) as usize];

        let mut indexes = [0; 8];

        #[bitmatch]
        let "acegikmo" = bit;

        #[bitmatch]
        let "bdfhjlnp" = color;

        #[bitmatch]
        let "aabbccddeeffgghh" = bitpack!("abcdefghijklmnop");

        for (j, &index) in [a, b, c, d, e, f, g, h].iter().enumerate() {
            indexes[j] = index as u8;
        }

        indexes
    }

    fn tile_map_to_colors(&self, tile_x: u8, tile_y: u8, row: u8, high: bool) -> [ColorIndex; 8] {
        let base_addr = if high {
            0x9C00u16 - 0x8000u16
        } else {
            0x9800u16 - 0x8000u16
        };

        let index_addr = tile_x as u16 + (tile_y as u16) * 32;

        let addr = base_addr.wrapping_add(index_addr);

        let tile_num = self.vram[addr as usize];

        self.tile_to_indexes(tile_num, row, !self.lcd_control.tile_data_select())
    }

    fn oam_to_colors(&self, oam: &Oam) -> [OamColor; 8] {
        let mut row = self.y + 16 - oam.y_pos;
        let mut tile = oam.tile_num;

        if self.lcd_control.sprite_size() && row >= 8 {
            row -= 8;
            tile |= 0b0000001;
        } else {
            tile &= 0b1111110;
        }

        if oam.sprite_flag.y_flip() {
            row = 7 - row;
        }

        let palette = if oam.sprite_flag.palette_num() {
            &self.object_palette_1
        } else {
            &self.object_palette_0
        };

        let blend = oam.sprite_flag.priority();

        let mut colors =
            OamColor::from_indexes(self.tile_to_indexes(tile, row, false), blend, palette);

        if oam.sprite_flag.x_flip() {
            colors.reverse();
        }

        colors
    }

    fn scan_oam(&mut self, i: usize) {
        let size = if self.lcd_control.sprite_size() {
            16
        } else {
            8
        };

        let oam = self.oam[i];
        let cur_y = self.lines as u16 + 16;
        let target_y = oam.y_pos as u16;

        if oam.x_pos > 8 && cur_y < target_y + size && target_y <= cur_y && self.buffer.len() < 10 {
            self.buffer.push(oam);
        }
    }

    fn draw_bg(&mut self) {
        if self.drawing_window {
            return;
        }

        let cx = self.x.wrapping_add(self.scroll_x);
        let cy = self.y.wrapping_add(self.scroll_y);
        let col = cx % 8;
        let row = cy % 8;
        let tile_x = cx / 8;
        let tile_y = cy / 8;

        if col == 0 || self.x == 0 {
            self.cur_bg =
                self.tile_map_to_colors(tile_x, tile_y, row, self.lcd_control.bg_tile_map_select());
        }
        self.bg_line[self.x as usize] = self.cur_bg[col as usize];
    }

    fn draw_window(&mut self) {
        if !self.drawing_window && !(self.x + 7 == self.window_x && self.y >= self.window_y) {
            return;
        }

        self.drawing_window = true;

        let cx = self.x.wrapping_sub(self.window_x);
        let cy = self.y.wrapping_sub(self.window_y);
        let col = cx % 8;
        let row = cy % 8;
        let tile_x = cx / 8;
        let tile_y = cy / 8;

        if col == 0 || self.x == 0 {
            self.cur_bg = self.tile_map_to_colors(
                tile_x,
                tile_y,
                row,
                self.lcd_control.window_tile_map_select(),
            );
        }
        self.bg_line[self.x as usize] = self.cur_bg[col as usize];
    }

    fn draw_sprite(&mut self) {
        for oam in self.buffer.iter() {
            if oam.x_pos == self.x + 8 {
                let x = self.x as usize;

                let colors = self.oam_to_colors(oam);

                self.oam_line[x..(x + 8)].copy_from_slice(&colors[..]);
            }
        }
    }

    fn put_pixels(&mut self, x: u8) {
        let x = x as usize;
        let index = self.bg_line[x] as usize;
        let mut color = self.bg_palette.0[index];

        let oam = self.oam_line[x];

        if (!oam.blend || index == 0) && oam.index != 0 {
            color = oam.color;
        }

        self.pixels
            .put_pixel(x as u32, self.y as u32, self.color_to_pixel(color));
    }

    pub fn tick(&mut self) -> Result<()> {
        self.cycles += 1;

        if self.cycles >= 456 {
            self.cycles = 0;
            self.lines += 1;
            self.buffer.clear();
            self.bg_line = [0; WIDTH];
            self.oam_line = [Default::default(); WIDTH];
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
                    self.drawing_window = false;
                }
                _ => {}
            }
        }

        if self.lines == 144 {
            self.mode = Mode::VBlank;
            self.int_v_blank = true;
        }

        match self.mode {
            Mode::Drawing => {
                if self.lcd_control.bg_win_enable() {
                    if self.lcd_control.window_display_enable() {
                        self.draw_window();
                    }

                    self.draw_bg();
                }

                if self.lcd_control.sprite_enable() {
                    self.draw_sprite();
                }
            }
            Mode::HBlank if self.cycles < 400 => {
                self.put_pixels((self.cycles - 240) as u8);
            }
            Mode::OamScan => {
                if self.cycles % 2 == 0 {
                    self.scan_oam((self.cycles / 2) as usize);
                }
            }
            _ => {}
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
        let index_addr = addr - 0xFE00;
        let index = index_addr / 4;
        let offset = index_addr % 4;
        let oam = self.oam[index as usize];

        match offset {
            0 => Ok(oam.y_pos),
            1 => Ok(oam.x_pos),
            2 => Ok(oam.tile_num),
            3 => Ok(oam.sprite_flag.0),
            _ => unreachable!(),
        }
    }

    pub fn write_oam(&mut self, addr: u16, val: u8) -> Result<()> {
        let index_addr = addr - 0xFE00;
        let index = (index_addr / 4) as usize;
        let offset = index_addr % 4;

        match offset {
            0 => {
                self.oam[index].y_pos = val;
            }
            1 => {
                self.oam[index].x_pos = val;
            }
            2 => {
                self.oam[index].tile_num = val;
            }
            3 => {
                self.oam[index].sprite_flag.0 = val;
            }
            _ => unreachable!(),
        }

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
