// for each file in directory
// read the file json
// for each chunk read chunk, calculate checksum of chunk + checksum of file
// if chunk is corrupted move to chunks_corrupted folder
// if file corrupted mark as corrupted

use std::fs;
use std::path::{Path, PathBuf};
use crate::routes::file::FILES_PATH;

fn visit_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                files.append(&mut visit_dirs(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}

pub fn check() {
    let full_path = Path::new(FILES_PATH);
    for file_path in visit_dirs(full_path) {
        log::info!("Checking file: {}", file_path.display());
    }
}