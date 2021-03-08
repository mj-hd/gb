use anyhow::Result;
use image::{ImageBuffer, Rgba};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

pub struct Ppu {
    vram: [u8; 8 * 1024],

    pixels: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            vram: [0; 8 * 1024],
            pixels: ImageBuffer::new(WIDTH as u32, HEIGHT as u32),
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        for (x, y, pixel) in self.pixels.enumerate_pixels_mut() {
            *pixel = Rgba([0x00, 0x00, 0x00, 0xFF]);
            if (x / 30) % 2 == 0 {
                *pixel = Rgba([0x00, 0xFF, 0x00, 0xFF]);
            }
            if (y / 30) % 2 == 0 {
                *pixel = Rgba([0x00, 0xFF, 0x00, 0xFF]);
            }
        }
        Ok(())
    }

    pub fn read(&self, addr: u16) -> Result<u8> {
        Ok(0)
    }

    pub fn write(&mut self, addr: u16, val: u8) -> Result<()> {
        Ok(())
    }

    pub fn read_oam(&self, addr: u16) -> Result<u8> {
        Ok(0)
    }

    pub fn write_oam(&mut self, addr: u16, val: u8) -> Result<()> {
        Ok(())
    }

    pub fn render(&mut self, frame: &mut [u8]) -> Result<()> {
        frame.clone_from_slice(&self.pixels.clone().into_raw());
        Ok(())
    }
}
