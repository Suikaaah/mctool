use crate::{
    io,
    map_err_anyhow::MapErrAnyhow,
    resources::Fonts,
    resources::Textures,
    state::{State, detail::Detail},
};
use anyhow::Result;
use sdl2::{
    EventPump, Sdl, VideoSubsystem,
    event::Event,
    pixels::Color,
    rect::Rect,
    render::{BlendMode, Canvas, TextureCreator},
    surface::Surface,
    ttf::Font,
    video::{Window, WindowContext},
};
use std::time::Duration;

pub struct Engine {
    frame_initialized: bool,
    canvas: Canvas<Window>,
    video: VideoSubsystem,
    event_pump: EventPump,
    _context: Sdl,
}

impl Engine {
    const TITLE: &str = "mctool";
    const PADDING: u32 = 32;
    const WIDTH: u32 = io::INV_WIDTH + io::ITEM_WIDTH + Self::PADDING * 3;
    const HEIGHT: u32 = io::INV_HEIGHT + Self::PADDING * 3 + Self::TAB_HEIGHT + Self::PATH_HEIGHT;
    const CENTER: (i32, i32) = (Self::WIDTH as i32 / 2, Self::HEIGHT as i32 / 2);
    const TAB_WIDTH: u32 = 110;
    const TAB_HEIGHT: u32 = 24;
    const PATH_HEIGHT: u32 = 6;
    const POLLING_RATE: Duration = Duration::from_millis(1);
    const BACKGROUND: Color = Color::RGB(0x4F, 0x4F, 0x4F);
    const TAB_BACKGROUND: Color = Color::RGB(0x38, 0x38, 0x38);
    const GREEN: Color = Color::RGB(0x00, 0x7F, 0x00);
    const RED: Color = Color::RGB(0x7F, 0x00, 0x00);
    const DIM: Color = Color::RGBA(0x00, 0x00, 0x00, 0xC0);

    pub fn new() -> Result<Self> {
        let context = sdl2::init().map_err_anyhow()?;

        let event_pump = context.event_pump().map_err_anyhow()?;

        let video = context.video().map_err_anyhow()?;

        let mut canvas = video
            .window(Self::TITLE, Self::WIDTH, Self::HEIGHT)
            .position_centered()
            .build()?
            .into_canvas()
            .accelerated()
            .build()?;

        canvas.set_blend_mode(BlendMode::Blend);

        Ok(Self {
            frame_initialized: false,
            canvas,
            video,
            event_pump,
            _context: context,
        })
    }

    pub fn tex_creator(&self) -> TextureCreator<WindowContext> {
        self.canvas.texture_creator()
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        self.event_pump.poll_event()
    }

    pub fn draw(&mut self, state: &State, fonts: &Fonts) -> Result<()> {
        let draw = state.draw_required() || !self.frame_initialized;

        if draw {
            self.frame_initialized = true;
            self.canvas.set_draw_color(Self::BACKGROUND);
            self.canvas.clear();

            let mut tab = |i, text, is_active, red| {
                let y = Self::HEIGHT - Self::TAB_HEIGHT;
                let cx = Self::TAB_WIDTH as i32 * i + Self::TAB_WIDTH as i32 / 2;
                let cy = Self::HEIGHT as i32 - Self::TAB_HEIGHT as i32 / 2;

                self.draw_rect(
                    Rect::new(
                        Self::TAB_WIDTH as i32 * i,
                        y as i32,
                        Self::TAB_WIDTH,
                        Self::TAB_HEIGHT,
                    ),
                    if red {
                        Self::RED
                    } else if is_active {
                        Self::GREEN
                    } else {
                        Self::TAB_BACKGROUND
                    },
                )
                .and_then(|_| self.draw_font_centered(&fonts.regular, text, (cx, cy), Color::WHITE))
            };

            tab(
                0,
                "DOUBLE",
                state.double_click_active_mapped(),
                state.double_click_temporarily_disabled(),
            )?;
            tab(1, "LEFT", state.spam_left.is_active(), false)?;
            tab(2, "RIGHT", state.spam_right.is_active(), false)?;
            tab(3, "SPACE", state.spam_space.is_active(), false)?;

            self.draw_lock(state, fonts)?;
            self.draw_thumbnail(state, fonts)?;

            match state.detail() {
                Detail::Idle => (),
                Detail::Recording { clicks, count } => {
                    let display = match clicks.last() {
                        None => "Recording...".to_string(),
                        Some(last) if count < &2 => format!("[..., {last:?}]"),
                        Some(last) => format!("[..., {last:?} * {count}]"),
                    };

                    self.dim()?;
                    self.draw_font_centered(&fonts.large, &display, Self::CENTER, Color::WHITE)?;
                }
                Detail::Naming { name, .. } => {
                    self.dim()?;
                    self.draw_font_centered(
                        &fonts.large,
                        &format!("Filename: [{name}]"),
                        Self::CENTER,
                        Color::WHITE,
                    )?;
                }
                Detail::Playing { .. } => {
                    self.dim()?;
                    self.draw_font_centered(
                        &fonts.large,
                        "Playing...",
                        Self::CENTER,
                        Color::WHITE,
                    )?;
                }
                Detail::TradingFirst { .. } | Detail::TradingSecond { .. } => {
                    self.dim()?;
                    self.draw_font_centered(
                        &fonts.large,
                        "Trading...",
                        Self::CENTER,
                        Color::WHITE,
                    )?;
                }
            }

            self.canvas.present();
        }

        Ok(())
    }

