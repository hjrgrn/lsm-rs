//! XXX:

use std::{
    fs::File,
    hash::Hash,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::memtable::MemTable;

pub struct SSTable {
    path: PathBuf,
    // TODO: store BufReader and BufWriter, instead of opening a new one every time.
}

impl SSTable {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn write_sstable<
        K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>,
        V: Clone + Serialize + for<'de> Deserialize<'de>,
    >(
        &self,
        mem_table: &MemTable<K, V>,
    ) -> Result<(), io::Error> {
        let mut f = BufWriter::new(File::create(&self.path)?);
        let mut sorted: Vec<_> = mem_table.data().iter().collect();
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
        // TODO: reader may become a field
        let mut reader = BufReader::new(File::open(&self.path)?);
        let data_stream =
            serde_json::Deserializer::from_reader(&mut reader).into_iter::<KeyValue<K, V>>();
        for element in data_stream {
            let element = element?;
            if element.key > key {
                break;
            } else if element.key == key {
                return Ok(element.value);
            }
        }
        Ok(None)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct KeyValue<K, V> {
    key: K,
    value: Option<V>,
}
