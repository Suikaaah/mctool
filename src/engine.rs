use crate::{
    io,
    state::{Detail, State, spam::Spam},
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
    const PADDING: u32 = 48;
    const WIDTH: u32 = io::INV_WIDTH + io::ITEM_WIDTH + Self::PADDING * 3;
    const HEIGHT: u32 = io::INV_HEIGHT + Self::PADDING * 2 + Self::TAB_HEIGHT;
    const CENTER: (i32, i32) = (Self::WIDTH as i32 / 2, Self::HEIGHT as i32 / 2);
    const TAB_WIDTH: u32 = 100;
    const TAB_HEIGHT: u32 = 24;
    const INDEX_WIDTH: u32 = 150;
    const POLLING_RATE: Duration = Duration::from_millis(1);
    const BACKGROUND: Color = Color::RGB(0x4F, 0x4F, 0x4F);
    const TAB_BACKGROUND: Color = Color::RGB(0x38, 0x38, 0x38);

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
            }

            let mut tab = |i, text, spam: &Spam| {
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
                    if spam.is_active() {
                        Color::GREEN
                    } else {
                        Self::TAB_BACKGROUND
                    },
                );

                self.render_font_centered(
                    font,
                    text,
                    (cx, cy),
                    if spam.is_active() {
                        Self::TAB_BACKGROUND
                    } else {
                        Color::WHITE
                    },
                );
            };

            tab(0, "LEFT", &state.spam_left);
            tab(1, "RIGHT", &state.spam_right);
            tab(2, "SPACE", &state.spam_space);

            self.draw_rect(
                Rect::new(
                    Self::WIDTH as i32 - Self::INDEX_WIDTH as i32,
                    Self::HEIGHT as i32 - Self::TAB_HEIGHT as i32,
                    Self::INDEX_WIDTH,
                    Self::TAB_HEIGHT,
                ),
                Self::TAB_BACKGROUND,
            );

            self.render_font_centered(
                font,
                state.recipes.to_string().as_str(),
                (
                    Self::WIDTH as i32 - Self::INDEX_WIDTH as i32 / 2,
                    Self::HEIGHT as i32 - Self::TAB_HEIGHT as i32 / 2,
                ),
                Color::WHITE,
            );

            self.canvas.present();

            println!("drawn {:?}", std::time::SystemTime::now());
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
