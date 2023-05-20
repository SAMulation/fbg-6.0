use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use crate::game::Game;

pub struct MyWebSocket {
    game: Game,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // Assume the text is in the format "x,y", parse into coordinates
                let coords: Vec<usize> = text.split(',')
                    .map(|x| x.trim().parse().unwrap_or(0))
                    .collect();

                if coords.len() == 2 {
                    let play_result = self.game.play(coords[0], coords[1]);
                    
                    // If the play was successful, send the updated game state to the client
                    if play_result {
                        let game_state = self.game.to_string();
                        ctx.text(game_state);
                    } else {
                        ctx.text("Invalid move".to_string());
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

pub async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let game = Game::new();
    let resp = ws::start(MyWebSocket { game }, &req, stream);
    println!("{:?}", resp);
    resp
}
