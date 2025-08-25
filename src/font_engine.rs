use crate::map_err_anyhow::MapErrAnyhow;
use anyhow::Result;
use sdl2::ttf::{Font, Sdl2TtfContext};
use std::path::Path;

pub struct FontEngine {
    context: Sdl2TtfContext,
}

pub struct Fonts<'ttf> {
    pub regular: Font<'ttf, 'static>,
    pub large: Font<'ttf, 'static>,
}

impl FontEngine {
    const FONT: &str = "CascadiaMono.ttf";
    const REGULAR: u16 = 16;
    const LARGE: u16 = 22;

    pub fn new() -> Result<Self> {
        let context = sdl2::ttf::init().map_err_anyhow()?;

        Ok(Self { context })
    }

    pub fn load_fonts(&self) -> Result<Fonts<'_>> {
        Ok(Fonts {
            regular: self.load_font(Self::FONT, Self::REGULAR)?,
            large: self.load_font(Self::FONT, Self::LARGE)?,
        })
    }

    fn load_font<P>(&self, path: P, point_size: u16) -> Result<Font<'_, 'static>>
    where
        P: AsRef<Path>,
    {
        self.context.load_font(path, point_size).map_err_anyhow()
    }
}
