use sdl2::ttf::{Font, Sdl2TtfContext};

pub struct FontEngine {
    context: Sdl2TtfContext,
}

impl FontEngine {
    const FONT: &str = "CascadiaMono.ttf";

    pub fn new() -> Self {
        Self {
            context: sdl2::ttf::init().expect("failed to initialize ttf"),
        }
    }

    pub fn load_font(&self, point_size: u16) -> Result<Font<'_, 'static>, String> {
        self.context.load_font(Self::FONT, point_size)
    }
}
