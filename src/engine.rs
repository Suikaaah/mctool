use crate::{
    io,
    state::{State, detail::Detail},
};
use anyhow::{Result, anyhow};
use sdl2::{
    EventPump, Sdl,
    event::Event,
    image::LoadTexture,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureCreator},
    surface::Surface,
    ttf::Font,
    video::{Window, WindowContext},
};
use std::time::Duration;

pub struct Engine {
    frame_initialized: bool,
    tex_creator: TextureCreator<WindowContext>,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    _context: Sdl,
}

impl Engine {
    const TITLE: &str = "mctool";
    const PADDING: u32 = 32;
    const WIDTH: u32 = io::INV_WIDTH + io::ITEM_WIDTH + Self::PADDING * 3;
    const HEIGHT: u32 = io::INV_HEIGHT + Self::PADDING * 2 + Self::TAB_HEIGHT;
    const CENTER: (i32, i32) = (Self::WIDTH as i32 / 2, Self::HEIGHT as i32 / 2);
    const TAB_WIDTH: u32 = 110;
    const TAB_HEIGHT: u32 = 24;
    const POLLING_RATE: Duration = Duration::from_millis(1);
    const BACKGROUND: Color = Color::RGB(0x4F, 0x4F, 0x4F);
    const TAB_BACKGROUND: Color = Color::RGB(0x38, 0x38, 0x38);
    const GREEN: Color = Color::RGB(0x00, 0x7F, 0x00);
    const RED: Color = Color::RGB(0x7F, 0x00, 0x00);

    pub fn new() -> Result<Self> {
        let context = sdl2::init().map_err(|e| anyhow!("{e}"))?;

        let event_pump = context.event_pump().map_err(|e| anyhow!("{e}"))?;

        let canvas = context
            .video()
            .map_err(|e| anyhow!("{e}"))?
            .window(Self::TITLE, Self::WIDTH, Self::HEIGHT)
            .position_centered()
            .build()?
            .into_canvas()
            .accelerated()
            .build()?;

        let tex_creator = canvas.texture_creator();

        Ok(Self {
            frame_initialized: false,
            tex_creator,
            canvas,
            event_pump,
            _context: context,
        })
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        self.event_pump.poll_event()
    }

    pub fn draw(&mut self, state: &State, font: &Font) -> Result<()> {
        if state.draw_required() || !self.frame_initialized {
            self.frame_initialized = true;
            self.canvas.set_draw_color(Self::BACKGROUND);
            self.canvas.clear();

            match state.detail() {
                Detail::Idle => self.render_thumbnail(state),
                Detail::Recording { .. } => {
                    self.render_font_centered(font, "Recording...", Self::CENTER, Color::WHITE)
                }
                Detail::Playing { .. } => {
                    self.render_font_centered(font, "Playing...", Self::CENTER, Color::WHITE)
                }
                Detail::TradingFirst { .. } | Detail::TradingSecond { .. } => {
                    self.render_font_centered(font, "Trading...", Self::CENTER, Color::WHITE)
                }
            }?;

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
                .and_then(|()| self.render_font_centered(font, text, (cx, cy), Color::WHITE))
            };

            tab(
                0,
                "DOUBLE",
                state.double_click_active(),
                state.double_click_temporarily_disabled(),
            )?;
            tab(1, "LEFT", state.spam_left.is_active(), false)?;
            tab(2, "RIGHT", state.spam_right.is_active(), false)?;
            tab(3, "SPACE", state.spam_space.is_active(), false)?;

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

            self.render_font_centered(
                font,
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
            )?;

            self.canvas
                .window_mut()
                .set_title(&format!("{} [{}]", Self::TITLE, state.recipes))?;

            self.canvas.present();
        }

        Ok(())
    }

    pub fn sleep() {
        std::thread::sleep(Self::POLLING_RATE);
    }

    fn render_thumbnail(&mut self, state: &State) -> Result<()> {
        if let Some(path) = state.recipes.get_path()? {
            {
                let texture = self
                    .tex_creator
                    .load_texture(path.join(io::FILENAME_THUMBNAIL))
                    .map_err(|e| anyhow!("{e}"))?;

                let dst = Rect::new(
                    Self::PADDING as i32,
                    Self::PADDING as i32,
                    io::INV_WIDTH,
                    io::INV_HEIGHT,
                );

                self.canvas
                    .copy(&texture, None, dst)
                    .map_err(|e| anyhow!("{e}"))?;
            }
            {
                let texture = self
                    .tex_creator
                    .load_texture(path.join(io::FILENAME_ITEM))
                    .map_err(|e| anyhow!("{e}"))?;

                let dst = Rect::new(
                    Self::WIDTH as i32 - Self::PADDING as i32 - io::ITEM_WIDTH as i32,
                    Self::PADDING as i32 + io::INV_HEIGHT as i32 / 2 - io::ITEM_HEIGHT as i32 / 2,
                    io::ITEM_WIDTH,
                    io::ITEM_HEIGHT,
                );

                self.canvas
                    .copy(&texture, None, dst)
                    .map_err(|e| anyhow!("{e}"))?;
            }
        }

        Ok(())
    }

    fn render_font_centered(
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
        let texture = surface.as_texture(&self.tex_creator)?;

        self.canvas
            .copy(
                &texture,
                None,
                Rect::new(x, y, surface.width(), surface.height()),
            )
            .map_err(|e| anyhow!("{e}"))
    }

    fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color);
        self.canvas.fill_rect(rect).map_err(|e| anyhow!("{e}"))
    }
}
