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
        .with(cors);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
