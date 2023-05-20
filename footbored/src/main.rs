mod game;
mod websocket;

use actix_web::{web, App, HttpServer};
use websocket::index;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/ws/", web::get().to(index)))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
