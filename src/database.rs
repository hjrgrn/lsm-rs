//! TODO:

use anyhow::Result as AnyResult;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    hash::Hash,
    io::{self, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    compaction::MiniMemTab,
    manifest::Manifest,
    memtable::MemTable,
    sstable::{KeyValue, SSTable},
    wal::WAL,
};

pub struct Database<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Clone + Serialize + for<'de> Deserialize<'de>,
> {
    working_dir: PathBuf,
    memtable: MemTable<K, V>,
    max_memtable_size: usize,
    memtable_size: usize,
    sstables: Vec<SSTable>,
    sstable_counter: usize,
    wal: WAL,
    manifest: Manifest,
    compaction_threshold: usize,
}

// TODO: error
impl<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Clone + Serialize + for<'de> Deserialize<'de>,
> Database<K, V>
{
    pub fn build(
        working_dir: impl AsRef<Path>,
        max_memtable_size: usize,
        wal_path: impl AsRef<Path>,
        manifest_path: impl AsRef<Path>,
        compaction_threshold: usize,
    ) -> AnyResult<Self> {
        let tab = WAL::replay_wal::<K, V>(&wal_path)?;
        Ok(Self {
            working_dir: working_dir.as_ref().to_path_buf(),
            memtable_size: tab.size(),
            memtable: tab,
            max_memtable_size,
            sstables: Vec::new(),
            sstable_counter: 0,
            wal: WAL::build(&wal_path)?,
            manifest: Manifest::read_manifest(manifest_path)?,
            compaction_threshold,
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

    // TODO: refactor this
    pub fn compact_sstables(&mut self) -> AnyResult<()> {
        // TODO: make it a pathbuf
        let new_sstable_path = format!("./instance/data-0.sstable");
        let mut mini_mem_tab: MiniMemTab<K, V> = MiniMemTab::build(&new_sstable_path)?;
        let mut tables = Vec::with_capacity(self.sstable_counter);
        let mut index = 0;
        for tab in self.sstables.iter() {
            let reader = BufReader::new(File::open(&tab.path)?);
            let mut table =
                serde_json::Deserializer::from_reader(reader).into_iter::<KeyValue<K, V>>();

            // Populate mini_mem_tab
            if mini_mem_tab.insert(index, &mut table)? {
                tables.push(table);
                index += 1;
            }
        }
        loop {
            let opt = mini_mem_tab.write_to_sstable()?;
            let index = if let Some(i) = opt {
                i
            } else {
                break;
            };

            let tab = &mut tables[index];
            let _ = mini_mem_tab.insert(index, tab)?;
        }

        // At this point we have a temporary file for the new sstable

        self.manifest.refresh_manifest(&new_sstable_path)?;
        for tab in &self.sstables {
            fs::remove_file(&tab.path)?;
        }
        self.sstables = Vec::new();
        let new_sstable_path = PathBuf::from_str(&new_sstable_path).unwrap();
        let new_sstable = SSTable::new(new_sstable_path);
        self.sstables.push(new_sstable);
        self.sstable_counter = 1;
        mini_mem_tab.atomic_rename().map_err(|e| anyhow::anyhow!(e))
    }

    fn add_element(&mut self, key: K, value: Option<V>) -> AnyResult<Option<V>> {
        // TODO: explain why clone is necessary
        self.wal.write::<K, V>(key.clone(), value.clone())?;
        let val = match value {
            Some(e) => self.memtable.put(key, e),
            None => self.memtable.remove(key),
        };
        self.memtable_size += 1;
        if self.memtable_size >= self.max_memtable_size {
            self.flush_memtable()?;
        }
        Ok(val)
    }

    // TODO: error handling
    fn flush_memtable(&mut self) -> AnyResult<()> {
        let sstable_path = self
            .working_dir
            .join(format!("data-{}.sstable", self.sstable_counter));
        let sstable = SSTable::new(sstable_path.clone());
        sstable.write_sstable(&self.memtable)?;
        self.sstable_counter += 1;
        self.sstables.push(sstable);
        self.manifest.add_sstable(sstable_path);
        self.manifest.write_manifest()?;
        self.format_memtable();

        if self.sstable_counter > self.compaction_threshold {
            self.compact_sstables()?;
        }
        Ok(())
    }

    fn format_memtable(&mut self) {
        self.memtable = MemTable::new();
        self.memtable_size = 0;
    }
}
