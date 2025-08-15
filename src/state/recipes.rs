use std::path::PathBuf;

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

    pub fn increment(&mut self) {
        if let Some(index) = &mut self.index {
            *index = if *index + 1 == self.recipes.len() {
                0
            } else {
                *index + 1
            };
        }
    }

    pub fn decrement(&mut self) {
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
