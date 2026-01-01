#![warn(missing_docs)]

//! TODO:

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
    V: Tombstone + Clone + Serialize + for<'de> Deserialize<'de>,
> {
    memtable: MemTable<K, V>,
}

// TODO: error
impl<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Tombstone + Clone + Serialize + for<'de> Deserialize<'de>,
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
}

struct SSTable {
    path: PathBuf,
}

impl SSTable {
    pub fn write_sstable<
        K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>,
        V: Tombstone + Clone + Serialize + for<'de> Deserialize<'de>,
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
        V: Tombstone + Clone + Serialize + for<'de> Deserialize<'de>,
    >(
        &self,
        key: K,
    ) -> Result<Option<V>, io::Error> {
        let mut reader = BufReader::new(File::open(&self.path)?);
        let data_stream =
            serde_json::Deserializer::from_reader(&mut reader).into_iter::<KeyValue<K, V>>();
        let mut previous_element: Option<KeyValue<K, V>> = None;
        for element in data_stream {
            let element = element?;
            if element.key > key {
                match previous_element {
                    Some(e) => return Ok(Some(e.value)),
                    None => {}
                }
                break;
            } else if element.key == key {
                if element.value.is_tombstone() {
                    break;
                }
                previous_element = Some(element);
            }
        }
        Ok(None)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct KeyValue<K, V> {
    key: K,
    value: V,
}

trait Tombstone {
    fn is_tombstone(&self) -> bool;
}

impl Tombstone for &str {
    fn is_tombstone(&self) -> bool {
        // TODO: proper tombstone
        *self == "TODO"
    }
}

impl Tombstone for String {
    fn is_tombstone(&self) -> bool {
        // TODO: proper tombstone
        *self == "TODO"
    }
}
