use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::client::{ClientMessage, ClientAction};

pub struct GameServer {
    clients: Mutex<HashMap<Uuid, mpsc::UnboundedSender<ClientMessage>>>,
}

impl GameServer {
    pub fn new() -> Self {
        GameServer {
            clients: Mutex::new(HashMap::new()),
        }
    }

    pub async fn add_client(&self, client_id: Uuid) -> mpsc::UnboundedSender<ClientMessage> {
        let (sender, receiver) = mpsc::unbounded_channel::<ClientMessage>();
        self.clients.lock().await.insert(client_id, receiver);
        sender
    }

    pub async fn remove_client(&self, client_id: Uuid) {
        self.clients.lock().await.remove(&client_id);
    }

    pub async fn send_message(&self, client_id: Uuid, message: ClientMessage) {
        if let Some(sender) = self.clients.lock().await.get(&client_id) {
            if let Err(error) = sender.send(message) {
                eprintln!("Error sending message to client {}: {:?}", client_id, error);
            }
        }
    }

    pub async fn broadcast_message(&self, sender_id: Uuid, message: ClientMessage) {
        let clients = self.clients.lock().await.clone();
        for (client_id, sender) in clients.iter() {
            if client_id != &sender_id {
                if let Err(error) = sender.send(message.clone()) {
                    eprintln!("Error sending message to client {}: {:?}", client_id, error);
                }
            }
        }
    }

    pub async fn handle_client_action(&self, client_id: Uuid, action: ClientAction) {
        match action {
            ClientAction::ChatMessage(content) => {
                let message = format!("Client {}: {}", client_id, content);
                self.broadcast_message(client_id, ClientMessage::ChatMessage(message)).await;
            }
            ClientAction::GameRequest(request) => {
                // Handle the game request
                // Implement your game request logic here
                println!("Received game request from client {}: {:?}", client_id, request);
            }
            // Handle other client actions if needed
            _ => {
                eprintln!("Invalid client action received from client {}: {:?}", client_id, action);
            }
        }
    }
}
