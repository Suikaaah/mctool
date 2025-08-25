pub mod detail;
mod key;
mod recipes;
pub mod spam;

use crate::{
    coord::Coord,
    grid::Grid,
    io,
    state::{
        detail::{Cursor, Detail, TradeFirst, TradeSecond},
        recipes::Recipes,
    },
};
use anyhow::Result;
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
    key_lock: Key,
    key_cancel_dc: Key,
    key_confirm: Key,
    key_prev_skip: Key,
    key_next_skip: Key,
    pub spam_left: Spam,
    pub spam_right: Spam,
    pub spam_space: Spam,
    detail: Detail,
    pub recipes: Recipes,
    double_click_active: bool,
    double_click_origin: Option<Instant>,
    is_locked: bool,
}

impl State {
    const KEY_LEFT: VIRTUAL_KEY = kam::VK_Z;
    const KEY_RIGHT: VIRTUAL_KEY = kam::VK_X;
    const KEY_SPACE: VIRTUAL_KEY = kam::VK_C;
    const KEY_RECORD: VIRTUAL_KEY = kam::VK_B;
    const KEY_PLAY: VIRTUAL_KEY = kam::VK_G;
    const KEY_CLICK: VIRTUAL_KEY = kam::VK_LBUTTON;
    const KEY_RIGHT_CLICK: VIRTUAL_KEY = kam::VK_RBUTTON;
    const KEY_PREV: VIRTUAL_KEY = kam::VK_XBUTTON2;
    const KEY_NEXT: VIRTUAL_KEY = kam::VK_XBUTTON1;
    const KEY_DOUBLE_CLICK: VIRTUAL_KEY = kam::VK_TAB;
    const KEY_BEGIN_TRADE: VIRTUAL_KEY = kam::VK_R;
    const KEY_END_TRADE: VIRTUAL_KEY = kam::VK_LSHIFT;
    const KEY_ABORT: VIRTUAL_KEY = kam::VK_OEM_3;
    const KEYS_LOCK: &[VIRTUAL_KEY] = &[kam::VK_LCONTROL, kam::VK_MBUTTON];
    const KEY_CANCEL_DC: VIRTUAL_KEY = kam::VK_LCONTROL;
    const KEY_CONFIRM: VIRTUAL_KEY = kam::VK_RETURN;
    const KEYS_PREV_SKIP: &[VIRTUAL_KEY] = &[kam::VK_LCONTROL, Self::KEY_PREV];
    const KEYS_NEXT_SKIP: &[VIRTUAL_KEY] = &[kam::VK_LCONTROL, Self::KEY_NEXT];
    const INT_LEFT: Duration = Duration::from_millis(10);
    const INT_RIGHT: Duration = Duration::from_millis(10);
    const INT_SPACE: Duration = Duration::from_millis(50);
    const INT_PLAY: Duration = Duration::from_millis(7);
    const INT_DOUBLE_CLICK: Duration = Duration::from_millis(50);
    const SCREENSHOTS: &str =
        r"C:\Users\Suika\AppData\Roaming\.minecraft\versions\1.8.9-OptiFine_HD_U_M5\screenshots";
    const RECIPES: &str = r"D:\rust\mctool\recipes";

    pub fn new() -> Result<Self> {
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

        Ok(Self {
            draw_required: false,
            key_left: Key::single(Self::KEY_LEFT),
            key_right: Key::single(Self::KEY_RIGHT),
            key_space: Key::single(Self::KEY_SPACE),
            key_record: Key::single(Self::KEY_RECORD),
            key_play: Key::single(Self::KEY_PLAY),
            key_click: Key::single(Self::KEY_CLICK),
            key_right_click: Key::single(Self::KEY_RIGHT_CLICK),
            key_prev: Key::single(Self::KEY_PREV),
            key_next: Key::single(Self::KEY_NEXT),
            key_double_click: Key::single(Self::KEY_DOUBLE_CLICK),
            key_begin_trade: Key::single(Self::KEY_BEGIN_TRADE),
            key_end_trade: Key::single(Self::KEY_END_TRADE),
            key_abort: Key::single(Self::KEY_ABORT),
            key_lock: Key::multiple(Self::KEYS_LOCK),
            key_cancel_dc: Key::single(Self::KEY_CANCEL_DC),
            key_confirm: Key::single(Self::KEY_CONFIRM),
            key_prev_skip: Key::multiple(Self::KEYS_PREV_SKIP),
            key_next_skip: Key::multiple(Self::KEYS_NEXT_SKIP),
            spam_left,
            spam_right,
            spam_space,
            detail: Detail::Idle,
            recipes: Recipes::new(io::recipes(Self::RECIPES)?)?,
            double_click_active: false,
            double_click_origin: None,
            is_locked: false,
        })
    }

