//! XXX:

use anyhow::Result as AnyResult;
use std::{
    fs,
    io::{BufReader, BufWriter, ErrorKind},
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

    pub(crate) fn write_manifest(&self, path: impl AsRef<Path>) -> AnyResult<()> {
        let tmp_path = path.as_ref().to_path_buf().join(".tmp");
        let f = fs::File::create(&tmp_path)?;
        let writer = BufWriter::new(&f);
        serde_json::to_writer(writer, &self.sstable_paths)?;
        // TODO: explain atomic rename
        fs::rename(&tmp_path, path)?;

        Ok(())
    }
}
