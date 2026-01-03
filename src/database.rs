#![warn(missing_docs)]

//! TODO:

use std::{hash::Hash, path::PathBuf, str::FromStr};

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

    pub fn put(&mut self, key: K, value: V) -> Result<Option<V>, String> {
        let val = self.memtable.put(key, value);
        self.memtable_size += 1;
        if self.memtable_size > self.max_memtable_size {
            self.flush_memtable()?;
        }
        Ok(val)
    }

    pub fn get(&self, key: K) -> Option<V> {
        self.memtable.get(&key)
    }

    pub fn delete(&self, _key: K) -> Result<V, String> {
        Err("todo".to_string())
    }

    // TODO: error handling
    fn flush_memtable(&mut self) -> Result<(), String> {
        let sstable_path = format!("data-{}.sstable", self.sstable_counter);
        let sstable = SSTable::new(PathBuf::from_str(&sstable_path).unwrap());
        sstable.write_sstable(&self.memtable).unwrap();
        self.sstable_counter += 1;
        self.sstables.push(sstable);
        self.format_memtable();
        Ok(())
    }

    fn format_memtable(&mut self) {

    }
}
