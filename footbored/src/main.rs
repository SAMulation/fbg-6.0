#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use serde_derive::{Serialize, Deserialize};
mod game;
use game::{Game, Cell};
use warp::{self, Filter, Reply};

lazy_static! {
    static ref GAMES: Arc<Mutex<HashMap<Uuid, Game>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref PLAYERS: Arc<Mutex<HashMap<Uuid, Player>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref PLAYER_REQUESTS: Arc<Mutex<HashMap<Uuid, Vec<Uuid>>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Deserialize)]
struct MovePosition {
    x: usize,
    y: usize,
}

#[derive(Serialize)]
struct GameResponse {
    game_state: String,
    game_id: Uuid,
    game_over: bool,
    winner: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize, Deserialize)]
struct PlayerRequest {
    player_id: Uuid,
    game_id: Uuid,
}

#[derive(Serialize, Deserialize)]
struct GameRequest {
    player_id: Uuid,
    opponent_id: Uuid,
    game_id: Uuid,
}

#[derive(Serialize, Deserialize)]
struct GameRequestResponse {
    game_id: Uuid,
    player_id: Uuid,
    response: RequestResponse,
}

#[derive(Serialize, Deserialize, PartialEq)]
enum RequestResponse {
    Approve,
    Deny,
}

#[derive(Clone)]
struct Player {
    id: Uuid,
    requested_game_id: Option<Uuid>,
    playing_game_id: Option<Uuid>,
}

#[tokio::main]
async fn main() {
    let new_game = warp::path!("new_game")
        .and(warp::post())
        .map(|| {
            let mut games = GAMES.lock().unwrap();
            let game_id = Uuid::new_v4();
            games.insert(game_id, Game::new());
            warp::reply::json(&GameResponse {
                game_state: games.get(&game_id).unwrap().to_string(),
                game_id,
                game_over: false,
                winner: None,
            })
        });

    let make_move = warp::path!("game" / Uuid / "move")
        .and(warp::filters::method::method())
        .and(warp::body::json())
        .map(|game_id: Uuid, method: warp::http::Method, move_position: MovePosition| {
            if method == warp::http::Method::OPTIONS {
                // Return a response with the necessary CORS headers for the OPTIONS request
                warp::reply::with_header(
                    "",
                    "Access-Control-Allow-Origin",
                    "https://tictac.thencandesigns.com",
                )
                .into_response()
            } else if method == warp::http::Method::POST {
                // Process the actual move request
                let mut games = GAMES.lock().unwrap();
                if let Some(game) = games.get_mut(&game_id) {
                    if game.play(move_position.x, move_position.y) {
                        let game_over = game.is_over();
                        let winner = match game.get_winner() {
                            Some(Cell::X) => Some("X".to_string()),
                            Some(Cell::O) => Some("O".to_string()),
                            _ => None,
                        };
                        warp::reply::json(&GameResponse {
                            game_state: game.to_string(),
                            game_id,
                            game_over,
                            winner,
                        })
                        .into_response()
                    } else {
                        warp::reply::json(&ErrorResponse {
                            error: String::from("Invalid move"),
                        })
                        .into_response()
                    }
                } else {
                    warp::reply::json(&ErrorResponse {
                        error: String::from("Game not found"),
                    })
                    .into_response()
                }
            } else {
                // Handle other methods if needed
                warp::reply::with_status(
                    "",
                    warp::http::StatusCode::METHOD_NOT_ALLOWED,
                )
                .into_response()
            }
        });

    let game_state = warp::path!("game" / Uuid / "state")
        .and(warp::get())
        .map(|game_id: Uuid| {
            let games = GAMES.lock().unwrap();
            if let Some(game) = games.get(&game_id) {
                let game_over = game.is_over();
                let winner = match game.get_winner() {
                    Some(Cell::X) => Some("X".to_string()),
                    Some(Cell::O) => Some("O".to_string()),
                    _ => None,
                };
                warp::reply::json(&GameResponse {
                    game_state: game.to_string(),
                    game_id,
                    game_over,
                    winner,
                })
            } else {
                warp::reply::json(&ErrorResponse {
                    error: String::from("Game not found"),
                })
            }
        });

    let join_game = warp::path!("join_game")
        .and(warp::post())
        .and(warp::body::json())
        .map(|player_name: String| {
            let mut players = PLAYERS.lock().unwrap();
            let player_id = Uuid::new_v4();
            let player = Player {
                id: player_id,
                requested_game_id: None,
                playing_game_id: None,
            };
            players.insert(player_id, player);
            warp::reply::json(&player_id)
        });

    let request_game = warp::path!("request_game")
        .and(warp::post())
        .and(warp::body::json())
        .map(|game_request: GameRequest| {
            let mut player_requests = PLAYER_REQUESTS.lock().unwrap();
            
            let requesting_player = PLAYERS.lock().unwrap().get(&game_request.player_id).cloned();
            let opponent_player = PLAYERS.lock().unwrap().get(&game_request.opponent_id).cloned();
            
            if let (Some(mut requesting_player), Some(mut opponent_player)) = (requesting_player, opponent_player) {
                requesting_player.requested_game_id = Some(game_request.game_id);
                let requests = player_requests.entry(game_request.game_id).or_insert(Vec::new());
                requests.push(requesting_player.id);
                
                warp::reply::json(&RequestResponse::Approve)
            } else {
                warp::reply::json(&RequestResponse::Deny)
            }
        });

    let handle_game_request = warp::path!("handle_game_request")
        .and(warp::post())
        .and(warp::body::json())
        .map(|game_request_response: GameRequestResponse| {
            let mut players = PLAYERS.lock().unwrap();
            let mut player_requests = PLAYER_REQUESTS.lock().unwrap();
            let requesting_player = players.get_mut(&game_request_response.player_id);
            if let Some(requesting_player) = requesting_player {
                if game_request_response.response == RequestResponse::Approve {
                    if let Some(requested_game_id) = requesting_player.requested_game_id {
                        let requests = player_requests.get_mut(&requested_game_id);
                        if let Some(requests) = requests {
                            let index = requests.iter().position(|&id| id == requesting_player.id);
                            if let Some(index) = index {
                                requests.remove(index);
                                requesting_player.playing_game_id = Some(requested_game_id);
                            }
                        }
                    }
                }
                requesting_player.requested_game_id = None;
            }
            warp::reply()
        });

    let cors = warp::cors()
        .allow_origins(vec![
            "https://tictac.thencandesigns.com",
            "https://server.thencandesigns.com",
        ])
        .allow_methods(vec!["POST", "GET"])
        .allow_headers(vec!["Content-Type"]);

    let routes = new_game
        .or(make_move)
        .or(game_state)
        .or(join_game)
        .or(request_game)
        .or(handle_game_request)
        .with(cors);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
