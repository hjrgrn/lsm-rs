//! XXX:

use anyhow::Result as AnyResult;
use std::{
    fs,
    io::{BufReader, BufWriter, ErrorKind},
    path::{Path, PathBuf},
};

pub(crate) struct Manifest {
    path: PathBuf,
    sstable_paths: Vec<PathBuf>,
}

impl Manifest {
    pub(crate) fn read_manifest(path: impl AsRef<Path>) -> AnyResult<Manifest> {
        let path = path.as_ref().to_path_buf();
        let f = match fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                if let ErrorKind::NotFound = e.kind() {
                    return Ok(Manifest {
                        path,
                        sstable_paths: Vec::new(),
                    });
                } else {
                    return Err(e.into());
                }
            }
        };

        let mut reader = BufReader::new(&f);
        let sstable_paths: Vec<PathBuf> = serde_json::from_reader(&mut reader)?;
        Ok(Manifest {
            path,
            sstable_paths,
        })
    }

    pub(crate) fn write_manifest(&self) -> AnyResult<()> {
        // TODO: use crate tempfile
        let mut tmp_path = self.path.clone();
        let success = tmp_path.add_extension("tmp");
        if !success {
            return Err(anyhow::anyhow!(
                "Problems with tmp_path in Manifest::write_manifest."
            ));
        }
        let f = fs::File::create(&tmp_path)?;
        let writer = BufWriter::new(&f);
        if let Err(e) = serde_json::to_writer(writer, &self.sstable_paths) {
            fs::remove_file(tmp_path)?;
            return Err(e.into());
        }
        // TODO: explain atomic rename
        fs::rename(&tmp_path, &self.path)?;

        Ok(())
    }

    pub(crate) fn add_sstable(&mut self, path: impl AsRef<Path>) {
        self.sstable_paths.push(path.as_ref().to_path_buf());
    }

    pub(crate) fn refresh_manifest(&mut self, path: impl AsRef<Path>) -> AnyResult<()> {
        self.sstable_paths = Vec::new();
        self.add_sstable(path);
        self.write_manifest()
    }
}
