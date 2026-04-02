use std::io::{Error, ErrorKind, Read};
use actix_multipart::form::tempfile::TempFile;
use sha2::{Digest, Sha256};
use hex;
use crate::backup::storage::chunks_storage::ChunksStorage;

pub struct DeviceChunks<'a> {
    pub(crate) chunks_storage: Box<dyn ChunksStorage + 'a>
}

impl DeviceChunks<'_> {
    pub fn add(&self, temp_file: &TempFile, hash: &String) -> Result<bool, Error> {
        let device_repository = &self.chunks_storage;

        let mut hasher = Sha256::new();
        let mut buffer = Vec::new();
        let mut file = &temp_file.file;
        if let Err(e) = file.read_to_end(&mut buffer) {
            log::error!("Error reading chunk file: {}", e);
            return Err(e);
        }
        hasher.update(&buffer);
        let result = hasher.finalize();

        let hash_hex = hex::encode(result);

        if hash_hex != hash.to_string() {
            log::error!("Chunk hash not corresponding uploaded_hash: {} computed_hash: {}", hash_hex, hash);
            return Err(Error::new(ErrorKind::InvalidData, "Chunk hash not corresponding".to_string()))
        }

        let chunk = device_repository.add(hash, temp_file);

        if let Err(e) = chunk {
            log::error!("Error adding device chunk: {}", hash);
            return Err(e)
        }
        Ok(true)
    }

    pub fn exist(&self, hash: &String) -> bool {
        self.chunks_storage.exist(hash)
    }
}
