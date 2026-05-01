//! XXX:

use anyhow::Result as AnyResult;
use csv::{ErrorKind, Reader, WriterBuilder};
use std::io::ErrorKind as IoErrorKind;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) struct Manifest {
    path: PathBuf,
    // TODO: getter
    pub sstable_paths: Vec<PathBuf>,
}

impl Manifest {
    pub(crate) fn read_manifest(path: impl AsRef<Path>) -> AnyResult<Manifest> {
        let path = path.as_ref().to_path_buf();

        let mut reader = match Reader::from_path(&path) {
            Ok(r) => r,
            Err(e) => {
                if let ErrorKind::Io(err) = e.kind() {
                    if let IoErrorKind::NotFound = err.kind() {}
                    return Ok(Manifest {
                        path,
                        sstable_paths: Vec::new(),
                    });
                } else {
                    return Err(e.into());
                }
            }
        };

        let sstable_paths_res = reader.deserialize::<PathBuf>().collect::<Vec<_>>();
        let sstable_paths: Vec<PathBuf> = sstable_paths_res
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

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

        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_path(&tmp_path)?;
        // TODO: error handling: delete tmp file on failure, after dropping writer.
        writer.write_record(["SStablePaths"])?;
        for path in self.sstable_paths.iter() {
            writer.serialize(path)?;
        }
        writer.flush()?;

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
