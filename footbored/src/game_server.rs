use std::sync::Arc;
use std::collections::HashMap;
use warp::Filter;
use tokio::sync::Mutex;
use uuid::Uuid;
use log::info;
use crate::websocket::handle_connection;

pub async fn start_server() {
    // Create a shared state of connected clients
    let clients = Arc::new(Mutex::new(HashMap::new()));

    // Define WebSocket routes
    let routes = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let clients = clients.clone();
            // Upgrade HTTP connection to WebSocket and handle it
            ws.on_upgrade(move |websocket| {
                handle_connection(websocket, clients.clone())
            })
        });

    info!("Starting server on 0.0.0.0:3030");

    // Start the server
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
