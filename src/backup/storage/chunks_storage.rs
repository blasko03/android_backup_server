use std::collections::HashSet;
use std::fs::File;
use std::io::Error;
use actix_multipart::form::tempfile::TempFile;

pub struct DeviceChunk{
    pub(crate) hash: String,
    pub(crate) file: File,
}

pub trait ChunksStorage {
    fn add(&self, hash: &String, file: &TempFile) -> Result<bool, Error>;
    fn get(&self, hash: &String) -> Result<DeviceChunk, Error>;
    fn exist(&self, hash: &String) -> bool;
    fn remove(&self, hash: &String) -> Result<(), Error> ;

    fn list(&self) -> Result<HashSet<String>, Error>;
}