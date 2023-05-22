use log::info;

mod game_server;
mod websocket;
mod lobby;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting server");

    game_server::start_server().await;
}
