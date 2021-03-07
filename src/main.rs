use gb::gb::Gb;
use gb::rom::Rom;
use glfw::Context;
use std::fs::File;
use std::io::BufReader;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    let mut fw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    fw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    fw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    fw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    fw.window_hint(glfw::WindowHint::Resizable(true));

    let (mut window, events) = fw
        .create_window(160 * 2, 144 * 2, &"gb", glfw::WindowMode::Windowed)
        .unwrap();

    window.make_current();
    window.set_key_polling(true);

    unsafe {
        gl::load_with(|s| window.get_proc_address(s));

        gl::Enable(gl::TEXTURE_2D);
    }

    let mut reader = BufReader::new(File::open("roms/cpu_instrs.gb").unwrap());
    let rom = Rom::new(&mut reader).unwrap();

    let mut gb = Gb::new(rom);

    gb.reset().unwrap();

    let mut prev_time = Instant::now();

    while !window.should_close() {
        gb.tick().unwrap();

        if prev_time.elapsed() >= Duration::from_millis(1000 / 60) {
            gb.render().unwrap();
            prev_time = Instant::now();

            window.swap_buffers();
        }

        fw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        }

        // TODO 一旦100μs
        sleep(Duration::from_micros(100));
    }
}
