//! XXX:

use anyhow::Result as AnyResult;
use std::{
    fs,
    io::{BufReader, ErrorKind},
    path::{Path, PathBuf},
};

pub(crate) struct Manifest {
    sstable_paths: Vec<PathBuf>,
}

impl Manifest {
    pub(crate) fn read_manifest(path: impl AsRef<Path>) -> AnyResult<Manifest> {
        let f = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                if let ErrorKind::NotFound = e.kind() {
                    return Ok(Manifest {
                        sstable_paths: Vec::new(),
                    });
                } else {
                    return Err(e.into());
                }
            }
        };

        let mut reader = BufReader::new(&f);
        let sstable_paths: Vec<PathBuf> = serde_json::from_reader(&mut reader)?;
        Ok(Manifest { sstable_paths })
    }
}
