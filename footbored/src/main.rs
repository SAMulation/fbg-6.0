// Import the necessary modules for our application
use std::sync::{Arc, Mutex}; // Arc and Mutex for shared state across threads
use tokio::sync::mpsc; // mpsc (multiple producer, single consumer) for async communication
use uuid::Uuid; // Uuid for unique client identification
use warp::{self, Filter}; // warp for creating the server and its routes

// Import modules defined locally in our application
mod game_server;
mod client;
mod game;

// Use the structures defined in these modules
use client::Client;
use game_server::GameServer;

// This is the main function, which is the entry point of the application
#[tokio::main]
async fn main() {
    // Create an instance of the GameServer struct, and wrap it in an Arc Mutex for shared state
    let game_server = Arc::new(Mutex::new(GameServer::new()));

    // The game_server_filter is a warp Filter that we'll use in our routes. It clones the game server state
    // for each incoming request
    let game_server_filter = warp::any().map(move || game_server.clone());

    // Define the websocket route. This route is for the "/websocket" endpoint and upgrades the connection to a WebSocket.
    // We pass along the game server state for each connection
    let websocket_route = warp::path("websocket")
        .and(warp::ws())
        .and(game_server_filter.clone())
        .map(|ws: warp::ws::Ws, game_server: Arc<Mutex<GameServer>>| {
            // Split the WebSocket into a sender and receiver
            let (ws_tx, ws_rx) = ws.split();
            // Create a channel for chat communication
            let (chat_sender, chat_receiver) = mpsc::unbounded_channel();
            // Generate a unique id for the client
            let client_id = Uuid::new_v4();

            // Create a client instance
            let client = Client {
                id: client_id,
                ws: ws_tx,
                game_server: game_server.clone(),
                chat_sender: Some(chat_sender),
            };

            // Handle the connection
            Client::handle_connection(ws_rx, game_server, client, chat_receiver)
        });

    // Define the routes for the warp server
    let routes = websocket_route;

    // Start the warp server on port 3030
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
