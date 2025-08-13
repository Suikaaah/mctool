mod grid;
mod key;
mod spam;

use key::Key;
use spam::Spam;
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::{self as kam, VIRTUAL_KEY};

pub struct State {
    pub is_modified: bool,
    pub spam_left: Spam,
    pub spam_right: Spam,
    pub spam_space: Spam,
    key_left: Key,
    key_right: Key,
    key_space: Key,
    key_middle: Key,
}

impl State {
    const KEY_LEFT: VIRTUAL_KEY = kam::VK_Z;
    const KEY_RIGHT: VIRTUAL_KEY = kam::VK_X;
    const KEY_SPACE: VIRTUAL_KEY = kam::VK_C;
    const KEY_MIDDLE: VIRTUAL_KEY = kam::VK_MBUTTON;
    const INT_LEFT: Duration = Duration::from_millis(100);
    const INT_RIGHT: Duration = Duration::from_millis(100);
    const INT_SPACE: Duration = Duration::from_millis(100);

    pub fn new() -> Self {
        use crate::io::{self, MouseButton};

        let spam_left = Spam::new(
            Self::INT_LEFT,
            || io::send_mouse_down(MouseButton::Left),
            || io::send_mouse_up(MouseButton::Left),
        );

        let spam_right = Spam::new(
            Self::INT_RIGHT,
            || io::send_mouse_down(MouseButton::Right),
            || io::send_mouse_up(MouseButton::Right),
        );

        let spam_space = Spam::new(
            Self::INT_SPACE,
            || io::send_key_down(kam::VK_SPACE),
            || io::send_key_up(kam::VK_SPACE),
        );

        Self {
            is_modified: false,
            spam_left,
            spam_right,
            spam_space,
            key_left: Key::new(Self::KEY_LEFT),
            key_right: Key::new(Self::KEY_RIGHT),
            key_space: Key::new(Self::KEY_SPACE),
            key_middle: Key::new(Self::KEY_MIDDLE),
        }
    }

    pub fn step(&mut self) {
        self.update();

        let now = Instant::now();

        self.spam_left.step(now);
        self.spam_right.step(now);
        self.spam_space.step(now);
    }

    fn update(&mut self) {
        self.is_modified = false;

        let mut toggle = |key: &mut Key, spam: &mut Spam| {
            key.update();

            if key.is_pressed() {
                spam.toggle_active();
                self.is_modified = true;
            }
        };

        toggle(&mut self.key_left, &mut self.spam_left);
        toggle(&mut self.key_right, &mut self.spam_right);
        toggle(&mut self.key_space, &mut self.spam_space);

        let mut update = |key: &mut Key| {
            key.update();

            if key.is_pressed() {
                self.is_modified = true;
            }
        };

        update(&mut self.key_middle);
    }
}