    pub const fn draw_required(&self) -> bool {
        self.draw_required
    }

    pub const fn detail(&self) -> &Detail {
        &self.detail
    }

    pub const fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub fn double_click_active_mapped(&self) -> bool {
        self.double_click_active && !self.double_click_disable_condition()
    }

    pub fn double_click_temporarily_disabled(&self) -> bool {
        self.double_click_active && self.double_click_disable_condition()
    }

    pub fn push_text(&mut self, text: &str) {
        if let Detail::Naming {
            name,
            draw_required,
            ..
        } = &mut self.detail
        {
            *name += text;
            *draw_required = true;
        }
    }

    pub fn pop_text(&mut self) {
        if let Detail::Naming {
            name,
            draw_required,
            ..
        } = &mut self.detail
        {
            name.pop();
            *draw_required = true;
        }
    }

    pub fn step(&mut self) -> Result<()> {
        self.draw_required = false;
        self.update_keys();
        self.toggle_spams();

        if self.key_lock.is_pressed() {
            self.is_locked ^= true;
        }

        if self.key_abort.is_pressed() {
            if matches!(self.detail, Detail::Naming { .. }) {
                self.is_locked = false;
            }

            self.detail = Detail::Idle;
        }

        if self.key_double_click.is_pressed() {
            self.double_click_active ^= true;
        }

        if self.key_cancel_dc.is_released() {
            self.draw_required = true;
        }

        if self.double_click_active_mapped() && self.key_right_click.is_pressed() {
            self.double_click_origin = Some(Instant::now());
        }

        if let Some(instant) = self.double_click_origin
            && Self::INT_DOUBLE_CLICK <= instant.elapsed()
        {
            io::send_mouse(io::MouseButton::Right);
            self.double_click_origin = None;
        }

        {
            let now = Instant::now();

            self.spam_left.step(now);
            self.spam_right.step(now);
            self.spam_space.step(now);
        }

        let detail_taken = std::mem::replace(&mut self.detail, Detail::Idle);

        self.detail = match detail_taken {
            Detail::Idle => self.on_idle(),
            Detail::Recording { clicks } => self.on_record(clicks),
            Detail::Naming {
                clicks,
                name,
                draw_required,
            } => {
                if draw_required {
                    self.draw_required = true;
                }

                self.on_name(clicks, name)
            }
            Detail::Playing { clicks, origin } => self.on_play(clicks, origin),
            Detail::TradingFirst {
                state,
                position,
                origin,
            } => self.on_trade_first(state, position, origin),
            Detail::TradingSecond {
                state,
                position,
                origin,
            } => self.on_trade_second(state, position, origin),
        }?;

        Ok(())
    }

    fn on_idle(&mut self) -> Result<Detail> {
        if self.key_prev.is_pressed() {
            if self.key_prev_skip.is_pressed() {
                self.recipes.decrement_skip();
            } else {
                self.recipes.decrement();
            }
        }

        if self.key_next.is_pressed() {
            if self.key_next_skip.is_pressed() {
                self.recipes.increment_skip();
            } else {
                self.recipes.increment();
            }
        }

        let retval = if self.key_begin_trade.is_pressed() {
            io::send_mouse(io::MouseButton::Left);

            Detail::TradingFirst {
                state: TradeFirst::InvClicked,
                position: io::get_cursor()?,
                origin: Instant::now(),
            }
        } else if self.key_record.is_pressed() {
            Detail::Recording { clicks: Vec::new() }
        } else if self.key_play.is_pressed()
            && let Some(path) = self.recipes.get_path()?
        {
            Detail::Playing {
                clicks: io::load_clicks(path.join(io::FILENAME_CLICKS))?
                    .into_iter()
                    .map(|grid| (grid, Cursor::New))
                    .collect(),
                origin: Instant::now(),
            }
        } else {
            Detail::Idle
        };

        Ok(retval)
    }

    fn on_record(&mut self, mut clicks: Vec<Grid>) -> Result<Detail> {
        if self.key_click.is_pressed() {
            let coord = Coord::from(io::get_cursor()?);

            if let Ok(grid) = coord.try_into() {
                clicks.push(grid);
            }
        }

        let retval = if self.key_record.is_pressed() {
            self.is_locked = true;

            Detail::Naming {
                clicks,
                name: String::new(),
                draw_required: false,
            }
        } else {
            Detail::Recording { clicks }
        };

        Ok(retval)
    }

