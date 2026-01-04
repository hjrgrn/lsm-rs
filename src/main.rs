use lsm_tree_rs::database::Database;

fn main() {
    let mut db: Database<String, String> = Database::build(10).unwrap();
    let _ = db.put("a".to_string(), "apple".to_string());
    let g = db.get("a".to_string()).unwrap();
    println!("{}", g);
    // let d = db.delete("a".to_string()).unwrap();
}
