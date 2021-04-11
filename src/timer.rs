use bitmatch::bitmatch;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(FromPrimitive, Debug, Copy, Clone)]
enum Clock {
    Clock4096 = 0b00,
    Clock262144 = 0b01,
    Clock65536 = 0b10,
    Clock16384 = 0b11,
}

#[derive(Debug)]
pub struct Timer {
    counter: u16,
    tima: u8,
    tma: u8,
    enable: bool,
    clock: Clock,
    prev: bool,
    pub int: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            counter: 0,
            tima: 0,
            tma: 0,
            enable: false,
            clock: Clock::Clock4096,
            int: false,
            prev: false,
        }
    }
}

impl Timer {
    fn sync(&mut self) {
        let mut cur = false;

        if self.enable {
            let bit = match self.clock {
                Clock::Clock4096 => 1 << 9,
                Clock::Clock262144 => 1 << 3,
                Clock::Clock65536 => 1 << 5,
                Clock::Clock16384 => 1 << 7,
            };

            cur = self.counter & bit > 0;
        }

        if self.prev && !cur {
            self.tima = self.tima.wrapping_add(1);

            if self.counter % 4 == 0 && self.tima == 0 {
                self.tima = self.tma;
                self.int = true;
                // println!("{:?}", self);
            }
        }

        self.prev = cur;
    }

    pub fn tick(&mut self) {
        self.counter = self.counter.wrapping_add(1);

        self.sync();

        // println!("{:?}", self);
    }

    pub fn read_div(&self) -> u8 {
        (self.counter >> 8) as u8
    }

    pub fn write_div(&mut self, _val: u8) {
        self.counter = 0;
    }

    pub fn read_tima(&self) -> u8 {
        self.tima
    }

    pub fn write_tima(&mut self, val: u8) {
        self.sync();

        self.tima = val;
    }

    pub fn read_tma(&self) -> u8 {
        self.tma
    }

    pub fn write_tma(&mut self, val: u8) {
        self.tma = val;

        self.sync();
    }

    #[bitmatch]
    pub fn read_tac(&self) -> u8 {
        let e = self.enable;
        let s = self.clock;

        bitpack!("00000ess")
    }

    #[bitmatch]
    pub fn write_tac(&mut self, val: u8) {
        #[bitmatch]
        let "?????ess" = val;

        self.enable = e == 1;

        if let Some(clock) = FromPrimitive::from_u8(s) {
            self.clock = clock;
        } else {
            eprintln!("unknown clock {}", s);
        }
    }
}
