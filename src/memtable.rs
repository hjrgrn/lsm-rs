#![warn(missing_docs)]

//! TODO:

use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};

use crate::tombstone::Tombstone;

/// Backing storage.
// TODO: A trait BackingStorage that generalizes operations of lookup, insertion, deletion
pub struct MemTable<
    K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>,
    V: Tombstone + Clone,
> {
    data: HashMap<K, V>,
}

impl<
    K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>,
    V: Tombstone + Clone,
> MemTable<K, V>
{
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        self.data.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let val = self.data.get(key)?;
        Some(val.clone())
    }

    pub fn data(&self) -> &HashMap<K, V> {
        &self.data
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}
