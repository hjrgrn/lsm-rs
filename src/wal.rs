//! Write Ahead Log
//! XXX:

use std::{
    fs::{File, remove_file},
    hash::Hash,
    io::{self, BufReader, BufWriter, ErrorKind},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{memtable::MemTable, tombstone::Tombstone};

pub struct WAL {
    path: PathBuf,
    writer: BufWriter<File>,
}

impl WAL {
    pub fn build(path: PathBuf) -> Result<Self, io::Error> {
        let writer = BufWriter::new(File::options().append(true).create(false).open(&path)?);
        Ok(Self { path, writer })
    }

    // TODO: maybe other trait bounds
    pub fn write<K: Serialize, V: Serialize>(&mut self, key: K, value: V) -> Result<(), io::Error> {
        serde_json::to_writer(&mut self.writer, &(key, value))?;
        Ok(())
    }

    pub fn replay_wal<
        K: Ord + PartialEq + Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de>,
        V: Tombstone + Clone + Serialize + for<'de> Deserialize<'de>,
    >(
        path: &PathBuf,
    ) -> Result<MemTable<K, V>, io::Error> {
        let mut tab = MemTable::new();
        match File::open(path) {
            Ok(f) => {
                let mut reader = BufReader::new(&f);
                let data: Vec<(K, V)> = serde_json::from_reader(&mut reader)?;
                for (k, v) in data.into_iter() {
                    let _ = tab.put(k, v);
                }
                drop(f);
                remove_file(path)?;
                Ok(tab)
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    Ok(tab)
                } else {
                    Err(e)
                }
            }
        }
    }
}
