mod game;
mod websocket;

use actix_web::{web, App, HttpServer};
use native_tls::{Identity, TlsAcceptor};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use websocket::index;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let address = "0.0.0.0:8080";
    println!("Starting Rust server on {}", address);

    // Load SSL keys
    let cert_path = "path/to/certificate.crt";
    let key_path = "path/to/private.key";

    let cert_file = File::open(cert_path)?;
    let cert_reader = BufReader::new(cert_file);

    let key_file = File::open(key_path)?;
    let key_reader = BufReader::new(key_file);

    let identity = Identity::from_pem(&cert_reader, &key_reader)?;
    let tls_acceptor = TlsAcceptor::new(identity)?;

    HttpServer::new(|| App::new().route("/ws/", web::get().to(index)))
        .bind_tls(address, tls_acceptor)?
        .run()
        .await
}
