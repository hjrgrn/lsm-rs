//! TODO:

use std::{collections::HashMap, fmt::Debug, hash::Hash};

use serde::{Deserialize, Serialize};

/// Backing storage.
// TODO: A trait BackingStorage that generalizes operations of lookup, insertion, deletion
// TODO: explain use of Option to signify TOMBSTONE
pub struct MemTable<
    K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> + Debug,
    V: Clone + Debug,
> {
    data: HashMap<K, Option<V>>,
}

impl<
    K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> + Debug,
    V: Clone + Debug,
> MemTable<K, V>
{
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        self.data.insert(key, Some(value))?
    }

    pub fn get(&self, key: K) -> Option<V> {
        let val = self.data.get(&key)?;
        val.clone()
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let val = self.data.insert(key, None)?;
        val
    }

    pub fn data(&self) -> &HashMap<K, Option<V>> {
        &self.data
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}
