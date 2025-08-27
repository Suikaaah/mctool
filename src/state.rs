pub mod detail;
mod key;
mod recipes;
pub mod spam;

use crate::{
    coord::Coord,
    grid::Grid,
    io,
    resources::Resources,
    state::{
        detail::{Cursor, Detail, TradeFirst, TradeSecond},
        recipes::Recipes,
    },
};
use anyhow::Result;
use key::{Key, Keys};
use spam::Spam;
use std::{
    path::Path,
    time::{Duration, Instant},
};
use windows::Win32::UI::Input::KeyboardAndMouse as kam;

pub struct State<'resources> {
    detail: Detail,
    draw_required: bool,
    keys: Keys,
    pub spam_left: Spam,
    pub spam_right: Spam,
    pub spam_space: Spam,
    pub recipes: Recipes<'resources>,
    double_click_active: bool,
    double_click_origin: Option<Instant>,
    is_locked: bool,
}

impl<'resources> State<'resources> {
    const INT_LEFT: Duration = Duration::from_millis(10);
    const INT_RIGHT: Duration = Duration::from_millis(10);
    const INT_SPACE: Duration = Duration::from_millis(50);
    const INT_PLAY: Duration = Duration::from_millis(7);
    const INT_DOUBLE_CLICK: Duration = Duration::from_millis(50);
    const SCREENSHOTS: &'static str =
        r"C:\Users\Suika\AppData\Roaming\.minecraft\versions\1.8.9-OptiFine_HD_U_M5\screenshots";

