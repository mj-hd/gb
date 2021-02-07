use gb::gb::Gb;
use gb::rom::Rom;
use std::fs::File;
use std::io::BufReader;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut reader = BufReader::new(File::open("roms/cpu_instrs.gb").unwrap());
    let rom = Rom::new(&mut reader).unwrap();
    let mut gb = Gb::new(rom);

    gb.reset();

    loop {
        gb.tick().unwrap();

        // TODO 一旦100μs
        sleep(Duration::from_micros(100));
    }
}
