use gb::debugger::Debugger;
use gb::gb::Gb;
use gb::rom::Rom;
use pixels::{Pixels, SurfaceTexture};
use rustyline::Editor;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use std::u16;
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

    let mut rl = Editor::<()>::new();

    let mut rom = "".to_string();
    let mut breakpoints = Vec::new();

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) if line.starts_with("load ") => {
                if let Some(path) = line.split_ascii_whitespace().nth(1) {
                    rom = path.to_string();
                    println!("rom: {}", rom);
                    continue;
                }

                println!("load command parse failed");
            }
            Ok(line) if line.starts_with("break ") => {
                if let Some(addr_str) = line.split_ascii_whitespace().nth(1) {
                    if let Ok(addr) = u16::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                        breakpoints.push(addr);

                        println!("add breakpoint: {:04X}", addr);
                        continue;
                    }
                }

                println!("break command parse failed");
            }
            Ok(line) if line.starts_with("run") => {
                break;
            }
            Ok(line) => {
                println!("unknown command {}", line);
            }
            Err(_) => {
                println!("aborted");
                std::process::exit(0);
            }
        }
    }

    let mut reader = BufReader::new(File::open(rom).unwrap());
    let rom = Rom::new(&mut reader).unwrap();

    println!("rom loaded {:?}", rom);

    let debugger = Debugger::new(
        breakpoints,
        Box::new(move || loop {
            let readline = rl.readline(">>> ");

            match readline {
                Ok(line) if line.starts_with("continue") => {
                    return false;
                }
                Ok(line) if line.starts_with("step") => {
                    return true;
                }
                Ok(line) => {
                    println!("unknown command {}", line);
                }
                Err(_) => {
                    println!("aborted");
                    std::process::exit(0);
                }
            }
        }),
    );

    let mut gb = Gb::new(rom, debugger);

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

                // *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_micros(1));
                *control_flow = ControlFlow::Poll;
            }
        }
    });
}
