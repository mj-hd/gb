use bitmatch::bitmatch;

#[derive(Debug, Copy, Clone)]
pub enum JoypadKey {
    A,
    B,
    Up,
    Down,
    Left,
    Right,
    Select,
    Start,
}

#[derive(Debug)]
pub struct Joypad {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    a: bool,
    b: bool,
    start: bool,
    select: bool,

    direction: bool,
    button: bool,

    pub int: bool,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            a: false,
            b: false,
            start: false,
            select: false,

            direction: false,
            button: false,

            int: false,
        }
    }
}

impl Joypad {
    pub fn press(&mut self, key: JoypadKey) {
        match key {
            JoypadKey::A => {
                self.a = true;
            }
            JoypadKey::B => {
                self.b = true;
            }
            JoypadKey::Select => {
                self.select = true;
            }
            JoypadKey::Start => {
                self.start = true;
            }
            JoypadKey::Up => {
                self.up = true;
            }
            JoypadKey::Down => {
                self.down = true;
            }
            JoypadKey::Right => {
                self.right = true;
            }
            JoypadKey::Left => {
                self.left = true;
            }
        }

        self.int = true;
    }

    pub fn release(&mut self, key: JoypadKey) {
        match key {
            JoypadKey::A => {
                self.a = false;
            }
            JoypadKey::B => {
                self.b = false;
            }
            JoypadKey::Select => {
                self.select = false;
            }
            JoypadKey::Start => {
                self.start = false;
            }
            JoypadKey::Up => {
                self.up = false;
            }
            JoypadKey::Down => {
                self.down = false;
            }
            JoypadKey::Right => {
                self.right = false;
            }
            JoypadKey::Left => {
                self.left = false;
            }
        }
    }

    #[bitmatch]
    #[allow(clippy::many_single_char_names)]
    pub fn read_button(&self) -> u8 {
        let d = !self.direction;
        let s = !self.start;
        let e = !self.select;
        let b = !self.b;
        let a = !self.a;

        bitpack!("110dseba")
    }

    #[bitmatch]
    #[allow(clippy::many_single_char_names)]
    pub fn read_direction(&self) -> u8 {
        let b = !self.button;
        let d = !self.down;
        let u = !self.up;
        let l = !self.left;
        let r = !self.right;

        bitpack!("11b0dulr")
    }

    pub fn read(&self) -> u8 {
        if self.direction {
            return self.read_direction();
        }

        if self.button {
            return self.read_button();
        }

        0xFF
    }

    #[bitmatch]
    pub fn write(&mut self, val: u8) {
        #[bitmatch]
        let "??bd????" = val;

        self.direction = d == 0;
        self.button = b == 0;
    }
}
