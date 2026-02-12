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

use crate::{manifest::Manifest, memtable::MemTable, sstable::SSTable, wal::WAL};

pub struct Database<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Clone + Serialize + for<'de> Deserialize<'de>,
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
    V: Clone + Serialize + for<'de> Deserialize<'de>,
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
        self.add_element(key, Some(value))
    }

    pub fn get(&self, key: K) -> Result<Option<V>, io::Error> {
        // TODO: remove memcopy
        if let Some(v) = self.memtable.get(key.clone()) {
            return Ok(Some(v));
        }
        // TODO: explain
        for i in (0..self.sstable_counter).rev() {
            let tab = &self.sstables[i];
            let res = tab.get(key);
            if res.is_ok() {
                return res;
            } else {
                return res;
            }
        }
        Ok(None)
    }

    pub fn delete(&mut self, key: K) -> AnyResult<Option<V>> {
        self.add_element(key, None)
    }

    fn add_element(&mut self, key: K, value: Option<V>) -> AnyResult<Option<V>> {
        // TODO: explain why clone is necessary
        self.wal.write::<K, V>(key.clone(), value.clone())?;
        let val = match value {
            Some(e) => self.memtable.put(key, e),
            None => self.memtable.remove(key),
        };
        self.memtable_size += 1;
        if self.memtable_size > self.max_memtable_size {
            self.flush_memtable()?;
        }
        Ok(val)
    }

    // TODO: error handling
    fn flush_memtable(&mut self) -> AnyResult<()> {
        let sstable_path = format!("data-{}.sstable", self.sstable_counter);
        let sstable = SSTable::new(PathBuf::from_str(&sstable_path)?);
        sstable.write_sstable(&self.memtable)?;
        self.sstable_counter += 1;
        self.sstables.push(sstable);
        self.manifest.add_sstable(sstable_path);
        self.manifest.write_manifest()?;
        self.format_memtable();
        Ok(())
    }

    fn format_memtable(&mut self) {
        self.memtable = MemTable::new();
        self.memtable_size = 0;
    }
}
