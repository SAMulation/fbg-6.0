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

    // pub fn send_game_request(&self, request: GameRequest) {
    //     // Add logic to send game request to the intended recipient
    //     // For now, let's just print a message indicating the game request
    //     println!("Game request sent from {:?} to {:?}", request.from, request.to);
    // }

    pub async fn broadcast_message(
        &self,
        message: String,
        clients: &Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>
    ) {
        let guard = clients.lock().await.clone();
        for client in guard.values() {
            let _ = client.send(message.clone());
        }
    }

    pub async fn join_lobby(
        &mut self,
        player: Player,
        clients: &Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>
    ) {
        self.players.insert(player.id, player);

        let player_list = get_player_list(&self.players);
        let _ = clients.lock().await.iter().for_each(|(_, tx)| {
            let _ = tx.send(player_list.clone());
        });
    }

    pub async fn leave_lobby(
        &mut self,
        player_id: Uuid,
        clients: &Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>
    ) {
        self.players.remove(&player_id);

        let player_list = get_player_list(&self.players);
        let _ = clients.lock().await.iter().for_each(|(_, tx)| {
            let _ = tx.send(player_list.clone());
        });
    }
}

pub async fn start_lobby() -> Arc<Mutex<Lobby>> {
    let lobby = Arc::new(Mutex::new(Lobby::new()));
    info!("Lobby started");
    lobby
}

fn get_player_list(players: &HashMap<Uuid, Player>) -> String {
    let player_list: Vec<String> = players.keys().map(|k| k.to_string()).collect();
    serde_json::to_string(&player_list).unwrap_or_else(|_| "".to_string())
}
