use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;
use tokio::task::spawn;
use log::{info, warn, error};
use warp::ws::{Message, WebSocket};
use futures::{StreamExt, SinkExt};

pub async fn handle_connection(ws: WebSocket, clients: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>) {
    // Split WebSocket into sender and receiver halves
    let (mut client_ws_tx, mut client_ws_rx) = ws.split();
    // Create a broadcast channel for incoming messages
    let (client_tx, _) = broadcast::channel::<String>(10); // Set an appropriate buffer size
    // Generate a unique ID for each client
    let client_id = Uuid::new_v4();
    // Register the client by inserting the sender half into the shared state
    clients.lock().await.insert(client_id, client_tx.clone());

    // Spawn a task to read messages from the WebSocket
    let receiver_task = spawn(async move {
        while let Some(result) = client_ws_rx.next().await {
            match result {
                Ok(message) => {
                    if let Ok(text) = message.to_str() {
                        // Broadcast received message to all connected clients
                        let broadcast_msg = format!("Client {}: {}", client_id, text);
                        let mut client_ids = Vec::new();
                        for (id, client) in clients.lock().await.iter() {
                            client_ids.push(*id);
                            // Log and ignore send errors
                            if client.send(broadcast_msg.clone()).is_err() {
                                error!("Failed to send message to client {}", id);
                            }
                        }
                        // Log the list of connected client IDs
                        info!("Connected clients: {:?}", client_ids);
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
    });

    // Spawn a task to write messages to the WebSocket
    let sender_task = spawn(async move {
        while let Ok(message) = client_tx.subscribe().recv().await {
            if let Err(e) = client_ws_tx.send(Message::text(message)).await {
                error!("Failed to send message: {}", e);
                break;
            }
        }
    });

    // Wait for both tasks to complete
    let _ = tokio::try_join!(receiver_task, sender_task);
}
