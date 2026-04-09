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
    /// NOTE: Although this field is not used directly, it must be retained to
    /// prevent the temporary directory from being dropped prematurely.
    #[allow(unused)]
    wroking_dir: TempDir,
}

impl TestApp {
    pub(crate) fn build(max_memtable_size: usize, compaction_threshold: usize) -> AnyResult<Self> {
        let wroking_dir = tempfile::tempdir().unwrap();
        let wal_path = wroking_dir.path().join(WALL_NAME);
        let manifest_path = wroking_dir.path().join(MANIFEST_NAME);
        let db: Database<String, String> = Database::build(
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
            wroking_dir,
        })
    }
}
