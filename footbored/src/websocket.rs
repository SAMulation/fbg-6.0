use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::ws::WebSocket;

mod client;

use crate::client::Client;
use crate::game_server::GameServer;

pub async fn handle_connection(
    ws: WebSocket,
    game_server: Arc<Mutex<GameServer>>,
) {
    let client_id = Uuid::new_v4();
    let sender = game_server.lock().await.add_client(client_id);
    
    let client = Client::new(ws, game_server.clone(), client_id, sender);

    tokio::spawn(async move {
        client.handle_connection().await;
    });
}
