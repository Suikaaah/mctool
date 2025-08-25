use anyhow::{Result, anyhow};
use std::{fmt, path::PathBuf};

pub struct Recipes {
    paths: Box<[(PathBuf, char)]>,
    index: Option<usize>,
}

impl Recipes {
    pub fn new(paths: Box<[PathBuf]>) -> Result<Self> {
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

        Ok(Self { paths, index })
    }

    pub fn get_path(&self) -> Result<Option<&PathBuf>> {
        self.get().map(|opt| opt.map(|(path, _)| path))
    }

    pub const fn increment(&mut self) {
        if let Some(index) = &mut self.index {
            *index = if *index + 1 == self.paths.len() {
                0
            } else {
                *index + 1
            };
        }
    }

    pub const fn decrement(&mut self) {
        if let Some(index) = &mut self.index {
            *index = if *index == 0 {
                // since paths' length is fixed, len is always non-zero
                self.paths.len() - 1
            } else {
                *index - 1
            };
        }
    }

    pub fn increment_skip(&mut self) {
        self.skip_detail(Self::increment);
    }

    pub fn decrement_skip(&mut self) {
        self.decrement();
        self.skip_detail(Self::decrement);
        self.increment();
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
        match self.index {
            None => Ok(None),
            Some(index) => self
                .paths
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

impl fmt::Display for Recipes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = self
            .get_path()
            .ok()
            .flatten()
            .and_then(|path| path.file_name().and_then(|osstr| osstr.to_str()))
            .and_then(|name| {
                let index = self.index? + 1;
                let len = self.paths.len();
                Some(format!("◀ {name} [{index}/{len}] ▶"))
            })
            .unwrap_or_else(|| String::from("no recipes found"));

        write!(f, "{str}")
    }
}
