pub struct Debugger {
    pub breakpoints: Vec<u16>,
    pub step_run: bool,
    pub on_step: Box<dyn FnMut() -> bool>,
}

impl Debugger {
    pub fn new(breakpoints: Vec<u16>, on_step: Box<dyn FnMut() -> bool>) -> Self {
        Self {
            step_run: false,
            breakpoints,
            on_step,
        }
    }
}
