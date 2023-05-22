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

pub async fn handle_connection(
    ws: WebSocket,
    clients: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>,
    lobby: Arc<Mutex<Lobby>>,
    running: Arc<AtomicBool>,
) {
    let (client_id, client_ws_tx, client_ws_rx) = initialize_connection(ws, clients.clone()).await;

    let handle_message_task = spawn(handle_message(client_id, client_ws_rx, Arc::clone(&clients), Arc::new(Mutex::new(client_ws_tx)), Arc::clone(&lobby), Arc::clone(&running)));
    let handle_send_task = spawn(handle_send(client_id, Arc::clone(&clients)));

    tokio::select! {
        _ = handle_message_task => (),
        _ = handle_send_task => (),
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
                    if text.starts_with("/message") {
                        let chat_message = text.to_string();
                        lobby.lock().await.broadcast_message(chat_message, &clients).await;
                    } else if text.starts_with("/request_game") {
                        // Placeholder for requesting a game
                    } else if text.starts_with("/leave_lobby") {
                        // Placeholder for leaving the lobby gracefully
                    } else if text.starts_with("/join_lobby") {
                        handle_join_lobby(text, &mut *client_ws_tx.lock().await, &lobby).await;
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

async fn handle_send(
    client_id: Uuid,
    clients: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>,
) {
    let mut broadcast_receiver = clients.lock().await[&client_id].subscribe();
    while let Ok(message) = broadcast_receiver.recv().await {
        if let Some(client_ws_tx) = clients.lock().await.get_mut(&client_id) {
            if let Err(e) = client_ws_tx.send(message) {
                error!("Failed to send message: {}", e);
                break;
            }            
        }
    }
}

async fn handle_join_lobby(
    message: &str,
    client_ws_tx: &mut SplitSink<WebSocket, Message>,
    lobby: &Arc<Mutex<Lobby>>,
) {
    let player_name = message.trim_start_matches("/join_lobby").trim();
    let player_id = Uuid::new_v4();
    let player = Player {
        id: player_id,
        name: player_name.to_string(),
    };
    lobby.lock().await.join_lobby(player);

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
