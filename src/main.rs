use lsm_tree_rs::{database::Database, manifest::MANIFEST_PATH, wal::WAL_PATH};

fn main() {
    let mut db: Database<String, String> = Database::build(10, WAL_PATH, MANIFEST_PATH).unwrap();
    let _ = db.put("a".to_string(), "apple".to_string());
    let g = db.get("a".to_string()).unwrap();
    println!("{}", g.unwrap());
    // let d = db.delete("a".to_string()).unwrap();
}
