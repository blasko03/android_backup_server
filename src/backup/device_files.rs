use std::io::Error;
use std::io::ErrorKind::NotFound;
use std::path::Path;
use chrono::{TimeDelta, Utc};
use crate::backup::storage::files_storage::{DeviceFile, DeviceFileVersion, FilesStorage};

pub struct DeviceFiles<'a> {
    pub(crate) file_storage: Box<dyn FilesStorage + 'a>
}

const MAX_VERSIONS: u8 = 3;
const MIN_VERSIONS: u8 = 2;
const MAX_AGE: TimeDelta = chrono::Duration::days(180);

impl DeviceFiles<'_> {
    pub fn add(&self, path: &Path, chunks: &Vec<String>, hash: &String) -> Result<bool, Error> {
        let device_repository = &self.file_storage;
        let mut file = match device_repository.get(path) {
            Ok(f) => f,
            Err(e) => {
                if e.kind() != NotFound {
                    log::debug!("Error getting device file: {}", e);
                    return Err(e);
                }
                DeviceFile{path: path.to_path_buf(), versions: Vec::new()}
            }
        };

        if !file.versions.is_empty() && file.versions.last().unwrap().hash==hash.as_str() {
            return Ok(false);
        }

        file.versions.push(DeviceFileVersion{
            hash: hash.clone(),
            chunks: chunks.clone(),
            corrupted: false,
            deleted: false,
            created_at: Utc::now()
        });

        device_repository.save(file)
    }

    pub fn get(&self, path: &Path) -> Result<DeviceFile, Error> {
        self.file_storage.get(path)
    }

    pub fn delete(&self, path: &Path) -> Result<bool, Error> {
        let device_repository = &self.file_storage;
        let mut file = match device_repository.get(path) {
            Ok(f) => f,
            Err(e) => {
                return Err(e);
            }
        };
        if file.versions.is_empty() {
            return Err(Error::new(NotFound, "File not has no versions"));
        }
        let mut versions = file.versions.clone();
        let mut last_version = versions.last().unwrap().clone();
        last_version.deleted = true;
        versions.remove(versions.len() - 1);
        versions.push(last_version);

        file.versions = versions;
        device_repository.save(file)
    }

    pub fn files_clean(&self) -> Result<(), Error> {
        let file_storage = &self.file_storage;
        for file_path in file_storage.list()?.iter() {
            let mut file = file_storage.get(&file_path)?;
            let mut versions = file.versions;

            let last_element = versions
                .iter()
                .enumerate()
                .filter(|(i, _)| i + (MIN_VERSIONS as usize) < versions.len())
                .filter(|(i, v)| v.created_at < Utc::now()-MAX_AGE
                    || i + (MAX_VERSIONS as usize) < versions.len())
                .map(|(i, _)| i+1)
                .last()
                .unwrap_or_default();

            if versions.is_empty() {
                file_storage.remove(file_path)?;
            }

            if let Some(version) = versions.last() && version.deleted == true {
                file_storage.remove(file_path)?;
            }

            if last_element > 0 {
                log::warn!("Deleting versions from 0 to {} for file {:?}", last_element, file_path.display());
                versions.drain(0..last_element);
                file.versions = versions;
                file_storage.save(file)?;
            }
        }
        Ok(())
    }
}
