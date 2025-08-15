mod key;
mod spam;

use crate::{coord::Coord, grid::Grid};
use key::Key;
use spam::Spam;
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::{self as kam, VIRTUAL_KEY};

pub struct State {
    draw_required: bool,
    key_left: Key,
    key_right: Key,
    key_space: Key,
    key_record: Key,
    key_play: Key,
    key_click: Key,
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
    _Playing {
        clicks: Vec<Option<Grid>>,
        origin: Instant,
    },
}

impl State {
    const KEY_LEFT: VIRTUAL_KEY = kam::VK_Z;
    const KEY_RIGHT: VIRTUAL_KEY = kam::VK_X;
    const KEY_SPACE: VIRTUAL_KEY = kam::VK_C;
    const KEY_RECORD: VIRTUAL_KEY = kam::VK_B;
    const KEY_PLAY: VIRTUAL_KEY = kam::VK_G;
    const KEY_CLICK: VIRTUAL_KEY = kam::VK_LBUTTON;
    const INT_LEFT: Duration = Duration::from_millis(10);
    const INT_RIGHT: Duration = Duration::from_millis(10);
    const INT_SPACE: Duration = Duration::from_millis(50);
    const SCREENSHOTS: &str =
        r"C:\Users\Suika\AppData\Roaming\.minecraft\versions\1.8.9-OptiFine_HD_U_M5\screenshots";
    const RECIPES: &str = "recipes";

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
            draw_required: false,
            key_left: Key::new(Self::KEY_LEFT),
            key_right: Key::new(Self::KEY_RIGHT),
            key_space: Key::new(Self::KEY_SPACE),
            key_record: Key::new(Self::KEY_RECORD),
            key_play: Key::new(Self::KEY_PLAY),
            key_click: Key::new(Self::KEY_CLICK),
            spam_left,
            spam_right,
            spam_space,
            detail: Detail::Idle,
        }
    }

    pub const fn draw_required(&self) -> bool {
        self.draw_required
    }

    pub const fn detail(&self) -> &Detail {
        &self.detail
    }

    pub fn step(&mut self) {
        self.update_keys();
        self.toggle_spams();

        let now = Instant::now();

        self.spam_left.step(now);
        self.spam_right.step(now);
        self.spam_space.step(now);

        match &mut self.detail {
            Detail::Idle => {
                if self.key_record.is_pressed() {
                    self.detail = Detail::Recording { clicks: Vec::new() };
                }
            }
            Detail::Recording { clicks } => {
                if self.key_click.is_pressed() {
                    let coord = Coord::from(crate::io::get_cursor());

                    match coord.try_into() {
                        Ok(grid) => clicks.push(grid),
                        Err(e) => println!("{e}"),
                    }
                }

                if self.key_record.is_pressed() {
                    crate::io::save_clicks(Self::SCREENSHOTS, Self::RECIPES, clicks);
                    self.detail = Detail::Idle;
                }
            }
            Detail::_Playing { .. } => (),
        }
    }

    fn update_keys(&mut self) {
        self.draw_required = false;

        let mut update = |key: &mut Key| {
            key.update();

            if key.is_pressed() {
                self.draw_required = true;
            }
        };

        update(&mut self.key_left);
        update(&mut self.key_right);
        update(&mut self.key_space);
        update(&mut self.key_record);
        update(&mut self.key_play);

        // does not count as a modification which needs redraw
        self.key_click.update();
    }

    fn toggle_spams(&mut self) {
        let toggle = |key: &Key, spam: &mut Spam| {
            if key.is_pressed() {
                spam.toggle_active();
            }
        };

        toggle(&self.key_left, &mut self.spam_left);
        toggle(&self.key_right, &mut self.spam_right);
        toggle(&self.key_space, &mut self.spam_space);
    }
}
