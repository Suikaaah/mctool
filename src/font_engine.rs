use crate::map_err_anyhow::MapErrAnyhow;
use anyhow::Result;
use sdl2::ttf::{Font, Sdl2TtfContext};
use std::path::Path;

pub struct FontEngine {
    context: Sdl2TtfContext,
}

impl FontEngine {
    pub fn new() -> Result<Self> {
        let context = sdl2::ttf::init().map_err_anyhow()?;

        Ok(Self { context })
    }

    pub fn load_font<P>(&self, path: P, point_size: u16) -> Result<Font<'_, 'static>>
    where
        P: AsRef<Path>,
    {
        self.context.load_font(path, point_size).map_err_anyhow()
    }
}
