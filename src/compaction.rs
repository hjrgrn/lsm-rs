//! XXX:

use std::{
    collections::HashMap,
    fs::{File, rename},
    hash::Hash,
    io::{self, BufReader, BufWriter},
};

use serde::{Deserialize, Serialize};
use serde_json::{StreamDeserializer, de::IoRead};

use crate::sstable::KeyValue;

pub struct MiniMemTab<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Clone + Serialize + for<'de> Deserialize<'de>,
> {
    data: HashMap<K, (usize, Option<V>)>,
    path: String,
    tmp_path: String,
    writer: BufWriter<File>,
}

impl<
    K: Serialize + for<'de> Deserialize<'de> + Ord + PartialEq + Eq + Hash + Clone,
    V: Clone + Serialize + for<'de> Deserialize<'de>,
> MiniMemTab<K, V>
{
    // TODO: PathBuf instead of str
    pub fn build(path: &str) -> Result<Self, io::Error> {
        let tmp_path = format!("{path}.tmp");
        let writer = BufWriter::new(File::create(&tmp_path)?);
        Ok(Self {
            data: HashMap::new(),
            path: path.to_string(),
            tmp_path,
            writer,
        })
    }

    // TODO: return Ok(false) if iterator is empty, not the best design, have a custom error
    // enum, one variant for Empty, another for io::Error
    pub fn insert(
        &mut self,
        index: usize,
        iterator: &mut StreamDeserializer<'_, IoRead<BufReader<File>>, KeyValue<K, V>>,
    ) -> Result<bool, io::Error> {
        let pair = iterator.next();
        let pair = if let Some(p) = pair {
            p?
        } else {
            return Ok(false);
        };
        self.data.insert(pair.key, (index, pair.value));
        Ok(true)
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
        let index = pair.1.0.clone();
        if value.is_some() {
            let pair = KeyValue {
                key: pair.1,
                value: value,
            };
            serde_json::to_writer(&mut self.writer, &pair)?;
        }
        self.data.remove(&key);

        Ok(Some(index))
    }

    pub fn save_file(&self) -> io::Result<()> {
        rename(&self.tmp_path, &self.path)
    }
}
