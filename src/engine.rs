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
    const PADDING: u32 = 16;
    const PADDING_BOTTOM: u32 = 48;
    const WIDTH: u32 = io::INV_WIDTH + Self::PADDING * 2;
    const HEIGHT: u32 = io::INV_HEIGHT + Self::PADDING * 2 + Self::PADDING_BOTTOM;
    const POLLING_RATE: Duration = Duration::from_millis(2);
    const BACKGROUND: Color = Color::RGB(0x4F, 0x4F, 0x4F);

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

            if let Some(path) = state.recipes.get() {
                let texture = self
                    .tex_creator
                    .load_texture(path.join(io::FILENAME_THUMBNAIL))
                    .expect("failed to create texture");

                let query = texture.query();

                let dst = Rect::new(
                    Self::PADDING as i32,
                    Self::PADDING as i32,
                    query.width,
                    query.height,
                );

                self.canvas
                    .copy(&texture, None, dst)
                    .expect("failed to copy to canvas");
            }

            if state.spam_left.is_active() {
                self.render_font(font, "left", (0, 50));
            }

            if state.spam_right.is_active() {
                self.render_font(font, "right", (0, 80));
            }

            if state.spam_space.is_active() {
                self.render_font(font, "space", (0, 110));
            }

            match state.detail() {
                Detail::Idle => (),
                Detail::Recording { .. } => self.render_font(font, "recording...", (0, 0)),
                Detail::Playing { .. } => self.render_font(font, "playing...", (0, 0)),
            }

            self.canvas.present();

            println!("drawn {:?}", std::time::SystemTime::now());
        }
    }

    pub fn sleep() {
        std::thread::sleep(Self::POLLING_RATE);
    }

    fn render_font(&mut self, font: &Font, text: &str, position: (i32, i32)) {
        let surface = font
            .render(text)
            .blended(Color::WHITE)
            .expect("failed to render text");

        self.draw_surface(surface, position);
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
}
