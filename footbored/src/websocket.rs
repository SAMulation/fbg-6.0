use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{Mutex, broadcast};
use tokio::task::spawn;
use uuid::Uuid;
use log::{info, warn, error};
use warp::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use crate::lobby::{Lobby, Player};
use futures::stream::SplitSink;
use futures::stream::SplitStream;
use std::sync::atomic::{AtomicBool, Ordering};
use serde_json;

pub async fn handle_connection(
    ws: WebSocket,
    clients: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>,
    lobby: Arc<Mutex<Lobby>>,
    running: Arc<AtomicBool>,
) {
    let (client_id, client_ws_tx, client_ws_rx) = initialize_connection(ws, clients.clone()).await;

    let handle_message_task = spawn(handle_message(client_id, client_ws_rx, Arc::clone(&clients), Arc::new(Mutex::new(client_ws_tx)), Arc::clone(&lobby), Arc::clone(&running)));
    let handle_send_task = spawn(handle_send(client_id, Arc::clone(&clients)));

    let result = tokio::try_join!(handle_message_task, handle_send_task);

    match result {
        Ok(_) => info!("Both tasks completed successfully for client {}", client_id),
        Err(e) => warn!("One of the tasks for client {} returned an error: {}", client_id, e),
    }
    
    cleanup_connection(client_id, clients, running).await;
    
}

async fn initialize_connection(
    ws: WebSocket,
    clients: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>,
) -> (Uuid, SplitSink<WebSocket, Message>, SplitStream<WebSocket>) {
    let (client_ws_tx, client_ws_rx) = ws.split();
    let client_id = Uuid::new_v4();
    clients.lock().await.insert(client_id, broadcast::channel::<String>(10).0);
    (client_id, client_ws_tx, client_ws_rx)
}

async fn handle_message(
    client_id: Uuid,
    mut client_ws_rx: SplitStream<WebSocket>,
    clients: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>,
    client_ws_tx: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    lobby: Arc<Mutex<Lobby>>,
    running: Arc<AtomicBool>,
) {
    while let Some(result) = client_ws_rx.next().await {
        match result {
            Ok(message) => {
                if let Ok(text) = message.to_str() {
                    info!("Received message from client {}: {}", client_id, text);
                    if text.starts_with("/message ") {
                        let chat_message = text.trim_start_matches("/message ").to_string();
                        lobby.lock().await.broadcast_message(chat_message, &clients).await;
                    }
                     else if text.starts_with("/request_game") {
                        // Placeholder for requesting a game
                    } else if text.starts_with("/leave_lobby") {
                        let player_id = text.trim_start_matches("/leave_lobby").trim().parse::<Uuid>();
                        if let Ok(player_id) = player_id {
                            lobby.lock().await.leave_lobby(player_id, &clients).await;
                        }
                    } else if text.starts_with("/join_lobby") {
                        handle_join_lobby(text, &mut *client_ws_tx.lock().await, &lobby, &clients).await;
                    } else if text.starts_with("/players") {
                        handle_players(client_ws_tx.clone(), clients.clone()).await;
                    }
                }
            }
            Err(e) => {
                warn!("Error receiving message: {}", e);
                break;
            }
        }

        if !running.load(Ordering::Relaxed) {
            break;
        }
    }

    info!("Client {} disconnected", client_id);
    clients.lock().await.remove(&client_id);
}


async fn handle_send(client_id: Uuid, clients: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>) {
    let mut receiver = {
        let guard = clients.lock().await;
        guard.get(&client_id).unwrap().subscribe()
    };

    while let Ok(message) = receiver.recv().await {
        let client = {
            let guard = clients.lock().await;
            guard.get(&client_id).cloned()
        };

        if let Some(client) = client {
            match client.send(message.clone()) {
                Ok(_) => info!("Sent message to client {}: {}", client_id, message),
                Err(e) => error!("Failed to send message: {}", e),
            }
        } else {
            warn!("Client {} has disconnected", client_id);
            break;
        }
    }

    info!("Ending send task for client {}", client_id);
}

async fn handle_join_lobby(
    message: &str,
    client_ws_tx: &mut SplitSink<WebSocket, Message>,
    lobby: &Arc<Mutex<Lobby>>,
    clients: &Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>,
) {
    let player_name = message.trim_start_matches("/join_lobby").trim();
    let player_id = Uuid::new_v4();
    let player = Player {
        id: player_id,
        name: player_name.to_string(),
    };
    lobby.lock().await.join_lobby(player, &clients).await;

    let join_message = format!("You have joined the lobby as '{}'", player_name);
    let _ = client_ws_tx.send(Message::text(join_message)).await;
}

async fn cleanup_connection(
    client_id: Uuid,
    clients: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>,
    running: Arc<AtomicBool>,
) {
    info!("Client {} disconnected", client_id);
    clients.lock().await.remove(&client_id);

    if clients.lock().await.is_empty() {
        running.store(false, Ordering::Relaxed);
    }
}

async fn get_player_list(
    clients: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>
) -> String {
    let guard = clients.lock().await;
    let player_list: Vec<String> = guard.keys().map(|k| k.to_string()).collect();
    serde_json::to_string(&player_list).unwrap_or_else(|_| "".to_string())
}

pub async fn handle_players(
    client_ws_tx: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    clients: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>
) {
    let player_list = get_player_list(clients.clone()).await;
    if let Err(e) = client_ws_tx.lock().await.send(Message::text(player_list)).await {
        error!("Failed to send player list: {}", e);
        return;
    }

    // Subscribe the client to the player updates channel
    let (player_update_tx, mut player_update_rx) = broadcast::channel::<String>(10);
    clients.lock().await.insert(Uuid::new_v4(), player_update_tx);

    // Listen for player updates and send them to the client
    while let Ok(result) = player_update_rx.recv().await {
        if let Ok(update_message) = serde_json::to_string(&result) {
            if let Err(e) = client_ws_tx.lock().await.send(Message::text(update_message)).await {
                error!("Failed to send player update: {}", e);
                break;
            }
        } else {
            error!("Failed to serialize player update");
            break;
        }
    }
}