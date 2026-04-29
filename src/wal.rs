//! Write Ahead Log
//! XXX:

use std::io::ErrorKind as IoErrorKind;
use std::{
    fmt::Debug,
    fs::{File, remove_file},
    hash::Hash,
    io,
    path::Path,
};

use csv::{ErrorKind, Reader, Writer, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::memtable::MemTable;
use crate::sstable::KeyValue;

pub struct WAL {
    // path: PathBuf,
    writer: Writer<File>,
}

impl WAL {
    pub fn build(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let mut writer = WriterBuilder::new().has_headers(true).from_path(&path)?;
        writer.write_record(&["key", "value"])?;
        writer.flush()?;
        Ok(Self { writer })
    }

    // TODO: maybe other trait bounds
    pub fn write<K: Serialize, V: Serialize>(
        &mut self,
        key: K,
        value: Option<V>,
    ) -> Result<(), csv::Error> {
        self.writer.serialize((&key, &value))?;
        self.writer.flush().map_err(|e| e.into())
    }

    pub fn replay_wal<
        K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> + Debug,
        V: Clone + Serialize + for<'de> Deserialize<'de> + Debug,
    >(
        path: impl AsRef<Path>,
    ) -> Result<MemTable<K, V>, csv::Error> {
        let mut tab = MemTable::new();

        match Reader::from_path(&path) {
            Ok(mut reader) => {
                for kv in reader.deserialize::<KeyValue<K, V>>() {
                    let kv = kv?;
                    let _ = tab.put(kv.key, kv.value);
                }
            }
            Err(e) => {
                if let ErrorKind::Io(err) = e.kind() {
                    if let IoErrorKind::NotFound = err.kind() {
                        return Ok(tab);
                    }
                } else {
                    return Err(e);
                }
            }
        };
        remove_file(&path)?;
        Ok(tab)
    }
}
