use std::{fs::read_dir, path::PathBuf};

use anyhow::Result as AnyResult;
use csv::Reader;
use fake::{Fake, faker::lorem::en::Word};
use regex::Regex;
use tempfile::TempDir;

use lsm_tree_rs::{database::Database, sstable::KeyValue};

const WALL_NAME: &str = "db.wal";
const MANIFEST_NAME: &str = "MANIFEST";

pub struct TestApp {
    pub db: Database<usize, String>,
    pub wal_path: PathBuf,
    pub key_index: usize,
    pub manifest_path: PathBuf,
    working_dir: TempDir,
    pub table: Vec<(usize, String)>,
    pub table_cursor: usize,
    pub sstables: Vec<PathBuf>,
}

impl TestApp {
    pub fn build(max_memtable_size: usize, compaction_threshold: usize) -> AnyResult<Self> {
        let working_dir = tempfile::tempdir().unwrap();
        let working_dir_path = working_dir.path().to_path_buf();
        let wal_path = working_dir.path().join(WALL_NAME);
        let manifest_path = working_dir.path().join(MANIFEST_NAME);
        let db: Database<usize, String> = Database::build(
            working_dir_path,
            max_memtable_size,
            &wal_path,
            &manifest_path,
            compaction_threshold,
        )
        .unwrap();
        Ok(Self {
            db,
            wal_path,
            key_index: 0,
            manifest_path,
            working_dir,
            table: Vec::new(),
            table_cursor: 0,
            sstables: Vec::new(),
        })
    }

    pub fn working_dir(&self) -> PathBuf {
        self.working_dir.path().to_path_buf()
    }

    // Create a Vec of key/value pairs and populate the database with it.
    // The pairs with the smaller key have been created earlier.
    // Assert every entry has been `put` correctly into the database.
    pub fn populate_database(&mut self, n: usize) {
        for i in self.key_index..self.key_index + n {
            let val: String = Word().fake();
            self.table.push((i, val.clone()));
            let res = self.db.put(i, val);
            assert!(res.is_ok());
            self.key_index += 1;
        }
    }

    /// Gather every sstable file created.
    pub fn gather_and_sort_sstables(&mut self) {
        // Gather every sstable file created.
        let entries = read_dir(self.working_dir()).unwrap();
        let mut entries: Vec<_> = entries
            .filter_map(|e| {
                let entry_path = e.as_ref().unwrap().path();

                if entry_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with("sstable")
                {
                    Some(e.unwrap().path())
                } else {
                    None
                }
            })
            .collect();
        let rgx = Regex::new(r"^(?<index>[0-9]+)-data\.sstable$").unwrap();
        let extract_file_index = |file: &PathBuf| {
            let filename = file.file_name().unwrap();
            let filename = filename.to_str().unwrap();
            let c = rgx.captures(filename).unwrap();
            let index_file = &c["index"];
            let index_file: usize = index_file.parse().unwrap();
            index_file
        };
        // SSTables are stored in a Vec and ordered by creation time (via filename),
        // so earlier files appear in earlier SSTables.
        entries.sort_by(|file0, file1| {
            let index_file0 = extract_file_index(file0);
            let index_file1 = extract_file_index(file1);
            index_file0.cmp(&index_file1)
        });
        self.sstables = entries;
    }

    /// Uses `self.table_cursor` to remove the first added element from `self.db`.
    /// Does not delete the element from `self.table`.
    pub fn dequeue_db(&mut self) -> Result<KeyValue<usize, String>, anyhow::Error> {
        let (k, v) = self
            .table
            .get(self.table_cursor)
            .ok_or(anyhow::anyhow!("No elements in the table."))?;
        self.table_cursor += 1;

        let value = self.db.delete(*k)?.ok_or(anyhow::anyhow!(
            "No elements in the table (this should not happen)."
        ))?;

        // Just to be safe.
        assert_eq!(v, &value);

        Ok(KeyValue {
            key: *k,
            value: Some(v.clone()),
        })
    }

    /// Test entries are written correctly in the manifest.
    pub fn test_manifest_entries(&mut self, added_elements: usize, entries: usize) {
        // Add multiple sstables.
        self.populate_database(added_elements);
        let mut reader = Reader::from_path(&self.manifest_path).unwrap();
        let mut reader = reader.deserialize::<PathBuf>();
        let mut i = 0;
        for path in &mut reader {
            let file_name = format!("{}-data.sstable", i);
            let supposed_path = self.working_dir().join(file_name);
            // Assert `supposed_path` is one of the sstables.
            assert!(self.sstables.contains(&supposed_path));
            // Assert sstables are added to the manifest.
            assert_eq!(path.unwrap(), supposed_path);
            i += 1;
        }
        // Assert no more sstables have been written.
        assert!(reader.next().is_none());
        // Assert every sstable has been added.
        assert_eq!(i, entries);
    }
}

/// Assert a provided entry obtained from deserializing an sstable against `self.table`.
pub fn assert_entry(kv: Result<KeyValue<usize, String>, csv::Error>, app: &TestApp, i: &mut usize) {
    let kv = kv.unwrap();
    let (key, value) = &app.table[*i];
    let k = kv.key;
    assert_eq!(&k, key);
    assert_eq!(&kv.value.unwrap(), value);
    assert_eq!(k, *i);
    *i += 1;
}
