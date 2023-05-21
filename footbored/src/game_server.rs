// Import the necessary modules for our application
use std::collections::HashMap; // HashMap for client, chat sender and game state management
use std::sync::{Arc, Mutex}; // Arc and Mutex for shared state across threads
use tokio::sync::mpsc; // mpsc (multiple producer, single consumer) for async communication
use uuid::Uuid; // Uuid for unique client identification
use crate::client::{Client, ClientAction, ClientMessage}; // Client struct
use crate::game::Game; // Game related structures

// The structure of the game server containing clients, chat_senders, and games.
pub struct GameServer {
    // Clients are represented as a HashMap with UUIDs as keys and Client as values. 
    // The HashMap is wrapped in an Arc Mutex for shared state.
    clients: Arc<Mutex<HashMap<Uuid, Client>>>,
    // Chat_senders are also represented as a HashMap with UUIDs as keys and mpsc::UnboundedSender<String> as values.
    // This structure is used for sending chat messages.
    chat_senders: Arc<Mutex<HashMap<Uuid, mpsc::UnboundedSender<String>>>>,
    // Games are represented as a HashMap with UUIDs as keys and Game as values.
    // This structure represents the current games in progress.
    games: Arc<Mutex<HashMap<Uuid, Game>>>,
}

// Methods for the GameServer struct
impl GameServer {
    // The constructor method for the GameServer. This method initializes empty HashMaps for clients, chat_senders, and games.
    pub fn new() -> Self {
        GameServer {
            clients: Arc::new(Mutex::new(HashMap::new())),
            chat_senders: Arc::new(Mutex::new(HashMap::new())),
            games: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Adds a new client to the server
    pub fn add_client(&mut self, client: Client) {
        let client_id = client.id;
        self.clients.lock().unwrap().insert(client_id, client);
    }

    // Removes a client from the server
    pub fn remove_client(&mut self, client_id: Uuid) {
        self.clients.lock().unwrap().remove(&client_id);
        self.chat_senders.lock().unwrap().remove(&client_id);
    }

    // Sends a message to a specific client
    pub fn send_message(&self, client_id: Uuid, message: ClientMessage) {
        if let Some(client) = self.clients.lock().unwrap().get(&client_id) {
            if let Err(error) = client.send_message(message) {
                eprintln!("Error sending message to client {}: {:?}", client_id, error);
            }
        }
    }

    // Broadcasts a message to all clients except the sender
    pub fn broadcast_message(&self, sender_id: Uuid, message: ClientMessage) {
        let clients = self.clients.lock().unwrap();
        for (client_id, client) in clients.iter() {
            if client_id != &sender_id {
                if let Err(error) = client.send_message(message.clone()) {
                    eprintln!("Error sending message to client {}: {:?}", client_id, error);
                }
            }
        }
    }

    // Broadcasts a chat message to all players except the sender
    pub fn broadcast_chat_message(&self, sender_id: Uuid, content: String) {
        let message = format!("{}: {}", sender_id, content);
        let chat_senders = self.chat_senders.lock().unwrap();
        for (player_id, chat_sender) in chat_senders.iter() {
            if player_id != &sender_id {
                let _ = chat_sender.send(message.clone());
            }
        }
    }

    // Handles actions performed by clients
    pub fn handle_action(&self, client_id: Uuid, action: ClientAction) {
        match action {
            ClientAction::ChatMessage { content } => {
                self.broadcast_chat_message(client_id, content);
            }
            ClientAction::MakeMove { game_id, player_id, x, y } => {
                self.make_move(game_id, player_id, x, y);
                let games = self.games.lock().unwrap();
                if let Some(game) = games.get(&game_id) {
                    // Send game state update to players
                    let message = ClientMessage::GameStateUpdate(game.to_string());
                    self.send_message(player_id, message.clone());
                    let opponent_id = game.players.iter().find(|&&id| id != player_id);
                    if let Some(&id) = opponent_id {
                        self.send_message(id, message);
                    }
                }
            }
        }
    }

    // Creates a new game with two players and returns the game's UUID
    pub fn create_game(&mut self, player1_id: Uuid, player2_id: Uuid) -> Uuid {
        let game = Game::new(player1_id, player2_id);
        let game_id = Uuid::new_v4();
        self.games.lock().unwrap().insert(game_id, game);
        game_id
    }

    // Makes a move in a game
    pub fn make_move(&mut self, game_id: Uuid, player_id: Uuid, x: usize, y: usize) {
        let mut games = self.games.lock().unwrap();
        if let Some(game) = games.get_mut(&game_id) {
            game.play(player_id, x, y);
        }
    }
}
