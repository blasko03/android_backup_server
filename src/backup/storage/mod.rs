use std::path::{Path, PathBuf};
use std::string::ToString;

pub mod files_storage;
pub mod chunks_storage;
pub mod chunks_storage_local;
pub mod files_storage_local;


pub fn data_path() -> PathBuf {
    Path::new(&std::env::var("DATA_PATH").unwrap_or("./data".to_string())).to_path_buf()
}