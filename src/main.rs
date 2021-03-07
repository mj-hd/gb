use gb::gb::Gb;
use gb::rom::Rom;
use pixels::{Pixels, SurfaceTexture};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let size = LogicalSize::new(160, 144);
    let window = WindowBuilder::new()
        .with_title("gb")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap();

    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(160, 144, surface_texture).unwrap();

    let mut reader = BufReader::new(File::open("roms/cpu_instrs.gb").unwrap());
    let rom = Rom::new(&mut reader).unwrap();

    let mut gb = Gb::new(rom);

    gb.reset().unwrap();

    let mut time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::RedrawRequested(_) => {
                gb.render(pixels.get_frame()).unwrap();
                pixels.render().unwrap();
            }
            _ => {}
        }

        gb.tick().unwrap();

        match *control_flow {
            ControlFlow::Exit => {}
            _ => {
                if time.elapsed() >= Duration::from_millis(1000 / 60) {
                    time = Instant::now();

                    window.request_redraw();
                }

                if input.update(&event) {
                    if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    if let Some(size) = input.window_resized() {
                        pixels.resize(size.width, size.height);
                    }
                }

                *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_micros(100));
            }
        }
    });
}
