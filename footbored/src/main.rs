use std::sync::Arc;
use std::collections::HashMap;
use warp::Filter;
use tokio::sync::Mutex;
use uuid::Uuid;
use futures::{SinkExt, StreamExt};
use warp::ws::{Message, WebSocket};
use tokio::task::spawn;

#[tokio::main]
async fn main() {
    let clients = Arc::new(Mutex::new(HashMap::new()));

    let routes = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let clients = clients.clone();
            ws.on_upgrade(move |socket| handle_connection(socket, clients))
        });

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

async fn handle_connection(ws: WebSocket, clients: Arc<Mutex<HashMap<Uuid, futures::channel::mpsc::UnboundedSender<Message>>>>) {
    let (mut client_ws_tx, mut client_ws_rx) = ws.split();
    let (client_tx, mut client_rx) = futures::channel::mpsc::unbounded();
    let client_id = Uuid::new_v4();
    clients.lock().await.insert(client_id, client_tx);

    let client_id_clone = client_id;
    let clients_clone = clients.clone();

    let receiver_task = spawn(async move {
        while let Some(Ok(message)) = client_ws_rx.next().await {
            if let Ok(text) = message.to_str() {
                let broadcast_msg = format!("Client {}: {}", client_id_clone, text);
                for (_, client) in clients_clone.lock().await.iter() {
                    let _ = client.unbounded_send(Message::text(broadcast_msg.clone()));
                }
            }
        }
    });

    let sender_task = spawn(async move {
        while let Some(message) = client_rx.next().await {
            let _ = client_ws_tx.send(message).await;
        }
    });

    let _ = futures::future::join(receiver_task, sender_task).await;
}
