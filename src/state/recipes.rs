use anyhow::{Result, anyhow};
use std::{fmt, path::PathBuf};

pub struct Recipes {
    paths: Box<[PathBuf]>,
    index: Option<usize>,
}

impl Recipes {
    pub const fn new(paths: Box<[PathBuf]>) -> Self {
        let index = Self::last_index(&paths);

        Self { paths, index }
    }

    pub fn get_path(&self) -> Result<Option<&PathBuf>> {
        match self.index {
            None => Ok(None),
            Some(index) => self
                .paths
                .get(index)
                .ok_or_else(|| anyhow!("this should not be reachable. "))
                .map(Option::Some),
        }
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
            .index
            .map(|i| format!("{} / {}", i + 1, self.paths.len()))
            .unwrap_or_else(|| String::from("no recipes found"));

        write!(f, "{str}")
    }
}
