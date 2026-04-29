use std::fs;

use csv::Reader;
use lsm_tree_rs::sstable::KeyValue;

use crate::utils::{TestApp, assert_entry};

mod utils;

#[test]
pub fn assert_wall_is_created_correctly() {
    let app = TestApp::build(10, 10).unwrap();
    assert!(fs::exists(&app.wal_path).unwrap());
}

#[test]
pub fn put_get_and_delete_work_correctly_in_memory() {
    let key = 1;
    let value = "value".to_string();

    let mut app = TestApp::build(10, 10).unwrap();

    // Put a value and retrieve it.
    app.db.put(key, value.clone()).unwrap();
    let foo = app.db.get(key).unwrap().unwrap();
    assert_eq!(foo, value);

    // Delete existing value, the value is returned upon deletion.
    let foo = app.db.delete(key).unwrap().unwrap();
    assert_eq!(foo, value);
    // Cannot retrieve deleted value.
    let none = app.db.get(key).unwrap();
    assert!(none.is_none());

    // Deleting a non existing value returns None for "not found".
    let none = app.db.delete(key).unwrap();
    assert!(none.is_none());
}

#[test]
fn flush_memtable_writes_data_correctly() {
    let n_sstables = 10;
    let memtable_size = 2;
    let mut app = TestApp::build(memtable_size, n_sstables).unwrap();
    app.populate_database(n_sstables * memtable_size - 1);
    app.gather_and_sort_sstables();

    // Assert every pair of `table` has been saved correctly and in the correct
    // order.
    let mut i = 0;
    for entry in app.sstables.iter() {
        let entry_path = entry.to_str().unwrap().to_string();
        let mut reader = Reader::from_path(entry_path).unwrap();
        for kv in reader.deserialize::<KeyValue<usize, String>>() {
            assert_entry(kv, &app, &mut i);
        }
    }
    assert_eq!(i, app.table.len() - 1);
}

#[test]
fn compact_sstables_works_correctly() {
    let n_sstables = 10;
    let memtable_size = 2;
    let mut app = TestApp::build(memtable_size, n_sstables).unwrap();

    // Before compression.
    app.populate_database(n_sstables * memtable_size);
    app.gather_and_sort_sstables();
    assert_eq!(n_sstables, app.sstables.len());

    // Add 2 more entries, this will trigger a compation.
    app.populate_database(2);
    app.gather_and_sort_sstables();
    // Assert we only have one `[0-9]+-data.sstable` file in the working directory, because they
    // have been compacted.
    assert_eq!(app.sstables.len(), 1);

    let mut i = 0;
    let entry = app.sstables.iter().next().unwrap();
    let entry_path = entry.to_str().unwrap().to_string();
    let mut reader = Reader::from_path(entry_path).unwrap();
    for kv in reader.deserialize::<KeyValue<usize, String>>() {
        assert_entry(kv, &app, &mut i);
    }
    assert_eq!(i, app.table.len());
}

#[test]
fn wal_is_updated_correctly() {
    let n_sstables = 10;
    let memtable_size = 2;
    let mut app = TestApp::build(memtable_size, n_sstables).unwrap();

    let mut reader = Reader::from_path(&app.wal_path).unwrap();
    let mut reader = reader.deserialize::<KeyValue<usize, String>>();
    for i in 0..10 {
        // Add an entry
        app.populate_database(1);
        // Assert key has been added to WAL.
        let kv = reader.next().unwrap().unwrap();
        assert_eq!(kv.key, app.table[i].0);
        assert_eq!(kv.value.unwrap(), app.table[i].1);

        // Remove the entry that has been added.
        let removed_kv = app.dequeue_db().unwrap();

        // Assert the removal from WAL.
        let kv = reader.next().unwrap().unwrap();
        assert_eq!(kv.key, removed_kv.key);
        assert!(kv.value.is_none());
    }
}

#[test]
fn manifest_works_correctly() {
    let n_sstables = 10;
    let memtable_size = 2;
    let added_elements = 10;
    let mut app = TestApp::build(memtable_size, n_sstables).unwrap();

    // Manifest should not exists yet.
    assert!(!fs::exists(&app.manifest_path).unwrap());

    // Add multiple sstables.
    app.test_manifest_entries(added_elements, added_elements / 2);

    // Add more sstables.
    app.test_manifest_entries(added_elements, added_elements);

    // Instanciate a new test app using the files produced by the old app.
    let mut new_app = TestApp::build(memtable_size, n_sstables).unwrap();
    new_app.test_manifest_entries(added_elements, added_elements);

    // Trigger a compaction.
    new_app.test_manifest_entries(2, 1);
}
