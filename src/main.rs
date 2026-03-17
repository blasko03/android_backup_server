mod routes;
mod tasks;

use actix_web::{App, HttpServer};
use routes::chunk::{get_chunk, add_chunk};
use routes::file::{has_files, add_file};
use routes::liveliness::liveliness;
use tasks::file_consistency;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let port =std::env::var("PORT").unwrap_or_default().parse::<u16>().unwrap_or(8080);
    HttpServer::new(|| {
        App::new()
            .service(liveliness)
            .service(add_chunk)
            .service(get_chunk)
            .service(has_files)
            .service(add_file)
    }).bind(("0.0.0.0", port))?
        .run()
        .await
}
