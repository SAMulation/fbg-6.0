use warp::Filter;
use std::sync::Arc;

mod websocket;
mod game_server;

use crate::websocket::handle_connection;
use crate::game_server::GameServer;

#[tokio::main]
async fn main() {
    let game_server = Arc::new(GameServer::new());

    let game_server_filter = warp::any()
        .map(move || Arc::clone(&game_server));

    let routes = warp::path("ws")
        .and(warp::ws())
        .and(game_server_filter)
        .map(|ws: warp::ws::Ws, game_server: Arc<GameServer>| {
            ws.on_upgrade(move |socket| handle_connection(socket, game_server))
        });

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
