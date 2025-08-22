use crate::map_err_anyhow::MapErrAnyhow;
use anyhow::Result;
use sdl2::ttf::{Font, Sdl2TtfContext};

pub struct FontEngine {
    context: Sdl2TtfContext,
}

impl FontEngine {
    const FONT: &str = "CascadiaMono.ttf";

    pub fn new() -> Result<Self> {
        let context = sdl2::ttf::init().map_err_anyhow()?;

        Ok(Self { context })
    }

    pub fn load_font(&self, point_size: u16) -> Result<Font<'_, 'static>> {
        let font = self
            .context
            .load_font(Self::FONT, point_size)
            .map_err_anyhow()?;

        Ok(font)
    }
}
