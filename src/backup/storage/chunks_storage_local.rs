use std::collections::HashSet;
use std::fs;
use std::fs::{create_dir_all, File};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use actix_multipart::form::tempfile::TempFile;
use crate::backup::device::{Device};
use crate::backup::storage::chunks_storage::{ChunksStorage, DeviceChunk};
use crate::backup::storage::data_path;

const CHUNKS_PATH: &str = "chunks";
pub struct ChunkStorageLocal<'a>{
    pub(crate) device: &'a Device,
}

impl ChunksStorage for ChunkStorageLocal<'_> {
    fn add(&self, hash: &String, file: &TempFile) -> Result<bool, Error> {
        let dest_path = self.path_for_chunk(&hash);

        if let Err(e) = dest_path.parent()
            .ok_or("Invalid path".to_string())
            .and_then(|parent| create_dir_all(parent).map_err(|e| e.to_string())) {
            log::error!("Error creating file: {:?}", e);
            return Err(Error::new(ErrorKind::InvalidFilename, e));
        }

        if let Err(e) = fs::copy(file.file.path(), &dest_path) {
            log::error!("Failed to save file: {}", e);
            return Err(e);
        }

        Ok(true)
    }

    fn get(&self, hash: &String) -> Result<DeviceChunk, Error> {
        let dest_path = self.path_for_chunk(&hash);
        match File::open(dest_path){
            Ok(file) => Ok(DeviceChunk{hash: hash.clone(), file}),
            Err(e) => {
                log::error!("Failed to open file: {}", e);
                Err(e)
            }
        }
    }

    fn exist(&self, hash: &String) -> bool {
        let dest_path = self.path_for_chunk(&hash);
        dest_path.exists()
    }

    fn remove(&self, hash: &String) -> Result<(), Error> {
        log::warn!("Removing chunk {}", hash);
        fs::remove_file(self.path_for_chunk(&hash))
    }

    fn list(&self) -> Result<HashSet<String>, Error> {
        let chunks = match self.path_for_chunk("").read_dir(){
            Ok(files) => files,
            Err(e) => {
                log::error!("Error read_dir for path: {}", e);
                return Err(e)
            },
        }.filter(|f| f.is_ok())
            .map(|f| f.unwrap().path())
            .filter(|f| f.is_file())
            .map(|f| f.file_name().unwrap().to_str().unwrap().to_owned())
            .collect();
        Ok(chunks)
    }
}


impl ChunkStorageLocal<'_> {
    pub fn path_for_chunk(&self, name: &str) -> PathBuf {
        self.path_for().join(name)
    }

    pub fn path_for(&self) -> PathBuf {
        data_path().join(self.device.uuid.as_str()).join(CHUNKS_PATH)
    }
}