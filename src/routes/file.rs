use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};


pub const FILES_PATH: &str = "./data/files";


#[derive(Deserialize)]
#[derive(Serialize)]
struct UploadedFile {
    hash: String,
    chunks: Box<[String]>,
    name: String,
    corrupted: Option<bool>,
}

#[derive(Serialize)]
struct UploadedFilePresent {
    name: String,
    present: bool
}

fn sanitize_path(input: &str) -> Result<PathBuf, &'static str> {
    let path = input
        .trim_start_matches("file://")
        .trim_start_matches('/');

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
async fn add_file(file: web::Json<UploadedFile>) -> impl Responder {
    let clean_path = match sanitize_path(&file.name) {
        Ok(p) => p,
        Err(e) => return HttpResponse::BadRequest().body(e),
    };

    let full_path = Path::new(FILES_PATH).join(clean_path);

    if let Some(parent) = full_path.parent() {
        if let Err(e) = create_dir_all(parent) {
            log::error!("Error creating file: {:?}", parent);
            return HttpResponse::InternalServerError()
                .body(format!("Failed to create directories: {}", e));
        }
    }
    let json = match serde_json::to_vec(&file) {
        Ok(j) => j,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Serialization error: {}", e))
        }
    };

    let mut f = match File::create(&full_path) {
        Ok(file) => file,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("File creation error: {}", e))
        }
    };

    log::info!("Created file: {:?} {}", &file.name, file.hash);
    if let Err(e) = f.write_all(&json) {
        return HttpResponse::InternalServerError()
            .body(format!("Write error: {}", e));
    }

    HttpResponse::Ok().finish()
}

#[post("/has_files")]
async fn has_files(files: web::Json<Box<[UploadedFile]>>) -> web::Json<Box<[UploadedFilePresent]>> {

    let files_check: Box<[UploadedFilePresent]> = files.iter()
        .map(|file| {
            log::info!("{} {} {}", file.name, file.hash, file_exist(file));
            UploadedFilePresent {
            name: file.name.to_string(),
            present: file_exist(file)
        } }).collect();

    log::info!("{:?}", files_check.iter().count());
    web::Json(files_check)
}
fn file_exist(file: &UploadedFile) -> bool {
    let clean_path = match sanitize_path(&file.name) {
        Ok(p) => p,
        Err(_) => return false,
    };

    let file_path = Path::new(FILES_PATH).join(clean_path);
    File::open(file_path)
        .ok()
        .and_then(|f| serde_json::from_reader::<_, UploadedFile>(BufReader::new(f)).ok())
        .map(|stored| stored.hash == file.hash && !stored.corrupted.unwrap_or(false))
        .unwrap_or(false)
}
