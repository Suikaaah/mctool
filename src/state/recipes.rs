use crate::{
    io,
    resources::{Resources, Textures},
};
use anyhow::{Result, anyhow, bail};
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Recipes<'resources> {
    paths: Box<[PathChar]>,
    index: Option<usize>,
    textures: Option<Textures<'resources>>,
}

struct PathChar {
    path: PathBuf,
    first_char: char,
}

impl<'resources> Recipes<'resources> {
    pub const RECIPES: &'static str = r"D:\rust\mctool\recipes";

    pub fn new(resources: &'resources Resources) -> Result<Self> {
        let paths_opt: Option<Box<[PathChar]>> = io::recipes(Self::RECIPES)?
            .into_iter()
            .map(|path| {
                let first_char = path.file_name()?.to_str()?.chars().next()?;
                Some(PathChar { path, first_char })
            })
            .collect();

        let paths =
            paths_opt.ok_or_else(|| anyhow!("failed to extract first characters from files"))?;

        let index = Self::last_index(&paths);

        let textures = match Self::get_ext(&paths, index)? {
            None => None,
            Some(PathChar { path, .. }) => Some(resources.load_textures(path)?),
        };

        Ok(Self {
            paths,
            index,
            textures,
        })
    }

    pub const fn textures(&self) -> Option<&Textures<'_>> {
        self.textures.as_ref()
    }

    pub const fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn get_path(&self) -> Result<Option<&PathBuf>> {
        self.get().map(|opt| opt.map(|PathChar { path, .. }| path))
    }

    pub fn increment(&mut self, resources: &'resources Resources) -> Result<()> {
        self.increment_detail();
        self.update_textures(resources)
    }

    pub fn decrement(&mut self, resources: &'resources Resources) -> Result<()> {
        self.decrement_detail();
        self.update_textures(resources)
    }

    pub fn increment_skip(&mut self, resources: &'resources Resources) -> Result<()> {
        self.skip_detail(Self::increment_detail);
        self.update_textures(resources)
    }

    pub fn decrement_skip(&mut self, resources: &'resources Resources) -> Result<()> {
        self.decrement_detail();
        self.skip_detail(Self::decrement_detail);
        self.increment_detail();
        self.update_textures(resources)
    }

    pub fn update_textures(&mut self, resources: &'resources Resources) -> Result<()> {
        self.textures = match self.get_path()? {
            None => None,
            Some(path) => Some(resources.load_textures(path)?),
        };

        Ok(())
    }

    const fn increment_detail(&mut self) {
        let len = self.len();

        if let Some(index) = &mut self.index {
            *index = if *index + 1 == len { 0 } else { *index + 1 };
        }
    }

    const fn decrement_detail(&mut self) {
        let len = self.len();

        if let Some(index) = &mut self.index {
            // 0 < len
            *index = if *index == 0 { len - 1 } else { *index - 1 };
        }
    }

    pub fn delete(&mut self, resources: &'resources Resources) -> Result<()> {
        if let Some(path) = self.get_path()? {
            std::fs::remove_dir_all(path)?;
            *self = Self::new(resources)?;
        }

        Ok(())
    }

    pub fn rename(&mut self, to: impl AsRef<Path>, resources: &'resources Resources) -> Result<()> {
        if let Some(from) = self.get_path()? {
            if from == to.as_ref() {
                bail!("rename not needed");
            }

            std::fs::rename(from, to)?;
            *self = Self::new(resources)?;
        }

        Ok(())
    }

    fn skip_detail(&mut self, mut f: impl FnMut(&mut Self)) {
        if !self.ignore_skip()
            && let Some(PathChar {
                first_char: sample, ..
            }) = self.get().ok().flatten()
        {
            let sample = *sample;

            while let Some(PathChar { first_char, .. }) = self.get().ok().flatten()
                && sample == *first_char
            {
                f(self);
            }
        }
    }

    fn get(&self) -> Result<Option<&PathChar>> {
        Self::get_ext(&self.paths, self.index)
    }

    fn get_ext(paths: &[PathChar], index: Option<usize>) -> Result<Option<&PathChar>> {
        match index {
            None => Ok(None),
            Some(index) => paths
                .get(index)
                .ok_or_else(|| anyhow!("this should not be reachable. "))
                .map(Some),
        }
    }

    fn ignore_skip(&self) -> bool {
        match self.paths.first() {
            Some(PathChar {
                first_char: sample, ..
            }) => self
                .paths
                .iter()
                .all(|PathChar { first_char, .. }| first_char == sample),
            None => true,
        }
    }

    const fn last_index<T>(value: &[T]) -> Option<usize> {
        match value.len() {
            0 => None,
            len => Some(len - 1),
        }
    }
}

impl fmt::Display for Recipes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = self
            .get_path()
            .ok()
            .flatten()
            .and_then(|path| path.file_name().and_then(|osstr| osstr.to_str()))
            .and_then(|name| {
                let index = self.index? + 1;
                let len = self.len();
                Some(format!("◀ {name} [{index}/{len}] ▶"))
            })
            .unwrap_or_else(|| String::from("no recipes found"));

        write!(f, "{str}")
    }
}
