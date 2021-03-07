use anyhow::Result;
use gl::types::*;
use image::{ImageBuffer, Rgb};

const WIDTH: i32 = 160;
const HEIGHT: i32 = 144;

pub struct Ppu {
    vram: [u8; 8 * 1024],

    pixels: ImageBuffer<Rgb<u8>, Vec<u8>>,
    texture: GLuint,
    framebuffer: GLuint,
}

impl Ppu {
    pub fn new() -> Self {
        let mut texture: GLuint = 0;
        let mut framebuffer: GLuint = 0;

        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                WIDTH,
                HEIGHT,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                std::ptr::null_mut(),
            );

            gl::GenFramebuffers(1, &mut framebuffer);

            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);

            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture,
                0,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Ppu {
            vram: [0; 8 * 1024],
            pixels: ImageBuffer::new(WIDTH as u32, HEIGHT as u32),
            texture,
            framebuffer,
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        for (x, y, pixel) in self.pixels.enumerate_pixels_mut() {
            *pixel = Rgb([0x00, 0x00, 0x00]);
            if (x / 30) % 2 == 0 {
                *pixel = Rgb([0x00, 0xFF, 0x00]);
            }
            if (y / 30) % 2 == 0 {
                *pixel = Rgb([0x00, 0xFF, 0x00]);
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

    pub fn render(&mut self) -> Result<()> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                self.pixels.width() as i32,
                self.pixels.height() as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                self.pixels.clone().into_raw().as_ptr() as *const std::ffi::c_void,
            );

            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.framebuffer);

            gl::BlitFramebuffer(
                0,
                0,
                self.pixels.width() as i32,
                self.pixels.height() as i32,
                0,
                0,
                160 * 2,
                144 * 2,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );

            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Ok(())
    }
}
