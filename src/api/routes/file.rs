use std::path::{Path, PathBuf};
use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::backup::device::{Device};
use crate::backup::storage::files_storage::DeviceFile;

#[derive(Serialize, Deserialize)]
struct UploadedFile {
    hash: String,
    chunks: Vec<String>,
    name: String,
}

#[derive(Serialize)]
struct UploadedFilePresent {
    name: String,
    present: bool
}

fn sanitize_path(input: &str) -> Result<PathBuf, &'static str> {
    let path = input
        .trim_start_matches("file://")
        .trim_start_matches("/");

    let path = Path::new(path);

    let mut clean = PathBuf::new();

    for component in path.components() {
        use std::path::Component;

        match component {
            Component::Normal(c) => clean.push(c),
            _ => return Err("Invalid path"),
        }
    }

    Ok(clean)
}

#[post("/file")]
async fn add_file(file: web::Json<UploadedFile>, req: HttpRequest) -> impl Responder {
    let extensions = req.extensions();
    let device = extensions.get::<Device>().unwrap();

    let clean_path = match sanitize_path(&file.name) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Error sanitize_path: {}", e);
            return HttpResponse::BadRequest().body(e)
        },
    };

    match device.files().add(&clean_path, &file.chunks, &file.hash){
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error adding file: {}", e);
            HttpResponse::BadRequest().finish()
        }
    }
}

#[post("/has_files")]
async fn has_files(files: web::Json<Box<[UploadedFile]>>, req: HttpRequest) -> web::Json<Box<[UploadedFilePresent]>> {
    let extensions = req.extensions();
    let device = extensions.get::<Device>().unwrap();

    let files_check: Box<[UploadedFilePresent]> = files.iter()
        .map(|file| {
            log::info!("{} {} {}", file.name, file.hash, file_exist(&device, file));
            UploadedFilePresent {
            name: file.name.to_string(),
            present: file_exist(&device, file)
        } }).collect();

    log::info!("{:?}", files_check.iter().count());
    web::Json(files_check)
}
fn file_exist(device: &Device, file: &UploadedFile) -> bool {
    let clean_path = match sanitize_path(&file.name) {
        Ok(p) => p,
        Err(_) => return false,
    };

    let files = device.files();
    match files.get(&clean_path).ok().and_then(|file| file.versions.last().cloned()) {
        Some(version) => version.hash == file.hash && version.corrupted == false,
        _ => false
    }
}

#[post("/files")]
async fn files_list(req: HttpRequest) -> web::Json<Vec<DeviceFile>> {
    let extensions = req.extensions();
    let device = extensions.get::<Device>().unwrap();

    let files = match device.files().file_storage.list() {
        Ok(f) => f,
        Err(e) => return web::Json(Vec::new())
    }
        .iter()
        .map(|name| device.files().get(name).unwrap()).collect::<Vec<DeviceFile>>();
    web::Json(files)
}
