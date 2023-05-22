use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use uuid::Uuid;
use log::info;

#[derive(Debug)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug)]
pub struct GameRequest {
    pub from: Uuid,
    pub to: Uuid,
    // Add any other relevant data for game requests
}

#[derive(Debug)]
pub struct Lobby {
    players: HashMap<Uuid, Player>,
    // Add any other relevant data for the lobby
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    pub fn remove_player(&mut self, player_id: Uuid) {
        self.players.remove(&player_id);
    }

    pub fn send_game_request(&self, request: GameRequest) {
        // Add logic to send game request to the intended recipient
        // For now, let's just print a message indicating the game request
        println!("Game request sent from {:?} to {:?}", request.from, request.to);
    }

    pub fn broadcast_message(&self, message: String) {
        // Add logic to broadcast the message to all players in the lobby
        // For now, let's just print the message for each player
        for player in self.players.values() {
            println!("Broadcasting message to player {:?}: {}", player.id, message);
        }
    }
}

pub async fn start_lobby() -> Arc<Mutex<Lobby>> {
    let lobby = Arc::new(Mutex::new(Lobby::new()));
    info!("Lobby started");
    lobby
}
