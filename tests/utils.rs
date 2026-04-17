use std::path::PathBuf;

use anyhow::Result as AnyResult;

use lsm_tree_rs::database::Database;
use tempfile::TempDir;

const WALL_NAME: &str = "db.wal";
const MANIFEST_NAME: &str = "MANIFEST";

pub(crate) struct TestApp {
    pub(crate) db: Database<String, String>,
    pub(crate) wal_path: PathBuf,
    pub(crate) manifest_path: PathBuf,
    working_dir: TempDir,
}

impl TestApp {
    pub(crate) fn build(max_memtable_size: usize, compaction_threshold: usize) -> AnyResult<Self> {
        let working_dir = tempfile::tempdir().unwrap();
        let working_dir_path = working_dir.path().to_path_buf();
        let wal_path = working_dir.path().join(WALL_NAME);
        let manifest_path = working_dir.path().join(MANIFEST_NAME);
        let db: Database<String, String> = Database::build(
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
            manifest_path,
            working_dir,
        })
    }
    pub(crate) fn working_dir(&self) -> PathBuf {
        self.working_dir.path().to_path_buf()
    }
}
