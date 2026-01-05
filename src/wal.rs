//! Write Ahead Log
//! XXX:

use std::{
    fs::File,
    io::{self, BufWriter},
    path::PathBuf,
};

use serde::Serialize;

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
}