    pub fn sleep() {
        std::thread::sleep(Self::POLLING_RATE);
    }

    pub fn start_text_input(&self) {
        self.video.text_input().start();
    }

    pub fn stop_text_input(&self) {
        self.video.text_input().stop();
    }

    fn dim(&mut self) -> Result<()> {
        self.draw_rect(Rect::new(0, 0, Self::WIDTH, Self::HEIGHT), Self::DIM)
    }

    fn draw_lock(&mut self, state: &State, fonts: &Fonts) -> Result<()> {
        self.draw_rect(
            Rect::new(
                Self::WIDTH as i32 - Self::TAB_WIDTH as i32,
                Self::HEIGHT as i32 - Self::TAB_HEIGHT as i32,
                Self::TAB_WIDTH,
                Self::TAB_HEIGHT,
            ),
            if state.is_locked() {
                Self::RED
            } else {
                Self::TAB_BACKGROUND
            },
        )?;

        self.draw_font_centered(
            &fonts.regular,
            if state.is_locked() {
                "LOCKED"
            } else {
                "UNLOCKED"
            },
            (
                Self::WIDTH as i32 - Self::TAB_WIDTH as i32 / 2,
                Self::HEIGHT as i32 - Self::TAB_HEIGHT as i32 / 2,
            ),
            Color::WHITE,
        )
    }

    fn draw_thumbnail(&mut self, state: &State, fonts: &Fonts) -> Result<()> {
        match &state.recipes.textures() {
            None => self.draw_font_centered(
                &fonts.large,
                &state.recipes.to_string(),
                Self::CENTER,
                Color::WHITE,
            ),
            Some(Textures { thumbnail, item }) => {
                self.draw_font_centered(
                    &fonts.large,
                    &state.recipes.to_string(),
                    (
                        Self::WIDTH as i32 / 2,
                        Self::PADDING as i32 + Self::PATH_HEIGHT as i32 / 2,
                    ),
                    Color::WHITE,
                )?;

                {
                    let dst = Rect::new(
                        Self::PADDING as i32,
                        (Self::PATH_HEIGHT + Self::PADDING * 2) as i32,
                        io::INV_WIDTH,
                        io::INV_HEIGHT,
                    );

                    self.canvas.copy(thumbnail, None, dst).map_err_anyhow()?;
                }
                {
                    let dst = Rect::new(
                        Self::WIDTH as i32 - Self::PADDING as i32 - io::ITEM_WIDTH as i32,
                        (Self::PATH_HEIGHT + Self::PADDING * 2) as i32 + io::INV_HEIGHT as i32 / 2
                            - io::ITEM_HEIGHT as i32 / 2,
                        io::ITEM_WIDTH,
                        io::ITEM_HEIGHT,
                    );

                    self.canvas.copy(item, None, dst).map_err_anyhow()?;
                }

                Ok(())
            }
        }
    }

    fn draw_font_centered(
        &mut self,
        font: &Font,
        text: &str,
        (x, y): (i32, i32),
        color: Color,
    ) -> Result<()> {
        let surface = font.render(text).blended(color)?;

        let center = (
            x - surface.width() as i32 / 2,
            y - surface.height() as i32 / 2,
        );

        self.draw_surface(surface, center)
    }

    fn draw_surface(&mut self, surface: Surface, (x, y): (i32, i32)) -> Result<()> {
        let tex_creator = self.canvas.texture_creator();
        let texture = surface.as_texture(&tex_creator)?;

        self.canvas
            .copy(
                &texture,
                None,
                Rect::new(x, y, surface.width(), surface.height()),
            )
            .map_err_anyhow()
    }

    fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color);
        self.canvas.fill_rect(rect).map_err_anyhow()
    }
}
