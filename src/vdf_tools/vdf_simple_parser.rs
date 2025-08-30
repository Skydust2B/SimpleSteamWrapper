use std::fs;
use std::path::PathBuf;
use vdf_reader::entry::Table;

pub fn read_vdf(path: PathBuf) -> Table {
    let text = fs::read_to_string(path).unwrap();
    Table::load_from_str(&text).unwrap()
}
