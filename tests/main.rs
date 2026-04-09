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

#[test]
pub fn put_get_and_delete_work_correctly_in_memory() {
    let key = "key".to_string();
    let value = "value".to_string();

    let wroking_dir = tempfile::tempdir().unwrap();
    let wal_path = wroking_dir.path().join(WALL_NAME);
    let manifest_path = wroking_dir.path().join(MANIFEST_NAME);
    let mut db: Database<String, String> =
        Database::build(10, &wal_path, &manifest_path, 10).unwrap();

    // Put a value and retrieve it.
    db.put(key.clone(), value.clone()).unwrap();
    let foo = db.get(key.clone()).unwrap().unwrap();
    assert_eq!(foo, value);

    // Delete existing value, the value is returned upon deletion.
    let foo = db.delete(key.clone()).unwrap().unwrap();
    assert_eq!(foo, value);
    // Cannot retrieve deleted value.
    let none = db.get(key.clone()).unwrap();
    assert!(none.is_none());

    // Deleting a non existing value returns None for "not found".
    let none = db.delete(key.clone()).unwrap();
    assert!(none.is_none());
}
