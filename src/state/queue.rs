use std::{collections::VecDeque, mem, task::Poll};

use derive_ex::Ex;

use crate::{StateContext, StateKey};

/// A queue for use in state type `St` of [`StateContainer<St>`](crate::StateContainer).
///
/// When the queue is empty and an attempt is made to retrieve a value,
/// the queue will register itself as a dependency in the context.
/// When items are added to an empty queue, it notifies its dependents.
#[derive(Debug)]
pub struct Queue<T> {
    items: VecDeque<T>,
    key: StateKey,
}
impl<T> Queue<T> {
    /// Creates a new empty queue.
    pub fn new(cx: &mut StateContext) -> Self {
        let key = StateKey::new(cx);
        Self {
            items: VecDeque::new(),
            key,
        }
    }

    /// Adds an item to the queue.
    ///
    /// If the queue was empty before pushing, notifies dependents that the state has changed.
    pub fn push(&mut self, item: T, cx: &mut StateContext) {
        if self.items.is_empty() {
            self.key.notify(cx);
        }
        self.items.push_back(item);
    }

    pub fn pop(&mut self, cx: &mut StateContext) -> Poll<T> {
        match self.items.pop_front() {
            Some(item) => Poll::Ready(item),
            None => {
                self.key.watch(cx);
                Poll::Pending
            }
        }
    }
}

#[derive(Debug, Ex)]
#[derive_ex(Default)]
pub struct QueueReader<T>(VecDeque<T>);

impl<T> QueueReader<T> {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
    pub fn fetch(&mut self, queue: &mut Queue<T>, cx: &mut StateContext) -> Poll<()> {
        if !self.0.is_empty() {
            Poll::Ready(())
        } else if !queue.items.is_empty() {
            mem::swap(&mut self.0, &mut queue.items);
            Poll::Ready(())
        } else {
            queue.key.watch(cx);
            Poll::Pending
        }
    }
}
impl<'a, T> IntoIterator for &'a mut QueueReader<T> {
    type Item = T;
    type IntoIter = QueueReaderIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        QueueReaderIter(self)
    }
}
pub struct QueueReaderIter<'a, T>(&'a mut QueueReader<T>);

impl<T> Iterator for QueueReaderIter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.0.pop_front()
    }
}
