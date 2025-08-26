use crate::{io, map_err_anyhow::MapErrAnyhow};
use anyhow::Result;
use sdl2::{
    image::LoadTexture,
    render::{Texture, TextureCreator},
    ttf::{Font, Sdl2TtfContext},
    video::WindowContext,
};
use std::path::Path;

pub struct Resources {
    ttf: Sdl2TtfContext,
    tex_creator: TextureCreator<WindowContext>,
}

pub struct Fonts<'ttf> {
    pub regular: Font<'ttf, 'static>,
    pub large: Font<'ttf, 'static>,
}

pub struct Textures<'creator> {
    pub thumbnail: Texture<'creator>,
    pub item: Texture<'creator>,
}

impl Resources {
    const FONT: &str = "CascadiaMono.ttf";
    const REGULAR: u16 = 16;
    const LARGE: u16 = 22;

    pub fn new(tex_creator: TextureCreator<WindowContext>) -> Result<Self> {
        let ttf = sdl2::ttf::init().map_err_anyhow()?;

        Ok(Self { ttf, tex_creator })
    }

    pub fn load_fonts(&self) -> Result<Fonts<'_>> {
        Ok(Fonts {
            regular: self.load_font(Self::FONT, Self::REGULAR)?,
            large: self.load_font(Self::FONT, Self::LARGE)?,
        })
    }

    pub fn load_textures(&self, path: impl AsRef<Path>) -> Result<Textures<'_>> {
        let path = path.as_ref();

        Ok(Textures {
            thumbnail: self.load_texture(path.join(io::FILENAME_THUMBNAIL))?,
            item: self.load_texture(path.join(io::FILENAME_ITEM))?,
        })
    }

    fn load_texture(&self, path: impl AsRef<Path>) -> Result<Texture<'_>> {
        self.tex_creator.load_texture(path).map_err_anyhow()
    }

    fn load_font<P>(&self, path: P, point_size: u16) -> Result<Font<'_, 'static>>
    where
        P: AsRef<Path>,
    {
        self.ttf.load_font(path, point_size).map_err_anyhow()
    }
}
