#![warn(missing_docs)]

//! TODO:

// TODO: Another possible way to implement this levereging Rust's typesystem would be
// to define an enum, EntryStatus:
// ```
// enum EntryStatus<V> {
//    Value<V>
//    Tombstone
// }
// ```
// Store this into MemTable::data instead of just V. When deleting an element from MemTable::data
// Instead of removing it completely you put a Tombstone.
pub trait Tombstone {
    fn is_tombstone(&self) -> bool;
}

impl Tombstone for &str {
    fn is_tombstone(&self) -> bool {
        // TODO: proper tombstone
        *self == "TODO"
    }
}

impl Tombstone for String {
    fn is_tombstone(&self) -> bool {
        // TODO: proper tombstone
        *self == "TODO"
    }
}
