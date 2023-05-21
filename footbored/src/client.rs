// Import the necessary modules for our application
use std::error::Error; // For general error handling
use std::sync::Arc; // Arc for shared ownership
use tokio::sync::{mpsc, Mutex}; // mpsc for async communication and Mutex for shared state
use tokio::task; // For spawning tasks
use uuid::Uuid; // Uuid for unique client identification
use warp::ws::WebSocket; // WebSocket for client connection
use tokio_tungstenite::WebSocketStream; // WebSocketStream for handling messages from WebSocket
use futures::StreamExt; // For async stream handling
use serde::{Deserialize, Serialize};

use crate::game_server::GameServer; // Importing the GameServer structure

// Client struct holds the client id, WebSocket, reference to game server, and chat sender channel.
pub struct Client {
    pub id: Uuid,
    pub ws: WebSocket,
    pub game_server: Arc<Mutex<GameServer>>,
    pub chat_sender: Option<mpsc::UnboundedSender<String>>,
}

// Methods for the Client struct
impl Client {
    // Method to start listening to chat messages.
    // It removes the chat_sender from the client (making sure it's only consumed here)
    // and then loops, awaiting messages and forwarding them to the client's WebSocket.
    pub async fn start_chat_listener(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut chat_receiver = self.chat_sender.take().unwrap();

        while let Some(message) = chat_receiver.recv().await {
            self.send_message(ClientMessage::ChatMessage(message.clone())).await?;
        }

        Ok(())
    }

    // Sends a message to the client by converting the message to a JSON string
    // and sending it through the WebSocket.
    pub async fn send_message(&mut self, message: ClientMessage) -> Result<(), Box<dyn Error + Send + Sync>> {
        let message = serde_json::to_string(&message)?;
        self.ws.send(warp::ws::Message::text(message)).await?;
        Ok(())
    }

    // Handles a new WebSocket connection.
    // Splits the WebSocket into a sender and receiver,
    // creates a new client and adds it to the game server,
    // and spawns tasks for handling incoming messages and chat listening.
    // Once either task completes, it removes the client from the game server.
    pub async fn handle_connection(ws: WebSocket, game_server: Arc<Mutex<GameServer>>) {
        let (ws_tx, ws_rx) = ws.split();
        let (chat_sender, chat_receiver) = mpsc::unbounded_channel();
        let client_id = Uuid::new_v4();

        let client = Client {
            id: client_id,
            ws: ws_tx,
            game_server: game_server.clone(),
            chat_sender: Some(chat_sender),
        };

        game_server.lock().await.add_client(client.clone());

        let handle_connection = task::spawn(client.handle_messages(ws_rx));
        let chat_listener = task::spawn(client.start_chat_listener());

        tokio::select! {
            _ = handle_connection => (),
            _ = chat_listener => (),
        }

        game_server.lock().await.remove_client(client_id);
    }

    // Handles incoming messages from the WebSocket.
    // If a message is received, it attempts to convert it to a ClientAction and handle it.
    // If an error occurs, it prints an error message and breaks the loop.
    pub async fn handle_messages(&self, mut ws_rx: WebSocketStream) {
        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(message) => {
                    if let Ok(text) = message.to_str() {
                        if let Ok(action) = serde_json::from_str::<ClientAction>(text) {
                            self.game_server.lock().await.handle_action(self.id, action);
                        } else {
                            eprintln!("Invalid client action received from client {}: {}", self.id, text);
                        }
                    } else {
                        eprintln!("Invalid message received from client {}: {:?}", self.id, message);
                    }
                }
                Err(error) => {
                    eprintln!("Error receiving message from client {}: {}", self.id, error);
                    break;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    ChatMessage(String),
    // Add more variants for different types of messages as needed
    GameStateUpdate(String),
    // Add more variants for different types of messages as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientAction {
    ChatMessage {
        content: String,
    },
    MakeMove {
        game_id: Uuid,
        player_id: Uuid,
        x: usize,
        y: usize,
    },
    // Add more variants for different client actions as needed
}

impl ClientMessage {
    // Utility method to create a chat message
    pub fn chat_message(content: String) -> Self {
        ClientMessage::ChatMessage(content)
    }

    // Utility method to create a game state update message
    pub fn game_state_update(state: String) -> Self {
        ClientMessage::GameStateUpdate(state)
    }
}

impl ClientAction {
    // Utility method to create a chat action
    pub fn chat_message(content: String) -> Self {
        ClientAction::ChatMessage { content }
    }

    // Utility method to create a make move action
    pub fn make_move(game_id: Uuid, player_id: Uuid, x: usize, y: usize) -> Self {
        ClientAction::MakeMove {
            game_id,
            player_id,
            x,
            y,
        }
    }
}
