use std::{
    char,
    fs::{self, File, read_dir},
    io::{BufRead, BufReader},
};

use csv::Reader;
use fake::{Fake, faker::lorem::en::Word};
use lsm_tree_rs::sstable::KeyValue;

use crate::utils::TestApp;

mod utils;

#[test]
pub fn assert_wall_is_created_correctly() {
    let app = TestApp::build(10, 10).unwrap();
    assert!(fs::exists(&app.wal_path).unwrap());
}

#[test]
pub fn put_get_and_delete_work_correctly_in_memory() {
    let key = "key".to_string();
    let value = "value".to_string();

    let mut app = TestApp::build(10, 10).unwrap();

    // Put a value and retrieve it.
    app.db.put(key.clone(), value.clone()).unwrap();
    let foo = app.db.get(key.clone()).unwrap().unwrap();
    assert_eq!(foo, value);

    // Delete existing value, the value is returned upon deletion.
    let foo = app.db.delete(key.clone()).unwrap().unwrap();
    assert_eq!(foo, value);
    // Cannot retrieve deleted value.
    let none = app.db.get(key.clone()).unwrap();
    assert!(none.is_none());

    // Deleting a non existing value returns None for "not found".
    let none = app.db.delete(key.clone()).unwrap();
    assert!(none.is_none());
}

#[test]
fn flush_memtable_writes_data_correctly() {
    let amount_of_tables = 10;
    let memtable_size = 2;
    let mut app = TestApp::build(memtable_size, amount_of_tables).unwrap();

    // Create a Vec of key/value pairs and populate the database with it.
    // The pairs with the smaller key have been created earlier.
    let mut table: Vec<(String, String)> = Vec::new();
    for i in 0..amount_of_tables * memtable_size - 1 {
        let key = i.to_string();
        let val: String = Word().fake();
        table.push((key.clone(), val.clone()));
        let res = app.db.put(key, val);
        assert!(res.is_ok());
    }

    // Gather every sstable file created.
    let entries = read_dir(app.working_dir()).unwrap();
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
                // XXX:
                let reader = BufReader::new(File::open(entry_path).unwrap());
                for l in reader.lines() {
                    println!("{}", l.unwrap());
                }
                println!("");
                Some(e.unwrap())
            } else {
                None
            }
        })
        .collect();
    // SSTables are stored in a Vec and ordered by creation time (via filename),
    // so earlier files appear in earlier SSTables.
    entries.sort_by(|file0, file1| {
        file0
            .file_name()
            .to_str()
            .unwrap()
            .cmp(file1.file_name().to_str().unwrap())
    });
    // Assert every pair of `table` has been saved correctly and in the correct
    // order.
    let mut i = 0;
    for entry in entries {
        let entry_path = entry.path().to_str().unwrap().to_string();
        let mut reader = Reader::from_path(entry_path).unwrap();
        for kv in reader.deserialize::<KeyValue<String, String>>() {
            let kv = kv.unwrap();
            let (key, value) = &table[i];
            let k = kv.key;
            assert_eq!(&k, key);
            assert_eq!(&kv.value.unwrap(), value);
            assert_eq!(k, i.to_string());
            i += 1;
        }
    }
    assert_eq!(i, table.len() - 1);
}
