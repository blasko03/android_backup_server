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
        let mut chunk = match device_repository.add(hash, temp_file)
            .and_then(|_| device_repository.get(hash)) {
            Ok(c) => c,
            Err(e) => return Err(e)
        };

        let mut hasher = Sha256::new();
        let mut buffer = Vec::new();
        if let Err(e) = chunk.file.read_to_end(&mut buffer) {
            log::error!("Error reading chunk file: {}", e);
            return Err(e);
        }
        hasher.update(&buffer);
        let result = hasher.finalize();

        let hash_base64 = hex::encode(result);

        if hash_base64 == hash.to_string() {
            return Ok(true);
        }

        log::error!("Chunk hash not corresponding uploaded_hash: {} computed_hash: {}", hash_base64, chunk.hash);
        if let Err(e) = device_repository.remove(hash) {
            log::error!("Error removing device file: {}", e);
        }
        Err(Error::new(ErrorKind::InvalidData, "Chunk hash not corresponding".to_string()))
    }

    pub fn exist(&self, hash: &String) -> bool {
        self.chunks_storage.exist(hash)
    }
}
