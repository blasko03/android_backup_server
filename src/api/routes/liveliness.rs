use actix_web::{get, HttpResponse, Responder};

#[get("/liveliness")]
async fn liveliness() -> impl Responder {
    HttpResponse::Ok().body("All ok")
}

