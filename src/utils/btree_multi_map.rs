use std::collections::{BTreeMap, btree_map};

#[cfg(test)]
mod tests;

pub struct BTreeMultiMap<K, V> {
    entries: BTreeMap<(K, usize), V>,
    ids: BTreeMap<K, usize>,
}

impl<K, V> BTreeMultiMap<K, V>
where
    K: Ord + Copy,
{
    pub const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            ids: BTreeMap::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn insert(&mut self, key: K, value: V) -> usize {
        let mut e_id = self.ids.entry(key);
        let mut need_save_id = false;
        let mut id = match &mut e_id {
            btree_map::Entry::Vacant(_) => 0,
            btree_map::Entry::Occupied(e) => {
                need_save_id = true;
                e.get().wrapping_add(1)
            }
        };

        loop {
            match self.entries.entry((key, id)) {
                btree_map::Entry::Vacant(e) => {
                    e.insert(value);
                    if need_save_id {
                        e_id.and_modify(|x| *x = id).or_insert(id);
                    }
                    return id;
                }
                btree_map::Entry::Occupied(_) => {
                    id = id.wrapping_add(1);
                    need_save_id = true;
                }
            }
        }
    }
    pub fn remove(&mut self, key: K, id: usize) -> Option<V> {
        let ret = self.entries.remove(&(key, id));
        if self.first_key() != Some(&key) {
            self.ids.remove(&key);
        }
        ret
    }
    pub fn first_key(&self) -> Option<&K> {
        Some(&self.entries.first_key_value()?.0.0)
    }
    pub fn first_entry(&mut self) -> Option<btree_map::OccupiedEntry<(K, usize), V>> {
        self.entries.first_entry()
    }
}
