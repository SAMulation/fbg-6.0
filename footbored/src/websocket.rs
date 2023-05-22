use std::sync::Arc;
use std::collections::HashMap;
use tokio::{sync::{Mutex, broadcast}, spawn};
use uuid::Uuid;
use log::{info, warn, error};
use warp::ws::{Message, WebSocket};
use futures::{StreamExt, SinkExt};
use crate::lobby::Lobby;

pub async fn handle_connection(
    ws: WebSocket,
    clients: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>,
    lobby: Arc<Mutex<Lobby>>,
    ) {
        // Split WebSocket into sender and receiver halves
        let (mut client_ws_tx, mut client_ws_rx) = ws.split();
        // Create a broadcast channel for incoming messages
        let (client_tx, _) = broadcast::channel::<String>(10); // Set an appropriate buffer size
        // Generate a unique ID for each client
        let client_id = Uuid::new_v4();
        // Register the client by inserting the sender half into the shared state
        clients.lock().await.insert(client_id, client_tx.clone());

        // Add logic to handle lobby-related messages
        // Add logic to handle lobby-related messages
        while let Some(result) = client_ws_rx.next().await {
            match result {
                Ok(message) => {
                    // Handle lobby-related messages, such as game requests and lobby chat messages
                    if let Ok(text) = message.to_str() {
                        if text.starts_with("/message") {
                            // Extract relevant data from the message and broadcast it to all players
                            let chat_message = text.to_string();
                            lobby.lock().await.broadcast_message(chat_message, &clients);
                        } else if text.starts_with("/request_game") {
                            // Placeholder for requesting a game
                            // Extract relevant data from the message and create a game request
                            // let request = create_game_request_from_message(text);
                            // lobby.lock().await.send_game_request(request);
                        } else if text.starts_with("/leave_lobby") {
                            // Placeholder for leaving the lobby gracefully
                            // Extract relevant data from the message, such as player ID
                            // lobby.lock().await.remove_player(player_id);
                        }
                    }
                }
                Err(e) => {
                    warn!("Error receiving message: {}", e);
                    break;
                }
            }
        }



    // Log disconnection and remove client from the shared state
    info!("Client {} disconnected", client_id);
    clients.lock().await.remove(&client_id);


    // Spawn a task to write messages to the WebSocket
    let sender_task = spawn(async move {
        while let Ok(message) = client_tx.subscribe().recv().await {
            if let Err(e) = client_ws_tx.send(Message::text(message)).await {
                error!("Failed to send message: {}", e);
                break;
            }
        }
    });

    // Wait for the sender task to complete
    let _ = sender_task.await;
}
