use crate::io;
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

pub struct Key {
    vkey: VIRTUAL_KEY,
    previous: bool,
    is_pressed: bool,
}

impl Key {
    pub const fn new(vkey: VIRTUAL_KEY) -> Self {
        Self {
            vkey,
            previous: false,
            is_pressed: false,
        }
    }

    pub fn update(&mut self) {
        let is_down = io::is_down(self.vkey);
        self.is_pressed = is_down && !self.previous;
        self.previous = is_down;
    }

    pub const fn is_pressed(&self) -> bool {
        self.is_pressed
    }
}
