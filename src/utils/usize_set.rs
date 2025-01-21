use std::mem;

use derive_ex::Ex;

use crate::utils::inf_vec::InfVec;

#[derive(Debug, Clone, Ex)]
#[derive_ex(Default)]
#[default(Self::new())]
pub struct USizeSet {
    bitmap: InfVec<bool>,
    values: Vec<usize>,
}
impl USizeSet {
    pub fn new() -> Self {
        Self {
            bitmap: InfVec::new(),
            values: Vec::new(),
        }
    }
    pub fn insert(&mut self, value: usize) {
        if !mem::replace(&mut self.bitmap[value], true) {
            self.values.push(value);
        }
    }
    pub fn clear(&mut self) {
        self.values.clear();
        self.bitmap.clear();
    }
    pub fn iter(&self) -> USizeSetIter<'_> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a USizeSet {
    type Item = usize;
    type IntoIter = USizeSetIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        USizeSetIter {
            source: self,
            index: 0,
        }
    }
}

pub struct USizeSetIter<'a> {
    source: &'a USizeSet,
    index: usize,
}
impl Iterator for USizeSetIter<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let value = *self.source.values.get(self.index)?;
        self.index += 1;
        Some(value)
    }
}
