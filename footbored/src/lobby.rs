use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use uuid::Uuid;
use log::info;
use tokio::sync::broadcast;


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

    pub async fn broadcast_message(&self, message: String, clients: &Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>) {
        // Obtain a clone of the clients' MutexGuard
        let guard = clients.lock().await.clone();
        for client in guard.values() {
            // Ignore any errors that occur while sending the message
            let _ = client.send(message.clone());
        }
    }
    
    
    
    
    
    pub fn join_lobby(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }
}

pub async fn start_lobby() -> Arc<Mutex<Lobby>> {
    let lobby = Arc::new(Mutex::new(Lobby::new()));
    info!("Lobby started");
    lobby
}
