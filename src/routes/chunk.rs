use std::fs;
use std::io::Read;
use std::path::Path;
use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_web::{get, post, web, HttpResponse, Responder};
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
const CHUNKS_PATH: &str = "./data/chunks";

#[derive(Debug, MultipartForm)]
struct UploadForm {
    file: TempFile,
}

#[post("/chunk")]
async fn add_chunk(MultipartForm(form): MultipartForm<UploadForm>) -> impl Responder {
    let chunks_path = Path::new(CHUNKS_PATH);
    let file_name = match &form.file.file_name {
        Some(name) => name,
        None => return HttpResponse::BadRequest().body("Missing file name"),
    };
    let dest_path = chunks_path.join(&file_name);

    // Copy uploaded file
    if let Err(e) = fs::copy(form.file.file.path(), &dest_path) {
        return HttpResponse::InternalServerError().body(format!("Failed to save file: {}", e));
    }

    // Compute SHA-256 hash
    let mut file = match fs::File::open(&dest_path) {
        Ok(f) => f,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Failed to open file: {}", e));
        }
    };

    let mut hasher = Sha256::new();
    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        return HttpResponse::InternalServerError().body(format!("Failed to read file: {}", e));
    }
    hasher.update(&buffer);
    let result = hasher.finalize();

    // Convert hash to Base64
    let hash_base64 = URL_SAFE.encode(result);


    log::info!("File '{}' saved with Base64 hash: {}", file_name, hash_base64);
    if hash_base64 == file_name.to_string() {
        return HttpResponse::Ok().finish()
    }

    let _ =fs::remove_file(&dest_path);
    HttpResponse::BadRequest().body("invalid file hash")
}

#[get("/chunk/{hash}")]
async fn get_chunk(hash: web::Path<String>) -> impl Responder {
    let hash = hash.into_inner();
    let chunks_path = Path::new(CHUNKS_PATH);
    log::info!("Path to chunks: {} {}", chunks_path.join(hash.as_str()).display(), chunks_path.join(hash.as_str()).exists());
    if chunks_path.join(hash).exists() {
        return HttpResponse::Ok()
    }
    HttpResponse::NotFound()
}
