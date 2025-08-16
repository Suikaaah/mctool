use crate::io;
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

pub struct Key {
    vkeys: Box<[VIRTUAL_KEY]>,
    previous: bool,
    is_pressed: bool,
}

impl Key {
    pub fn new(vkeys: Vec<VIRTUAL_KEY>) -> Self {
        Self {
            vkeys: vkeys.into_boxed_slice(),
            previous: false,
            is_pressed: false,
        }
    }

    pub fn update(&mut self) {
        let is_down = self.vkeys.iter().all(|vkey| io::is_down(*vkey));
        self.is_pressed = is_down && !self.previous;
        self.previous = is_down;
    }

    pub const fn is_pressed(&self) -> bool {
        self.is_pressed
    }
}
