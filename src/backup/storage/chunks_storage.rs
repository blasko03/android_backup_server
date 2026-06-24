use actix_multipart::form::tempfile::TempFile;
use std::collections::HashSet;
use std::fs::File;
use std::io::Error;

#[allow(dead_code)]
pub struct DeviceChunk {
    pub(crate) hash: String,
    pub(crate) file: File,
}

pub trait ChunksStorage {
    fn add(&self, hash: &str, file: &TempFile) -> Result<bool, Error>;
    fn get(&self, hash: &str) -> Result<DeviceChunk, Error>;
    fn exist(&self, hash: &str) -> bool;
    fn remove(&self, hash: &str) -> Result<(), Error>;

    fn list(&self) -> Result<HashSet<String>, Error>;
}
