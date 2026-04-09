use std::fs;

use lsm_tree_rs::database::Database;

const WALL_NAME: &str = "db.wal";
const MANIFEST_NAME: &str = "MANIFEST";

#[test]
pub fn assert_wall_is_created_correctly() {
    let wroking_dir = tempfile::tempdir().unwrap();
    let wal_path = wroking_dir.path().join(WALL_NAME);
    let manifest_path = wroking_dir.path().join(MANIFEST_NAME);
    let _: Database<String, String> = Database::build(10, &wal_path, &manifest_path, 10).unwrap();
    assert!(fs::exists(&wal_path).unwrap());
}
