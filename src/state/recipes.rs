use crate::resources::{Resources, Textures};
use anyhow::{Result, anyhow};
use std::{fmt, path::PathBuf};

pub struct Recipes<'tex> {
    paths: Box<[(PathBuf, char)]>,
    index: Option<usize>,
    textures: Option<Textures<'tex>>,
}

impl<'tex> Recipes<'tex> {
    pub fn new(paths: Box<[PathBuf]>, resources: &'tex Resources) -> Result<Self> {
        let paths_opt: Option<Box<[(PathBuf, char)]>> = paths
            .into_iter()
            .map(|path| {
                let first_character = path.file_name()?.to_str()?.chars().next()?;
                Some((path, first_character))
            })
            .collect();

        let paths =
            paths_opt.ok_or_else(|| anyhow!("failed to extract first characters from files"))?;

        let index = Self::last_index(&paths);

        let textures = match Self::get_ext(&paths, index)? {
            None => None,
            Some((path, _)) => Some(resources.load_textures(path)?),
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
        self.get().map(|opt| opt.map(|(path, _)| path))
    }

    pub fn increment(&mut self, resources: &'tex Resources) -> Result<()> {
        self.increment_detail();
        self.update_textures(resources)
    }

    pub fn decrement(&mut self, resources: &'tex Resources) -> Result<()> {
        self.decrement_detail();
        self.update_textures(resources)
    }

    pub fn increment_skip(&mut self, resources: &'tex Resources) -> Result<()> {
        self.skip_detail(Self::increment_detail);
        self.update_textures(resources)
    }

    pub fn decrement_skip(&mut self, resources: &'tex Resources) -> Result<()> {
        self.decrement_detail();
        self.skip_detail(Self::decrement_detail);
        self.increment_detail();
        self.update_textures(resources)
    }

    pub fn update_textures(&mut self, resources: &'tex Resources) -> Result<()> {
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

    fn skip_detail(&mut self, mut f: impl FnMut(&mut Self)) {
        if !self.ignore_skip()
            && let Some((_, sample)) = self.get().ok().flatten()
        {
            while let Some((_, first_character)) = self.get().ok().flatten()
                && sample == first_character
            {
                f(self);
            }
        }
    }

    fn get(&self) -> Result<Option<(&PathBuf, char)>> {
        Self::get_ext(&self.paths, self.index)
    }

    fn get_ext(
        paths: &[(PathBuf, char)],
        index: Option<usize>,
    ) -> Result<Option<(&PathBuf, char)>> {
        match index {
            None => Ok(None),
            Some(index) => paths
                .get(index)
                .ok_or_else(|| anyhow!("this should not be reachable. "))
                .map(|(path, first_character)| Some((path, *first_character))),
        }
    }

    fn ignore_skip(&self) -> bool {
        match self.paths.first() {
            Some((_, sample)) => self
                .paths
                .iter()
                .all(|(_, first_character)| first_character == sample),
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
