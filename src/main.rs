use gb::gb::Gb;
use gb::joypad::JoypadKey;
use gb::rom::Rom;
use pixels::{Pixels, SurfaceTexture};
use rustyline::Editor;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;
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

    let args = env::args().collect::<Vec<String>>();

    let mut reader = BufReader::new(File::open(args[1].clone()).unwrap());
    let rom = Rom::new(&mut reader).unwrap();

    let rl = Editor::<()>::new();

    let gb = Arc::new(Mutex::new(Gb::new(rom, rl)));

    {
        let gb = gb.clone();

        gb.lock().unwrap().reset().unwrap();

        thread::spawn(move || loop {
            let time = Instant::now();

            for _ in 0..17556 {
                gb.lock().unwrap().tick().unwrap();
            }

            let elapsed = time.elapsed().as_millis();

            let (wait, c) = ((1000 / 60) as u128).overflowing_sub(elapsed);

            if !c {
                thread::sleep(Duration::from_millis(wait as u64));
            }
        });
    }

    {
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
                    gb.lock().unwrap().render(pixels.get_frame()).unwrap();
                    pixels.render().unwrap();
                }
                _ => {}
            }

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

                        if input.key_pressed(VirtualKeyCode::B) {
                            gb.lock().unwrap().debug_break().unwrap();
                        }

                        for (input_key, joypad_key) in [
                            (VirtualKeyCode::Z, JoypadKey::A),
                            (VirtualKeyCode::X, JoypadKey::B),
                            (VirtualKeyCode::C, JoypadKey::Select),
                            (VirtualKeyCode::V, JoypadKey::Start),
                            (VirtualKeyCode::Up, JoypadKey::Up),
                            (VirtualKeyCode::Down, JoypadKey::Down),
                            (VirtualKeyCode::Left, JoypadKey::Left),
                            (VirtualKeyCode::Right, JoypadKey::Right),
                        ]
                        .iter()
                        {
                            if input.key_pressed(*input_key) {
                                gb.lock().unwrap().press(*joypad_key);
                            }
                            if input.key_released(*input_key) {
                                gb.lock().unwrap().release(*joypad_key);
                            }
                        }

                        if let Some(size) = input.window_resized() {
                            pixels.resize(size.width, size.height);
                        }
                    }

                    *control_flow = ControlFlow::Poll;
                }
            }
        });
    }
}
