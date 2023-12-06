use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

pub struct Literals {
    pub names: Vec<String>,
    pub name_to_idx: HashMap<String, usize>,
}

impl Literals {
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            name_to_idx: HashMap::new(),
        }
    }

    pub fn new_with_literals(literals: &[&str]) -> Self {
        let (names, name_to_idx) = literals
            .iter()
            .enumerate()
            .map(|(idx, name)| (name.to_string(), (name.to_string(), idx)))
            .unzip();
        Self { names, name_to_idx }
    }

    /// Returns the index of the literal with the given name. If the literal does not exist, adds it and returns the new index.
    pub fn get_or_add_index(&mut self, name: &str) -> usize {
        if let Some(idx) = self.name_to_idx.get(name) {
            *idx
        } else {
            let idx = self.names.len();
            self.names.push(name.to_string());
            self.name_to_idx.insert(name.to_string(), idx);
            idx
        }
    }

    /// Returns the index of the literal with the given name. If the literal does not exist, returns None.
    pub fn get_index(&self, name: &str) -> Option<usize> {
        self.name_to_idx.get(name).copied()
    }
}

impl Index<usize> for Literals {
    type Output = str;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.names[idx]
    }
}

impl IndexMut<usize> for Literals {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.names[idx]
    }
}
