use std::ops::{Index, IndexMut};

use derive_ex::Ex;

#[derive(Debug, Clone, Ex)]
#[derive_ex(Default(bound(T)))]
#[default(Self::new())]
pub struct InfVec<T> {
    items: Vec<T>,
    default_value: T,
}

impl<T: Default> InfVec<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            default_value: Default::default(),
        }
    }
    pub fn clear(&mut self) {
        self.items.clear();
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }
}
impl<T: Default> Index<usize> for InfVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        if let Some(item) = self.items.get(index) {
            item
        } else {
            &self.default_value
        }
    }
}
impl<T: Default> IndexMut<usize> for InfVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if self.items.len() <= index {
            self.items.resize_with(index + 1, || Default::default());
        }
        &mut self.items[index]
    }
}

impl<'a, T> IntoIterator for &'a InfVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
