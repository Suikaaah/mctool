use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

pub struct Recipes {
    recipes: Box<[PathBuf]>,
    index: Option<usize>,
}

impl Recipes {
    pub const fn new(recipes: Box<[PathBuf]>) -> Self {
        let index = Self::last_index(&recipes);

        Self { recipes, index }
    }

    pub fn get(&self) -> Option<&PathBuf> {
        self.index.map(|i| {
            self.recipes
                .get(i)
                .expect("this should be unreachable. check your code")
        })
    }

    pub const fn increment(&mut self) {
        if let Some(index) = &mut self.index {
            *index = if *index + 1 == self.recipes.len() {
                0
            } else {
                *index + 1
            };
        }
    }

    pub const fn decrement(&mut self) {
        if let Some(index) = &mut self.index {
            *index = if *index == 0 {
                self.recipes.len() - 1
            } else {
                *index - 1
            };
        }
    }

    const fn last_index<T>(value: &[T]) -> Option<usize> {
        if value.is_empty() {
            None
        } else {
            Some(value.len() - 1)
        }
    }
}

impl Display for Recipes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = self
            .index
            .map(|i| format!("{} of {}", i + 1, self.recipes.len()))
            .unwrap_or_else(|| String::from("NO RECIPES"));

        write!(f, "{str}")
    }
}
