//! Write Ahead Log
//! XXX:

use std::{
    fs::File,
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
        let res = File::open(path);
        if let Err(e) = res {
            if e.kind() == ErrorKind::NotFound {
                return Ok(tab);
            } else {
                return Err(e);
            }
        }
        let mut reader = BufReader::new(File::open(path)?);
        let data: Vec<(K, V)> = serde_json::from_reader(&mut reader)?;
        for (k, v) in data.into_iter() {
            let _ = tab.put(k, v);
        }
        Ok(tab)
    }
}
