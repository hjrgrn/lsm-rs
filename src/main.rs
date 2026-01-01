#![warn(missing_docs)]

use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

fn main() {
    let mut db: Database<String, String> = Database::build().unwrap();
    let _ = db.put("a".to_string(), "apple".to_string());
    let g = db.get("a".to_string()).unwrap();
    println!("{}", g);
    // let d = db.delete("a".to_string()).unwrap();
}

struct Database<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Clone + Serialize + for<'de> Deserialize<'de>,
> {
    memtable: MemTable<K, V>,
}

// TODO: error
impl<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Clone + Serialize + for<'de> Deserialize<'de>,
> Database<K, V>
{
    pub fn build() -> Result<Self, String> {
        Ok(Self {
            memtable: MemTable::new(),
        })
    }

    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        self.memtable.put(key, value)
    }

    pub fn get(&self, key: K) -> Option<V> {
        self.memtable.get(&key)
    }

    pub fn delete(&self, key: K) -> Result<V, String> {
        Err("todo".to_string())
    }
}

/// Backing storage.
// TODO: A trait BackingStorage that generalizes operations of lookup, insertion, deletion
struct MemTable<
    K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>,
    V: Clone,
> {
    data: HashMap<K, V>,
}

impl<K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>, V: Clone>
    MemTable<K, V>
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
}

struct SSTable {
    path: PathBuf,
}

impl SSTable {
    pub fn write_sstable<
        K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>,
        V: Clone + Serialize + for<'de> Deserialize<'de>,
    >(
        &self,
        mem_table: MemTable<K, V>,
    ) -> Result<(), io::Error> {
        let mut f = BufWriter::new(File::create(&self.path)?);
        let mut sorted: Vec<_> = mem_table.data.iter().collect();
        sorted.sort_by_key(|e| e.0);
        serde_json::to_writer(&mut f, &sorted)?;

        Ok(())
    }

    pub fn get<
        K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>,
        V: Clone + Serialize + for<'de> Deserialize<'de>,
    >(
        &self,
        key: K,
    ) -> Result<Option<V>, io::Error> {
        let mut reader = BufReader::new(File::open(&self.path)?);
        let data: HashMap<K, V> = serde_json::from_reader(&mut reader)?;
        Ok(data.get(&key).cloned())
    }
}
