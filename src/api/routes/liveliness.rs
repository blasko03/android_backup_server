use actix_web::{HttpResponse, Responder, get};

#[get("/liveliness")]
async fn liveliness() -> impl Responder {
    HttpResponse::Ok().body("All ok")
}
