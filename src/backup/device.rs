use std::io::{Error, ErrorKind, Read};
use sha2::{Digest, Sha256};
use hex;
use crate::backup::device_chunks::DeviceChunks;
use crate::backup::device_files::DeviceFiles;
use crate::backup::storage::chunks_storage_local::ChunkStorageLocal;
use crate::backup::storage::files_storage::DeviceFileVersion;
use crate::backup::storage::files_storage_local::FilesStorageLocal;
pub struct Device  {
    pub(crate) uuid: String,
}

impl Device {
    pub fn files(&'_ self) -> DeviceFiles<'_> {
        let storage = FilesStorageLocal {device: self};
        DeviceFiles { file_storage: Box::new(storage)}
    }

    pub fn chunks(&'_ self) -> DeviceChunks<'_> {
        let storage = ChunkStorageLocal {device: self};
        DeviceChunks { chunks_storage: Box::new(storage)}
    }

    pub fn chunks_clean(&self) -> Result<(), Error> {
        let file_storage = self.files().file_storage;
        let chunks_storage = self.chunks().chunks_storage;
        let mut chunks = match chunks_storage.list(){
            Ok(chunks) => chunks,
            Err(e) => {
                log::debug!("Chunk storage list error: {:?}", e);
                return Err(e)
            },
        };
        let files = match file_storage.list(){
            Ok(chunks) => chunks,
            Err(e) => {
                log::debug!("File storage list error: {:?}", e);
                return Err(e)
            },
        };

        for file_path in files {
            let file = match file_storage.get(&file_path){
                Ok(file) => file,
                Err(e) => {
                    log::debug!("File storage get error: {:?}", e);
                    return Err(Error::new(ErrorKind::Other, "Failed to retrieve file storage"))
                },
            };

            for chunk in file.versions.iter().flat_map(|f| f.chunks.clone()) {
                chunks.remove(&chunk);
            };
        };

        log::warn!("Found {} orphan chunks", chunks.len());
        for chunk in chunks {
            chunks_storage.remove(&chunk)?;
        }
        Ok(())
    }

    pub fn consistency_check(&self) -> Result<(), Error> {
        let file_storage = self.files().file_storage;
        let files = match file_storage.list() {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        for path in files {
            let file = match file_storage.get(&path){
                Ok(f) => f,
                Err(e) => return Err(e),
            };

            for version in file.versions {
                 self.hash_check_for_version(version).unwrap();
            }
        };
        Ok(())
    }

    fn hash_check_for_version(&self, version: DeviceFileVersion) -> Result<(), Error> {
        let chunk_storage = self.chunks().chunks_storage;
        let mut file_hasher = Sha256::new();
        for chunk_name in version.chunks {
            let mut chunk_hasher = Sha256::new();
            let mut buffer = Vec::new();
            chunk_storage
                .get(&chunk_name)
                .and_then(|mut chunk| chunk.file.read_to_end(&mut buffer))?;
            file_hasher.update(&buffer);
            chunk_hasher.update(&buffer);
            let hash =  chunk_hasher.finalize();
            if hex::encode(hash) != chunk_name {
                return Err(Error::new(ErrorKind::Other, "Chunk hash mismatch"));
            }
        }

        let result = file_hasher.finalize();
        if hex::encode(result) == version.hash{
            return Ok(());
        }
        Err(Error::new(ErrorKind::Other, "File hash mismatch"))
    }
}
