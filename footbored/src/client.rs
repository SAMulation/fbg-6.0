use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;
use warp::ws::{WebSocket, Message};
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use futures::SinkExt;
use futures::StreamExt;

use crate::game_server::{GameServer, ClientMessage};

pub struct Client {
    id: Uuid,
    ws: WebSocket,
    game_server: Arc<Mutex<GameServer>>,
    chat_sender: mpsc::UnboundedSender<ClientMessage>,
}

impl Client {
    pub fn new(ws: WebSocket, game_server: Arc<Mutex<GameServer>>, client_id: Uuid, chat_sender: mpsc::UnboundedSender<ClientMessage>) -> Self {
        Self {
            id: client_id,
            ws,
            game_server,
            chat_sender,
        }
    }

    pub async fn handle_connection(mut self) {
        while let Some(result) = self.ws.next().await {
            match result {
                Ok(message) => {
                    if let Ok(text) = message.to_text() {
                        let msg = ClientMessage::TextMessage {
                            sender_id: self.id,
                            content: text.to_string(),
                        };
                        self.chat_sender.send(msg);
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

        self.ws.send(Message::close_with(CloseCode::Normal)).await;
    }
}
