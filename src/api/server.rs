use crate::api::auth::jwt_auth::{jwt_auth_middleware, login};
use crate::api::routes::chunk::{add_chunk, download_chunk, get_chunk};
use crate::api::routes::file::{add_file, files_list, has_files};
use crate::api::routes::liveliness::liveliness;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

#[actix_web::main]
pub async fn start_server() -> std::io::Result<()> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(
            "/home/daniel/apps/certs/danielblasinaCA/issued/backups.danielblasina.dev.key",
            SslFiletype::PEM,
        )
        .unwrap();
    builder
        .set_certificate_chain_file(
            "/home/daniel/apps/certs/danielblasinaCA/issued/backups.danielblasina.dev.crt",
        )
        .unwrap();
    let port = std::env::var("PORT")
        .unwrap_or_default()
        .parse::<u16>()
        .unwrap_or(8080);
    HttpServer::new(|| {
        App::new()
            .wrap(from_fn(jwt_auth_middleware))
            .service(liveliness)
            .service(add_chunk)
            .service(get_chunk)
            .service(download_chunk)
            .service(has_files)
            .service(add_file)
            .service(files_list)
            .service(login)
    })
    .bind_openssl("127.0.0.1:9090", builder)?
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
