//! Write Ahead Log
//! XXX:

use std::{
    fs::File,
    io::{self, BufWriter},
    path::PathBuf,
};

pub struct WAL {
    path: PathBuf,
    writer: BufWriter<File>,
}

impl WAL {
    pub fn build(path: PathBuf) -> Result<Self, io::Error> {
        let writer = BufWriter::new(File::options().append(true).create(false).open(&path)?);
        Ok(Self { path, writer })
    }
}
