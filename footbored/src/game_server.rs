use std::sync::Arc;
use std::collections::HashMap;
use warp::Filter;
use tokio::sync::Mutex;
use log::info;
use crate::websocket::handle_connection;
use crate::lobby::start_lobby;
use std::sync::atomic::AtomicBool;

pub async fn start_server() {
    // Create a shared state of connected clients
    let clients = Arc::new(Mutex::new(HashMap::new()));

    // Start the lobby
    let lobby = start_lobby().await;

    // Create a flag to indicate if the server is running
    let running = Arc::new(AtomicBool::new(true));

    // Define WebSocket routes
    let routes = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let clients = clients.clone();
            let lobby = lobby.clone();
            let running = running.clone();
            // Upgrade HTTP connection to WebSocket and handle it
            ws.on_upgrade(move |websocket| {
                handle_connection(websocket, clients.clone(), lobby.clone(), running.clone())
            })
        });

    info!("Starting server on 0.0.0.0:3030");

    // Start the server
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
