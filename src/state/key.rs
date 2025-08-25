use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

pub struct Key {
    vkeys: Box<[VIRTUAL_KEY]>,
    previous: bool,
    is_pressed: bool,
    is_released: bool,
}

impl Key {
    pub fn single(vkey: VIRTUAL_KEY) -> Self {
        Self {
            vkeys: Box::new([vkey]),
            previous: false,
            is_pressed: false,
            is_released: false,
        }
    }

    pub fn multiple(vkeys: impl Into<Box<[VIRTUAL_KEY]>>) -> Self {
        Self {
            vkeys: vkeys.into(),
            previous: false,
            is_pressed: false,
            is_released: false,
        }
    }

    pub fn update(&mut self, is_disabled: bool) {
        let is_down = !is_disabled && self.vkeys.iter().all(|vkey| crate::io::is_down(*vkey));

        self.is_pressed = is_down && !self.previous;
        self.is_released = !is_down && self.previous;
        self.previous = is_down;
    }

    pub const fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    pub const fn is_released(&self) -> bool {
        self.is_released
    }
}
