use std::{collections::VecDeque, iter::FusedIterator, marker::PhantomData};

use derive_ex::Ex;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct SharedQueueCursor<T> {
    age: usize,
    _phantom: PhantomData<T>,
}

#[derive(Debug, Ex)]
#[derive_ex(Default)]
#[default(Self::new())]
pub struct SharedQueue<T> {
    values: VecDeque<T>,
    ref_counts: VecDeque<usize>,
    age_base: usize,
}

impl<T> SharedQueue<T> {
    pub fn new() -> Self {
        Self {
            values: VecDeque::new(),
            ref_counts: vec![0].into(),
            age_base: 0,
        }
    }
    pub fn create_cursor(&mut self) -> SharedQueueCursor<T> {
        self.increment_ref_count(self.values.len());
        SharedQueueCursor {
            age: self.end_age(),
            _phantom: PhantomData,
        }
    }
    pub fn drop_cursor(&mut self, cursor: SharedQueueCursor<T>) {
        let index = self.age_to_index(cursor.age);
        self.decrement_ref_count(index);
    }
    fn end_age(&self) -> usize {
        self.age_base + self.values.len()
    }
    fn index_to_age(&self, index: usize) -> usize {
        self.age_base.wrapping_add(index)
    }
    fn age_to_index(&self, age: usize) -> usize {
        age.wrapping_sub(self.age_base)
    }
    fn increment_ref_count(&mut self, index: usize) {
        let ref_count = &mut self.ref_counts[index];
        assert!(*ref_count < usize::MAX, "ref_count is MAX");
        *ref_count += 1;
    }
    fn decrement_ref_count(&mut self, index: usize) {
        let ref_count = &mut self.ref_counts[index];
        assert!(*ref_count > 0, "ref_count is 0");
        *ref_count -= 1;

        while let Some(&ref_count) = self.ref_counts.front() {
            if self.values.is_empty() || ref_count > 0 {
                break;
            }
            self.values.pop_front();
            self.ref_counts.pop_front();
            self.age_base = self.age_base.wrapping_add(1);
        }
    }
    pub fn reserve(&mut self, additional: usize) {
        self.values.reserve(additional);
        self.ref_counts.reserve(additional);
    }

    pub fn push(&mut self, value: T) {
        if self.values.is_empty() && self.ref_counts[0] == 0 {
            return;
        }
        self.values.push_back(value);
        self.ref_counts.push_back(0);
    }

    pub fn read<'a>(
        &'a mut self,
        cursor: &'a mut SharedQueueCursor<T>,
    ) -> SharedQueueReader<'a, T> {
        let index = self.age_to_index(cursor.age);
        SharedQueueReader {
            index_old: index,
            index,
            cursor,
            queue: self,
        }
    }
}
impl<T> Extend<T> for SharedQueue<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let size_hint = iter.size_hint();
        self.reserve(size_hint.0);
        for value in iter {
            self.push(value);
        }
    }
}

pub struct SharedQueueReader<'a, T> {
    index_old: usize,
    index: usize,
    cursor: &'a mut SharedQueueCursor<T>,
    queue: &'a mut SharedQueue<T>,
}
impl<T> SharedQueueReader<'_, T> {
    pub fn pop(&mut self) -> Option<&T> {
        let value = self.queue.values.get(self.index)?;
        self.index += 1;
        Some(value)
    }
    pub fn iter(&mut self) -> SharedQueueIter<'_, T> {
        self.into_iter()
    }
}
impl<T> Drop for SharedQueueReader<'_, T> {
    fn drop(&mut self) {
        self.cursor.age = self.queue.index_to_age(self.index);
        self.queue.increment_ref_count(self.index);
        self.queue.decrement_ref_count(self.index_old);
    }
}
impl<'a, T> IntoIterator for &'a mut SharedQueueReader<'_, T> {
    type Item = &'a T;
    type IntoIter = SharedQueueIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        SharedQueueIter {
            index: &mut self.index,
            values: &self.queue.values,
        }
    }
}
pub struct SharedQueueIter<'a, T> {
    index: &'a mut usize,
    values: &'a VecDeque<T>,
}
impl<'a, T> Iterator for SharedQueueIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.values.get(*self.index)?;
        *self.index += 1;
        Some(value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.values.len() - *self.index;
        (len, Some(len))
    }
}
impl<T> ExactSizeIterator for SharedQueueIter<'_, T> {}
impl<T> FusedIterator for SharedQueueIter<'_, T> {}
