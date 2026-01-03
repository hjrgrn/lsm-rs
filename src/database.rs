#![warn(missing_docs)]

//! TODO:

use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::{memtable::MemTable, sstable::SSTable, tombstone::Tombstone};

pub struct Database<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Tombstone + Clone + Serialize + for<'de> Deserialize<'de>,
> {
    memtable: MemTable<K, V>,
    max_memtable_size: usize,
    memtable_size: usize,
    sstables: Vec<SSTable>,
    sstable_counter: usize,
}

// TODO: error
impl<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Tombstone + Clone + Serialize + for<'de> Deserialize<'de>,
> Database<K, V>
{
    pub fn build(max_memtable_size: usize) -> Result<Self, String> {
        Ok(Self {
            memtable: MemTable::new(),
            max_memtable_size,
            memtable_size: 0,
            sstables: Vec::new(),
            sstable_counter: 0,
        })
    }

    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        self.memtable.put(key, value)
    }

    pub fn get(&self, key: K) -> Option<V> {
        self.memtable.get(&key)
    }

    pub fn delete(&self, _key: K) -> Result<V, String> {
        Err("todo".to_string())
    }
}
