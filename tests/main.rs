use std::fs;

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