    pub fn new(resources: &'resources Resources) -> Result<Self> {
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
            detail: Detail::Idle,
            draw_required: false,
            keys: Keys::new(),
            spam_left,
            spam_right,
            spam_space,
            recipes: Recipes::new(resources)?,
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
        }
        | Detail::Renaming {
            name,
            draw_required,
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
        }
        | Detail::Renaming {
            name,
            draw_required,
        } = &mut self.detail
        {
            name.pop();
            *draw_required = true;
        }
    }

    pub fn step(&mut self, resources: &'resources Resources) -> Result<()> {
        self.draw_required = false;
        self.update_keys();
        self.toggle_spams();
        self.on_step();

        self.detail = match std::mem::take(&mut self.detail) {
            Detail::Idle => self.on_idle(resources),
            Detail::Recording { clicks, count } => self.on_record(clicks, count),
            Detail::Naming {
                clicks,
                name,
                draw_required,
            } => {
                self.draw_required |= draw_required;
                self.on_name(clicks, name, resources)
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
            Detail::Deleting => self.on_delete(resources),
            Detail::Renaming {
                name,
                draw_required,
            } => {
                self.draw_required |= draw_required;
                self.on_rename(name, resources)
            }
        }?;

        Ok(())
    }

    fn on_step(&mut self) {
        if self.keys.lock.is_pressed() {
            self.is_locked ^= true;
        }

        if self.keys.abort.is_pressed() {
            if matches!(self.detail, Detail::Naming { .. }) {
                self.is_locked = false;
            }

            self.detail = Detail::Idle;
        }

        if self.keys.double_click.is_pressed() {
            self.double_click_active ^= true;
        }

        if self.keys.cancel_dc.is_released() {
            self.draw_required = true;
        }

        if self.double_click_active_mapped() && self.keys.right_click.is_pressed() {
            self.double_click_origin = Some(Instant::now());
        }

        if let Some(instant) = self.double_click_origin
            && Self::INT_DOUBLE_CLICK <= instant.elapsed()
        {
            io::send_mouse(io::MouseButton::Right);
            self.double_click_origin = None;
        }

        let now = Instant::now();

        self.spam_left.step(now);
        self.spam_right.step(now);
        self.spam_space.step(now);
    }

    fn on_idle(&mut self, resources: &'resources Resources) -> Result<Detail> {
        if self.keys.prev.is_pressed() {
            if self.keys.prev_skip.is_pressed() {
                self.recipes.decrement_skip(resources)?;
            } else {
                self.recipes.decrement(resources)?;
            }
        }

        if self.keys.next.is_pressed() {
            if self.keys.next_skip.is_pressed() {
                self.recipes.increment_skip(resources)?;
            } else {
                self.recipes.increment(resources)?;
            }
        }

        let retval = if self.keys.begin_trade.is_pressed() {
            io::send_mouse(io::MouseButton::Left);

            Detail::TradingFirst {
                state: TradeFirst::InvClicked,
                position: io::get_cursor()?,
                origin: Instant::now(),
            }
        } else if self.keys.record.is_pressed() {
            Detail::Recording {
                clicks: Vec::new(),
                count: 0,
            }
        } else if self.keys.play.is_pressed()
            && let Some(path) = self.recipes.get_path()?
        {
            Detail::Playing {
                clicks: io::load_clicks(path.join(io::FILENAME_CLICKS))?
                    .into_iter()
                    .map(|grid| (grid, Cursor::New))
                    .collect(),
                origin: Instant::now(),
            }
        } else if self.keys.delete.is_pressed() {
            Detail::Deleting
        } else if self.keys.rename.is_pressed() {
            Detail::Renaming {
                name: String::new(),
                draw_required: false,
            }
        } else {
            Detail::Idle
        };

        Ok(retval)
    }

    fn on_record(&mut self, mut clicks: Vec<Grid>, mut count: usize) -> Result<Detail> {
        if self.keys.click.is_pressed() {
            let coord = Coord::from(io::get_cursor()?);

            if let Ok(grid) = coord.try_into() {
                self.draw_required = true;

                count = match clicks.last() {
                    Some(last) if last == &grid => count + 1,
                    _ => 1,
                };

                clicks.push(grid);
            }
        }

        let retval = if self.keys.record.is_pressed() {
            self.is_locked = true;

            Detail::Naming {
                clicks,
                name: String::new(),
                draw_required: false,
            }
        } else {
            Detail::Recording { clicks, count }
        };

        Ok(retval)
    }

    fn on_name(
        &mut self,
        clicks: Vec<Grid>,
        name: String,
        resources: &'resources Resources,
    ) -> Result<Detail> {
        let retval = if self.keys.confirm.is_pressed() {
            match io::save_clicks(Self::SCREENSHOTS, Recipes::RECIPES, &clicks, &name) {
                Err(e) => {
                    io::message_box(format!("Reason: {e}"), "Failed to crate recipe")?;

                    Detail::Naming {
                        clicks,
                        name,
                        draw_required: false,
                    }
                }
                Ok(_) => {
                    self.reload_recipes(resources)?;
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
            TradeFirst::Waiting if self.keys.end_trade.is_pressed() => {
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

    fn on_delete(&mut self, resources: &'resources Resources) -> Result<Detail> {
        let retval = if self.keys.confirm.is_pressed() {
            match self.recipes.delete(resources) {
                Err(e) => {
                    io::message_box(format!("Reason: {e}"), "Failed to delete recipe")?;
                    Detail::Deleting
                }
                Ok(_) => Detail::Idle,
            }
        } else {
            Detail::Deleting
        };

        Ok(retval)
    }

    fn on_rename(&mut self, name: String, resources: &'resources Resources) -> Result<Detail> {
        let retval = if self.keys.confirm.is_pressed() {
            match self
                .recipes
                .rename(Path::new(Recipes::RECIPES).join(&name), resources)
            {
                Err(e) => {
                    io::message_box(format!("Reason: {e}"), "Failed to rename recipe")?;
                    Detail::Renaming {
                        name,
                        draw_required: false,
                    }
                }
                Ok(_) => Detail::Idle,
            }
        } else {
            Detail::Renaming {
                name,
                draw_required: false,
            }
        };

        Ok(retval)
    }

    fn update_keys(&mut self) {
        // does not count as a modification which needs redraw
        self.keys.click.update(false);
        self.keys.right_click.update(false);

        let mut update_nolock = |key: &mut Key| {
            key.update(false);

            if key.is_pressed() {
                self.draw_required = true;
            }
        };

        update_nolock(&mut self.keys.abort);
        update_nolock(&mut self.keys.lock);
        update_nolock(&mut self.keys.cancel_dc);
        update_nolock(&mut self.keys.confirm);

        let is_locked = self.is_locked();
        let mut update = |key: &mut Key| {
            key.update(is_locked);

            if key.is_pressed() {
                self.draw_required = true;
            }
        };

        update(&mut self.keys.left);
        update(&mut self.keys.right);
        update(&mut self.keys.space);
        update(&mut self.keys.record);
        update(&mut self.keys.play);
        update(&mut self.keys.prev);
        update(&mut self.keys.next);
        update(&mut self.keys.double_click);
        update(&mut self.keys.begin_trade);
        update(&mut self.keys.end_trade);
        update(&mut self.keys.prev_skip);
        update(&mut self.keys.next_skip);
        update(&mut self.keys.delete);
        update(&mut self.keys.rename);
    }

    fn toggle_spams(&mut self) {
        let toggle = |key: &Key, spam: &mut Spam| {
            if key.is_pressed() {
                spam.toggle_active();
            }
        };

        toggle(&self.keys.left, &mut self.spam_left);
        toggle(&self.keys.right, &mut self.spam_right);
        toggle(&self.keys.space, &mut self.spam_space);
    }

    fn double_click_disable_condition(&self) -> bool {
        io::is_down(Keys::CANCEL_DC) || self.spam_right.is_active()
    }

    fn reload_recipes(&mut self, resources: &'resources Resources) -> Result<()> {
        self.recipes = Recipes::new(resources)?;
        Ok(())
    }
}
