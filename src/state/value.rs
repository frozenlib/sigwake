use crate::{StateContext, StateKey};

pub struct Value<T> {
    value: T,
    key: StateKey,
}

impl<T> Value<T> {
    pub fn new(value: T, cx: &mut StateContext) -> Self {
        let key = StateKey::new(cx);
        Self { value, key }
    }
    pub fn get(&self, cx: &mut StateContext) -> &T {
        self.key.watch(cx);
        &self.value
    }
    pub fn get_mut(&mut self, cx: &mut StateContext) -> &mut T {
        self.key.notify(cx);
        self.key.watch(cx);
        &mut self.value
    }
    pub fn set(&mut self, value: T, cx: &mut StateContext) {
        self.key.notify(cx);
        self.value = value;
    }
}
