mod grid;
mod key;
mod spam;

use grid::Grid;
use key::Key;
use spam::Spam;
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::{self as kam, VIRTUAL_KEY};

pub struct State {
    is_modified: bool,
    key_left: Key,
    key_right: Key,
    key_space: Key,
    key_middle: Key,
    pub spam_left: Spam,
    pub spam_right: Spam,
    pub spam_space: Spam,
    detail: Detail,
}

pub enum Detail {
    Idle,
    Recording {
        clicks: Vec<Grid>,
    },
    Playing {
        clicks: Vec<Option<Grid>>,
        origin: Instant,
    },
}

impl State {
    const KEY_LEFT: VIRTUAL_KEY = kam::VK_Z;
    const KEY_RIGHT: VIRTUAL_KEY = kam::VK_X;
    const KEY_SPACE: VIRTUAL_KEY = kam::VK_C;
    const KEY_MIDDLE: VIRTUAL_KEY = kam::VK_MBUTTON;
    const INT_LEFT: Duration = Duration::from_millis(10);
    const INT_RIGHT: Duration = Duration::from_millis(10);
    const INT_SPACE: Duration = Duration::from_millis(50);

    pub fn new() -> Self {
        use crate::io::{self, MouseButton};

        let spam_left = Spam::new(
            State::INT_LEFT,
            || io::send_mouse_down(MouseButton::Left),
            || io::send_mouse_up(MouseButton::Left),
        );

        let spam_right = Spam::new(
            State::INT_RIGHT,
            || io::send_mouse_down(MouseButton::Right),
            || io::send_mouse_up(MouseButton::Right),
        );

        let spam_space = Spam::new(
            State::INT_SPACE,
            || io::send_key_down(kam::VK_SPACE),
            || io::send_key_up(kam::VK_SPACE),
        );

        Self {
            is_modified: false,
            key_left: Key::new(Self::KEY_LEFT),
            key_right: Key::new(Self::KEY_RIGHT),
            key_space: Key::new(Self::KEY_SPACE),
            key_middle: Key::new(Self::KEY_MIDDLE),
            spam_left,
            spam_right,
            spam_space,
            detail: Detail::Idle,
        }
    }

    pub const fn is_modified(&self) -> bool {
        self.is_modified
    }

    pub const fn detail(&self) -> &Detail {
        &self.detail
    }

    pub fn step(&mut self) {
        self.update_keys();

        let toggle = |key: &Key, spam: &mut Spam| {
            if key.is_pressed() {
                spam.toggle_active()
            }
        };

        toggle(&self.key_left, &mut self.spam_left);
        toggle(&self.key_right, &mut self.spam_right);
        toggle(&self.key_space, &mut self.spam_space);

        let now = Instant::now();

        self.spam_left.step(now);
        self.spam_right.step(now);
        self.spam_space.step(now);

        match &mut self.detail {
            Detail::Idle => {
                if self.key_middle.is_pressed() {
                    self.detail = Detail::Recording { clicks: Vec::new() }
                }
            }
            Detail::Recording { .. } => {
                if self.key_middle.is_pressed() {
                    self.detail = Detail::Idle
                }
            }
            Detail::Playing { .. } => (),
        }
    }

    fn update_keys(&mut self) {
        self.is_modified = false;

        let mut update = |key: &mut Key| {
            key.update();

            if key.is_pressed() {
                self.is_modified = true;
            }
        };

        update(&mut self.key_left);
        update(&mut self.key_right);
        update(&mut self.key_space);
        update(&mut self.key_middle);
    }
}
