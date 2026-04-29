//! XXX:

use std::{fmt::Debug, hash::Hash, io, path::PathBuf};

use csv::{Reader, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::memtable::MemTable;

pub struct SSTable {
    // TODO: we may want to have a method that gives back the data_stream, instead of making path
    // public.
    pub path: PathBuf,
}

impl SSTable {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn write_sstable<
        K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> + Debug,
        V: Clone + Serialize + for<'de> Deserialize<'de> + Debug,
    >(
        &self,
        mem_table: &MemTable<K, V>,
    ) -> Result<(), io::Error> {
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_path(&self.path)?;
        let mut sorted: Vec<_> = mem_table.data().iter().collect();
        sorted.sort_by_key(|e| e.0);
        writer.write_record(&["key", "value"])?;
        for record in &sorted {
            writer.serialize(record)?;
        }
        writer.flush()?;

        Ok(())
    }

    pub fn get<
        K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> + Debug,
        V: Clone + Serialize + for<'de> Deserialize<'de> + Debug,
    >(
        &self,
        key: K,
    ) -> Result<Option<V>, io::Error> {
        // TODO: maybe we will have a public method that returns the stream.
        let mut reader = Reader::from_path(&self.path)?;
        for record in reader.deserialize::<KeyValue<K, V>>() {
            let record = record?;
            if record.key > key {
                break;
            } else if record.key == key {
                return Ok(record.value);
            }
        }
        Ok(None)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyValue<K, V> {
    pub key: K,
    pub value: Option<V>,
}
