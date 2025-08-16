mod key;
mod recipes;
pub mod spam;

use crate::{coord::Coord, grid::Grid, io, state::recipes::Recipes};
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
    key_prev: Key,
    key_next: Key,
    pub spam_left: Spam,
    pub spam_right: Spam,
    pub spam_space: Spam,
    detail: Detail,
    pub recipes: Recipes,
}

pub enum Detail {
    Idle,
    Recording {
        clicks: Vec<Grid>,
    },
    Playing {
        clicks: Box<[(Grid, Click)]>,
        origin: Instant,
    },
}

pub enum Click {
    New,
    Moved,
    Clicked,
}

impl State {
    const KEY_LEFT: VIRTUAL_KEY = kam::VK_Z;
    const KEY_RIGHT: VIRTUAL_KEY = kam::VK_X;
    const KEY_SPACE: VIRTUAL_KEY = kam::VK_C;
    const KEY_RECORD: VIRTUAL_KEY = kam::VK_B;
    const KEY_PLAY: VIRTUAL_KEY = kam::VK_G;
    const KEY_CLICK: VIRTUAL_KEY = kam::VK_LBUTTON;
    const KEY_PREV: VIRTUAL_KEY = kam::VK_XBUTTON1;
    const KEY_NEXT: VIRTUAL_KEY = kam::VK_XBUTTON2;
    const INT_LEFT: Duration = Duration::from_millis(10);
    const INT_RIGHT: Duration = Duration::from_millis(10);
    const INT_SPACE: Duration = Duration::from_millis(50);
    const INT_PLAY: Duration = Duration::from_millis(7);
    const SCREENSHOTS: &str =
        r"C:\Users\Suika\AppData\Roaming\.minecraft\versions\1.8.9-OptiFine_HD_U_M5\screenshots";
    const RECIPES: &str = "recipes";

    pub fn new() -> Self {
        use io::MouseButton;

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
            key_prev: Key::new(Self::KEY_PREV),
            key_next: Key::new(Self::KEY_NEXT),
            spam_left,
            spam_right,
            spam_space,
            detail: Detail::Idle,
            recipes: Recipes::new(io::recipes(Self::RECIPES)),
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
                if self.key_prev.is_pressed() {
                    self.recipes.decrement();
                }

                if self.key_next.is_pressed() {
                    self.recipes.increment();
                }

                if self.key_record.is_pressed() {
                    self.detail = Detail::Recording { clicks: Vec::new() };
                }

                if self.key_play.is_pressed()
                    && let Some(path) = self.recipes.get()
                {
                    self.detail = Detail::Playing {
                        clicks: io::load_clicks(path.join(io::FILENAME_CLICKS))
                            .into_iter()
                            .map(|grid| (grid, Click::New))
                            .collect(),
                        origin: Instant::now(),
                    };
                }
            }
            Detail::Recording { clicks } => {
                if self.key_click.is_pressed() {
                    let coord = Coord::from(io::get_cursor());

                    match coord.try_into() {
                        Ok(grid) => clicks.push(grid),
                        Err(e) => println!("{e}"),
                    }
                }

                if self.key_record.is_pressed() {
                    io::save_clicks(Self::SCREENSHOTS, Self::RECIPES, clicks);
                    self.recipes = Recipes::new(io::recipes(Self::RECIPES));
                    self.detail = Detail::Idle;
                }
            }
            Detail::Playing { clicks, origin } => {
                let index =
                    (origin.elapsed().as_secs_f64() / Self::INT_PLAY.as_secs_f64()) as usize;

                match clicks.get_mut(index / 2) {
                    None => {
                        self.detail = Detail::Idle;
                        self.draw_required = true;
                    }
                    Some(opt) => match opt {
                        (grid, click @ Click::New) => {
                            if index & 1 == 0 {
                                println!("cursor set");
                                grid.set_cursor();
                                *click = Click::Moved;
                            }
                        }
                        (_, click @ Click::Moved) => {
                            if index & 1 == 1 {
                                println!("left clicked");
                                io::send_mouse(io::MouseButton::Left);
                                *click = Click::Clicked;
                            }
                        }
                        (_, Click::Clicked) => (),
                    },
                }
            }
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
        update(&mut self.key_prev);
        update(&mut self.key_next);

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
