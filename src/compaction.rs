//! XXX:

use std::{
    collections::HashMap,
    fmt::Debug,
    fs::{File, rename},
    hash::Hash,
    io,
    path::{Path, PathBuf},
};

use csv::Writer;
use serde::{Deserialize, Serialize};

use crate::sstable::KeyValue;

pub struct MiniMemTab<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone + Debug,
    V: Clone + Serialize + for<'de> Deserialize<'de> + Debug,
> {
    data: HashMap<K, (usize, Option<V>)>,
    path: PathBuf,
    tmp_path: PathBuf,
    writer: Writer<File>,
}

impl<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone + Debug,
    V: Clone + Serialize + for<'de> Deserialize<'de> + Debug,
> MiniMemTab<K, V>
{
    pub fn build(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let tmp_path = path.as_ref().with_added_extension("tmp");
        let mut writer = csv::WriterBuilder::new()
            .has_headers(true)
            .from_path(&tmp_path)?;
        writer.write_record(["key", "value"])?;
        Ok(Self {
            data: HashMap::new(),
            path: path.as_ref().to_path_buf(),
            tmp_path,
            writer,
        })
    }

    // TODO:
    pub fn insert(
        &mut self,
        index: usize,
        iterator: &mut impl Iterator<Item = Result<KeyValue<K, V>, csv::Error>>,
    ) -> Result<Option<()>, csv::Error> {
        let pair = match iterator.next() {
            Some(p) => p,
            None => {
                return Ok(None);
            }
        }?;
        self.data.insert(pair.key, (index, pair.value));
        Ok(Some(()))
    }

    // TODO: needs refactoring
    // TODO: return the index of the table that has the value that has been written, or None if the
    // mini_mem_tab is empty.
    // Maybe have a custom error enum, one variant for Empty, another for io::Error
    pub fn write_to_sstable(&mut self) -> Result<Option<usize>, io::Error> {
        let pair = self
            .data
            .iter()
            // TODO: explain: - because we want the one with the biggest index, despite the fact
            // that we are using `.min_by()`
            .min_by(|a, b| (a.0, -(a.1.0 as i8)).cmp(&(b.0, -(b.1.0 as i8))));
        let pair = if let Some(p) = pair {
            p
        } else {
            return Ok(None);
        };
        // TODO: handle the None value
        let key = pair.0.clone();
        let value = pair.1.1.clone();
        let index = pair.1.0;
        if value.is_some() {
            let pair = (&key, &value);
            self.writer.serialize(pair)?;
            self.writer.flush()?;
        }
        self.data.remove(&key);

        Ok(Some(index))
    }

    pub fn atomic_rename(&self) -> io::Result<()> {
        rename(&self.tmp_path, &self.path)
    }
}
