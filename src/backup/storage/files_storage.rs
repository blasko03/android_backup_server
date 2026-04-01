use std::collections::HashSet;
use std::io::Error;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DeviceFile{
    pub(crate) path: PathBuf,
    pub(crate) versions: Vec<DeviceFileVersion>
}
#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceFileVersion{
    pub(crate) hash: String,
    pub(crate) chunks: Vec<String>,
    pub(crate) corrupted: bool,
    pub(crate) deleted: bool,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
}

pub trait FilesStorage {
    fn save(&self, device_file: DeviceFile) -> Result<bool, Error>;
    fn get(&self, path: &Path) -> Result<DeviceFile, Error>;
    fn list(&self) -> Result<HashSet<PathBuf>, Error>;
    fn exist(&self, path: &Path) -> bool;
    fn remove(&self, path: &Path) -> Result<(), Error> ;
}
