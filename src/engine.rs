use crate::{
    io,
    state::{Detail, State},
};
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
    const TAB_WIDTH: u32 = 100;
    const TAB_HEIGHT: u32 = 24;
    const POLLING_RATE: Duration = Duration::from_millis(1);
    const BACKGROUND: Color = Color::RGB(0x4F, 0x4F, 0x4F);
    const TAB_BACKGROUND: Color = Color::RGB(0x38, 0x38, 0x38);
    const GREEN: Color = Color::RGB(0x00, 0x7F, 0x00);
    const RED: Color = Color::RGB(0x7F, 0x00, 0x00);

    pub fn new() -> Self {
        let context = sdl2::init().expect("failed to initialize sdl2");

        let event_pump = context.event_pump().expect("failed to obtain event_pump");

        let canvas = context
            .video()
            .expect("failed to initialize video")
            .window(Self::TITLE, Self::WIDTH, Self::HEIGHT)
            .position_centered()
            .build()
            .expect("failed to build window")
            .into_canvas()
            .accelerated()
            .build()
            .expect("failed to build canvas");

        let tex_creator = canvas.texture_creator();

        Self {
            frame_initialized: false,
            tex_creator,
            canvas,
            event_pump,
            _context: context,
        }
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        self.event_pump.poll_event()
    }

    pub fn draw(&mut self, state: &State, font: &Font) {
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
            }

            let mut tab = |i, text, is_active| {
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
                    if is_active {
                        Self::GREEN
                    } else {
                        Self::TAB_BACKGROUND
                    },
                );

                self.render_font_centered(font, text, (cx, cy), Color::WHITE);
            };

            tab(0, "DOUBLE", state.double_click_active());
            tab(1, "LEFT", state.spam_left.is_active());
            tab(2, "RIGHT", state.spam_right.is_active());
            tab(3, "SPACE", state.spam_space.is_active());

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
            );

            self.render_font_centered(
                font,
                "LOCKED",
                (
                    Self::WIDTH as i32 - Self::TAB_WIDTH as i32 / 2,
                    Self::HEIGHT as i32 - Self::TAB_HEIGHT as i32 / 2,
                ),
                Color::WHITE,
            );

            self.canvas
                .window_mut()
                .set_title(&format!("mctool [{}]", state.recipes))
                .expect("failed to set title");

            self.canvas.present();
        }
    }

    pub fn sleep() {
        std::thread::sleep(Self::POLLING_RATE);
    }

    fn render_thumbnail(&mut self, state: &State) {
        if let Some(path) = state.recipes.get() {
            {
                let texture = self
                    .tex_creator
                    .load_texture(path.join(io::FILENAME_THUMBNAIL))
                    .expect("failed to create texture");

                let dst = Rect::new(
                    Self::PADDING as i32,
                    Self::PADDING as i32,
                    io::INV_WIDTH,
                    io::INV_HEIGHT,
                );

                self.canvas
                    .copy(&texture, None, dst)
                    .expect("failed to copy to canvas");
            }
            {
                let texture = self
                    .tex_creator
                    .load_texture(path.join(io::FILENAME_ITEM))
                    .expect("failed to create texture");

                let dst = Rect::new(
                    Self::WIDTH as i32 - Self::PADDING as i32 - io::ITEM_WIDTH as i32,
                    Self::PADDING as i32 + io::INV_HEIGHT as i32 / 2 - io::ITEM_HEIGHT as i32 / 2,
                    io::ITEM_WIDTH,
                    io::ITEM_HEIGHT,
                );

                self.canvas
                    .copy(&texture, None, dst)
                    .expect("failed to copy to canvas");
            }
        }
    }

    fn render_font_centered(&mut self, font: &Font, text: &str, (x, y): (i32, i32), color: Color) {
        let surface = font
            .render(text)
            .blended(color)
            .expect("failed to render text");

        let center = (
            x - surface.width() as i32 / 2,
            y - surface.height() as i32 / 2,
        );

        self.draw_surface(surface, center);
    }

    fn draw_surface(&mut self, surface: Surface, (x, y): (i32, i32)) {
        let texture = surface
            .as_texture(&self.tex_creator)
            .expect("failed to convert to texture");

        self.canvas
            .copy(
                &texture,
                None,
                Rect::new(x, y, surface.width(), surface.height()),
            )
            .expect("failed to copy to canvas");
    }

    fn draw_rect(&mut self, rect: Rect, color: Color) {
        self.canvas.set_draw_color(color);
        self.canvas.fill_rect(rect).expect("failed to fill rect");
    }
}
