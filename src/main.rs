#![warn(missing_docs)]

use std::{collections::HashMap, hash::Hash};

fn main() {
    let mut db: Database<String, String> = Database::build().unwrap();
    let _ = db.put("a".to_string(), "apple".to_string());
    let g = db.get("a".to_string()).unwrap();
    println!("{}", g);
    // let d = db.delete("a".to_string()).unwrap();
}

struct Database<K: PartialEq + Eq + Hash + Clone, V: Clone> {
    memtable: MemTable<K, V>,
}

// TODO: error
impl<K: PartialEq + Eq + Hash + Clone, V: Clone> Database<K, V> {
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
struct MemTable<K: PartialEq + Eq + Hash + Clone, V: Clone> {
    data: HashMap<K, V>,
}

impl<K: PartialEq + Eq + Hash + Clone, V: Clone> MemTable<K, V> {
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
