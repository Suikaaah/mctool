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
    key_right_click: Key,
    key_prev: Key,
    key_next: Key,
    key_double_click: Key,
    key_begin_trade: Key,
    key_end_trade: Key,
    key_abort: Key,
    pub spam_left: Spam,
    pub spam_right: Spam,
    pub spam_space: Spam,
    detail: Detail,
    pub recipes: Recipes,
    double_click_active: bool,
    double_click_origin: Option<Instant>,
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
    TradingFirst {
        state: TradeFirst,
        position: (i32, i32),
        origin: Instant,
    },
    TradingSecond {
        state: TradeSecond,
        position: (i32, i32),
        origin: Instant,
    },
}

pub enum Click {
    New,
    Moved,
    Clicked,
}

pub enum TradeFirst {
    InvClicked,
    MovedToLeft,
    LeftClicked,
    Waiting,
}

pub enum TradeSecond {
    RightClicked,
    MovedToLeft,
    LeftClicked,
}

impl State {
    const KEY_LEFT: VIRTUAL_KEY = kam::VK_Z;
    const KEY_RIGHT: VIRTUAL_KEY = kam::VK_X;
    const KEY_SPACE: VIRTUAL_KEY = kam::VK_C;
    const KEY_RECORD: VIRTUAL_KEY = kam::VK_B;
    const KEY_PLAY: VIRTUAL_KEY = kam::VK_G;
    const KEY_CLICK: VIRTUAL_KEY = kam::VK_LBUTTON;
    const KEY_RIGHT_CLICK: VIRTUAL_KEY = kam::VK_RBUTTON;
    const KEY_PREV: VIRTUAL_KEY = kam::VK_XBUTTON1;
    const KEY_NEXT: VIRTUAL_KEY = kam::VK_XBUTTON2;
    const KEY_DOUBLE_CLICK: VIRTUAL_KEY = kam::VK_TAB;
    const KEY_BEGIN_TRADE: VIRTUAL_KEY = kam::VK_R;
    const KEY_END_TRADE: VIRTUAL_KEY = kam::VK_LSHIFT;
    const KEY_ABORT: VIRTUAL_KEY = kam::VK_OEM_3;
    const INT_LEFT: Duration = Duration::from_millis(10);
    const INT_RIGHT: Duration = Duration::from_millis(10);
    const INT_SPACE: Duration = Duration::from_millis(50);
    const INT_PLAY: Duration = Duration::from_millis(7);
    const INT_DOUBLE_CLICK: Duration = Duration::from_millis(50);
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
            key_right_click: Key::new(Self::KEY_RIGHT_CLICK),
            key_prev: Key::new(Self::KEY_PREV),
            key_next: Key::new(Self::KEY_NEXT),
            key_double_click: Key::new(Self::KEY_DOUBLE_CLICK),
            key_begin_trade: Key::new(Self::KEY_BEGIN_TRADE),
            key_end_trade: Key::new(Self::KEY_END_TRADE),
            key_abort: Key::new(Self::KEY_ABORT),
            spam_left,
            spam_right,
            spam_space,
            detail: Detail::Idle,
            recipes: Recipes::new(io::recipes(Self::RECIPES)),
            double_click_active: false,
            double_click_origin: None,
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

        if self.key_abort.is_pressed() {
            self.detail = Detail::Idle;
        }

        if self.key_double_click.is_pressed() {
            self.double_click_active ^= true;
        }

        if self.double_click_active && self.key_right_click.is_pressed() {
            self.double_click_origin = Some(now);
        }

        if let Some(instant) = self.double_click_origin
            && Self::INT_DOUBLE_CLICK <= instant.elapsed()
        {
            io::send_mouse(io::MouseButton::Right);
            self.double_click_origin = None;
        }

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

                if self.key_begin_trade.is_pressed() {
                    io::send_mouse(io::MouseButton::Left);

                    self.detail = Detail::TradingFirst {
                        state: TradeFirst::InvClicked,
                        position: io::get_cursor(),
                        origin: now,
                    };
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
                        origin: now,
                    };
                }
            }
            Detail::Recording { clicks } => {
                if self.key_click.is_pressed() {
                    let coord = Coord::from(io::get_cursor());

                    if let Ok(grid) = coord.try_into() {
                        clicks.push(grid);
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
                                grid.set_cursor();
                                *click = Click::Moved;
                            }
                        }
                        (_, click @ Click::Moved) => {
                            if index & 1 == 1 {
                                io::send_mouse(io::MouseButton::Left);
                                *click = Click::Clicked;
                            }
                        }
                        (_, Click::Clicked) => (),
                    },
                }
            }
            Detail::TradingFirst {
                state,
                position,
                origin,
            } => {
                let index =
                    (origin.elapsed().as_secs_f64() / Self::INT_PLAY.as_secs_f64()) as usize;

                match state {
                    TradeFirst::InvClicked => {
                        if 0 < index {
                            io::set_cursor(828, 474);
                            *state = TradeFirst::MovedToLeft;
                        }
                    }
                    TradeFirst::MovedToLeft => {
                        if 1 < index {
                            io::send_mouse(io::MouseButton::Left);
                            *state = TradeFirst::LeftClicked;
                        }
                    }
                    TradeFirst::LeftClicked => {
                        if 2 < index {
                            io::set_cursor(1080, 474);
                            *state = TradeFirst::Waiting;
                        }
                    }
                    TradeFirst::Waiting => {
                        if self.key_end_trade.is_pressed() {
                            io::send_mouse(io::MouseButton::Left);

                            self.detail = Detail::TradingSecond {
                                state: TradeSecond::RightClicked,
                                position: *position,
                                origin: now,
                            };
                        }
                    }
                }
            }
            Detail::TradingSecond {
                state,
                position: (x, y),
                origin,
            } => {
                let index =
                    (origin.elapsed().as_secs_f64() / Self::INT_PLAY.as_secs_f64()) as usize;

                match state {
                    TradeSecond::RightClicked => {
                        if 0 < index {
                            io::set_cursor(828, 474);
                            *state = TradeSecond::MovedToLeft;
                        }
                    }
                    TradeSecond::MovedToLeft => {
                        if 1 < index {
                            io::send_mouse(io::MouseButton::Left);
                            *state = TradeSecond::LeftClicked;
                        }
                    }
                    TradeSecond::LeftClicked => {
                        if 2 < index {
                            io::set_cursor(*x, *y);
                            self.draw_required = true;
                            self.detail = Detail::Idle;
                        }
                    }
                }
            }
        }
    }

    pub const fn double_click_active(&self) -> bool {
        self.double_click_active
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
        update(&mut self.key_double_click);
        update(&mut self.key_begin_trade);
        update(&mut self.key_end_trade);
        update(&mut self.key_abort);

        // does not count as a modification which needs redraw
        self.key_click.update();
        self.key_right_click.update();
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
