use std::io::Read;
use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use crate::backup::device::Device;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    file: TempFile,
}

#[post("/chunk")]
async fn add_chunk(MultipartForm(form): MultipartForm<UploadForm>, req: HttpRequest) -> impl Responder {
    let extensions = req.extensions();
    let device = extensions.get::<Device>().unwrap();
    let file_name = match &form.file.file_name {
        Some(name) => name,
        None => return HttpResponse::BadRequest().body("Missing file name"),
    };

    match device.chunks().add(&form.file, file_name) {
        Ok(_) => HttpResponse::Ok().body("Successfully added file"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to add chunk: {}", e)),
    }
}

#[get("/chunk/{hash}")]
async fn get_chunk(hash: web::Path<String>, req: HttpRequest) -> impl Responder {
    let extensions = req.extensions();
    let device = extensions.get::<Device>().unwrap();
    let hash = hash.into_inner();
    if device.chunks().exist(&hash) {
        return HttpResponse::Ok()
    }
    HttpResponse::NotFound()
}

#[get("/chunk/{hash}/download")]
async fn download_chunk(hash: web::Path<String>, req: HttpRequest) -> impl Responder {
    let extensions = req.extensions();
    let device = extensions.get::<Device>().unwrap();
    let hash = hash.into_inner();
    if !device.chunks().exist(&hash) {
        return HttpResponse::NotFound().finish();
    }

    let mut buffer = Vec::new();
    let mut chunk = match device.chunks().chunks_storage.get(&hash) {
        Ok(chunk) => chunk,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if let Err(e) = chunk.file.read_to_end(&mut buffer){
        log::error!("Failed to read chunk: {}", e);
        return HttpResponse::InternalServerError().finish();
    }
    
    HttpResponse::Ok()
        .append_header(("Content-Type", "application/octet-stream"))
        .append_header(("Content-Disposition", format!("attachment; filename={:?}", hash)))
        .body(buffer)
}
