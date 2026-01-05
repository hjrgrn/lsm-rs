//! Write Ahead Log
//! XXX:

use std::{fs::File, io::BufWriter, path::PathBuf};

pub struct WAL {
    path: PathBuf,
    writer: BufWriter<File>,
}
