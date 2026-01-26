#![warn(missing_docs)]

//! TODO:

use anyhow::Result as AnyResult;
use serde::{Deserialize, Serialize};
use std::{
    hash::Hash,
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    manifest::Manifest, memtable::MemTable, sstable::SSTable, tombstone::Tombstone, wal::WAL,
};

pub struct Database<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Tombstone + Clone + Serialize + for<'de> Deserialize<'de>,
> {
    memtable: MemTable<K, V>,
    max_memtable_size: usize,
    memtable_size: usize,
    sstables: Vec<SSTable>,
    sstable_counter: usize,
    wal: WAL,
    manifest: Manifest,
}

// TODO: error
impl<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Tombstone + Clone + Serialize + for<'de> Deserialize<'de>,
> Database<K, V>
{
    pub fn build(
        max_memtable_size: usize,
        wal_path: impl AsRef<Path>,
        manifest_path: impl AsRef<Path>,
    ) -> AnyResult<Self> {
        let tab = WAL::replay_wal::<K, V>(&wal_path)?;
        Ok(Self {
            memtable_size: tab.size(),
            memtable: tab,
            max_memtable_size,
            sstables: Vec::new(),
            sstable_counter: 0,
            wal: WAL::build(&wal_path)?,
            manifest: Manifest::read_manifest(manifest_path)?,
        })
    }

    pub fn put(&mut self, key: K, value: V) -> AnyResult<Option<V>> {
        self.wal.write(key.clone(), value.clone())?;
        let val = self.memtable.put(key, value);
        self.memtable_size += 1;
        if self.memtable_size > self.max_memtable_size {
            self.flush_memtable()?;
        }
        Ok(val)
    }

    pub fn get(&self, key: K) -> Result<Option<V>, io::Error> {
        if let Some(v) = self.memtable.get(&key) {
            return Ok(Some(v));
        }
        // TODO: explain
        for i in (0..self.sstable_counter).rev() {
            let tab = &self.sstables[i];
            // TODO: remove memcopy
            let res = tab.get(key.clone());
            if res.is_ok() {
                return res;
            } else {
                return res;
            }
        }
        Ok(None)
    }

    pub fn delete(&self, _key: K) -> Result<V, String> {
        Err("todo".to_string())
    }

    // TODO: error handling
    fn flush_memtable(&mut self) -> AnyResult<()> {
        let sstable_path = format!("data-{}.sstable", self.sstable_counter);
        let sstable = SSTable::new(PathBuf::from_str(&sstable_path)?);
        sstable.write_sstable(&self.memtable)?;
        self.sstable_counter += 1;
        self.sstables.push(sstable);
        self.manifest.add_sstable(sstable_path);
        self.format_memtable();
        Ok(())
    }

    fn format_memtable(&mut self) {
        self.memtable = MemTable::new();
        self.memtable_size = 0;
    }
}
