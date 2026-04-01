use std::collections::HashSet;
use std::fs;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Error, ErrorKind, Write};
use std::io::ErrorKind::NotFound;
use std::path::{Path, PathBuf};
use crate::backup::device::{Device};
use crate::backup::storage::{data_path};
use crate::backup::storage::files_storage::{DeviceFile, FilesStorage};

const FILES_PATH: &str = "files";
pub struct FilesStorageLocal<'a>{
    pub(crate) device: &'a Device,
}
impl FilesStorage for FilesStorageLocal<'_> {
    fn save(&self, device_file: DeviceFile) -> Result<bool, Error> {
        let full_path = self.path_for_file(&device_file.path);
        if let Err(e) = full_path.parent()
            .ok_or("Invalid path".to_string())
            .and_then(|parent| create_dir_all(parent).map_err(|e| e.to_string())){
            log::error!("Error creating folders: {:?}", e);
            return Err(Error::new(ErrorKind::InvalidFilename, e));
        };

        let json = match serde_json::to_vec(&device_file){
            Ok(json) => json,
            Err(e) => {
                log::error!("Error serializing device file: {}", e);
                return Err(Error::from(e))
            }
        };

        match File::create(full_path).map_err(|e| Error::from(e))
            .and_then(|mut file| file.write_all(&json)) {
            Ok(_) => Ok(true),
            Err(e) => {
                log::error!("Error writing device file: {}", e);
                Err(e)
            }
        }
    }

    fn get(&self, path: &Path) -> Result<DeviceFile, Error> {
        let full_path = self.path_for_file(&path);
        if !self.exist(&full_path){
            log::debug!("File does not exist: {:?}", full_path);
            return Err(Error::from(NotFound))
        }

        File::open(full_path).map_err(|e| Error::from(e))
            .and_then(|file| {
                serde_json::from_reader::<_, DeviceFile>(BufReader::new(file))
                    .map_err(|e| Error::from(e))
            })
    }

    fn list(&self) -> Result<HashSet<PathBuf>, Error> {
        let mut directories: Vec<PathBuf> = Vec::new();
        let root_dir = self.path_for();
        directories.push(root_dir.clone());
        let mut files: HashSet<PathBuf> = HashSet::new();

        while let Some(dir) =directories.pop() {
            match dir.read_dir() {
                Ok(files ) => files,
                Err(e) => {
                    log::error!("Error reading directory: {}", e);
                    return Err(e)
                },
            }.filter(|file| file.is_ok())
                .map(|file| file.unwrap().path())
                .for_each(|file| {
                    if file.is_dir() {
                        directories.push(file.clone());
                    } else if file.is_file() {
                        files.insert(file.strip_prefix(&root_dir).unwrap().to_owned());
                    }
            })
        };

        Ok(files)
    }

    fn exist(&self, path: &Path) -> bool {
        path.exists()
    }

    fn remove(&self, path: &Path) -> Result<(), Error> {
        log::warn!("Removing file {}", path.display());
        fs::remove_file(self.path_for_file(&path))
    }
}

impl FilesStorageLocal<'_> {
    pub fn path_for_file(&self, file: &Path) -> PathBuf {
        self.path_for().join(file)
    }
    pub fn path_for(&self) -> PathBuf {
        data_path().join(self.device.uuid.as_str()).join(FILES_PATH)
    }
}