    fn on_name(&mut self, clicks: Vec<Grid>, name: String) -> Result<Detail> {
        let retval = if self.key_confirm.is_pressed() {
            match io::save_clicks(Self::SCREENSHOTS, Self::RECIPES, &clicks, &name) {
                Err(e) => {
                    io::message_box(format!("Reason: {e}"), "Failed to crate recipe")?;

                    Detail::Naming {
                        clicks,
                        name,
                        draw_required: false,
                    }
                }
                Ok(_) => {
                    self.recipes = Recipes::new(io::recipes(Self::RECIPES)?)?;
                    self.is_locked = false;

                    Detail::Idle
                }
            }
        } else {
            Detail::Naming {
                clicks,
                name,
                draw_required: false,
            }
        };

        Ok(retval)
    }

    fn on_play(&mut self, mut clicks: Box<[(Grid, Cursor)]>, origin: Instant) -> Result<Detail> {
        let index = (origin.elapsed().as_secs_f64() / Self::INT_PLAY.as_secs_f64()) as usize;

        let retval = match clicks.get_mut(index / 2) {
            None => {
                self.draw_required = true;
                Detail::Idle
            }
            Some((grid, cursor)) => {
                match cursor {
                    Cursor::New if index & 1 == 0 => {
                        grid.set_cursor()?;
                        *cursor = Cursor::Moved;
                    }
                    Cursor::Moved if index & 1 == 1 => {
                        io::send_mouse(io::MouseButton::Left);
                        *cursor = Cursor::Clicked;
                    }
                    Cursor::New | Cursor::Moved | Cursor::Clicked => (),
                }

                Detail::Playing { clicks, origin }
            }
        };

        Ok(retval)
    }

    fn on_trade_first(
        &mut self,
        state: TradeFirst,
        position: (i32, i32),
        origin: Instant,
    ) -> Result<Detail> {
        let index = (origin.elapsed().as_secs_f64() / Self::INT_PLAY.as_secs_f64()) as usize;

        let f = |state| Detail::TradingFirst {
            state,
            position,
            origin,
        };

        let retval = match state {
            TradeFirst::InvClicked if 0 < index => {
                io::set_cursor(828, 474)?;
                f(TradeFirst::MovedToLeft)
            }
            TradeFirst::MovedToLeft if 1 < index => {
                io::send_mouse(io::MouseButton::Left);
                f(TradeFirst::LeftClicked)
            }
            TradeFirst::LeftClicked if 2 < index => {
                io::set_cursor(1080, 474)?;
                f(TradeFirst::Waiting)
            }
            TradeFirst::Waiting if self.key_end_trade.is_pressed() => {
                io::send_mouse(io::MouseButton::Left);

                Detail::TradingSecond {
                    state: TradeSecond::RightClicked,
                    position,
                    origin: Instant::now(),
                }
            }
            TradeFirst::InvClicked
            | TradeFirst::MovedToLeft
            | TradeFirst::LeftClicked
            | TradeFirst::Waiting => f(state),
        };

        Ok(retval)
    }

    fn on_trade_second(
        &mut self,
        state: TradeSecond,
        position: (i32, i32),
        origin: Instant,
    ) -> Result<Detail> {
        let index = (origin.elapsed().as_secs_f64() / Self::INT_PLAY.as_secs_f64()) as usize;

        let f = |state| Detail::TradingSecond {
            state,
            position,
            origin,
        };

        let retval = match state {
            TradeSecond::RightClicked if 0 < index => {
                io::set_cursor(828, 474)?;
                f(TradeSecond::MovedToLeft)
            }
            TradeSecond::MovedToLeft if 1 < index => {
                io::send_mouse(io::MouseButton::Left);
                f(TradeSecond::LeftClicked)
            }
            TradeSecond::LeftClicked if 2 < index => {
                io::set_cursor(position.0, position.1)?;
                self.draw_required = true;
                Detail::Idle
            }
            TradeSecond::RightClicked | TradeSecond::MovedToLeft | TradeSecond::LeftClicked => {
                f(state)
            }
        };

        Ok(retval)
    }

    fn update_keys(&mut self) {
        // does not count as a modification which needs redraw
        self.key_click.update(false);
        self.key_right_click.update(false);

        let mut update_nolock = |key: &mut Key| {
            key.update(false);

            if key.is_pressed() {
                self.draw_required = true;
            }
        };

        update_nolock(&mut self.key_abort);
        update_nolock(&mut self.key_lock);
        update_nolock(&mut self.key_cancel_dc);
        update_nolock(&mut self.key_confirm);

        let is_locked = self.is_locked();
        let mut update = |key: &mut Key| {
            key.update(is_locked);

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
        update(&mut self.key_prev_skip);
        update(&mut self.key_next_skip);
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

    fn double_click_disable_condition(&self) -> bool {
        io::is_down(Self::KEY_CANCEL_DC) || self.spam_right.is_active()
    }
}
