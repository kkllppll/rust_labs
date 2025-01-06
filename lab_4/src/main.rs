use warp::ws::{Message, WebSocket};
use warp::Filter;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use std::sync::{Arc, Mutex};

type Users = Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
    let users = Users::default();
    let users_filter = warp::any().map(move || users.clone());

    let chat = warp::path("ws")
        .and(warp::ws())
        .and(users_filter)
        .map(|ws: warp::ws::Ws, users| {
            ws.on_upgrade(move |socket| handle_connection(socket, users))
        });

    println!("Server running on ws://127.0.0.1:3030/ws");
    warp::serve(chat).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_connection(ws: WebSocket, users: Users) {
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let rx_stream = UnboundedReceiverStream::new(rx);
    tokio::spawn(rx_stream.map(Ok).forward(user_ws_tx));

    users.lock().unwrap().push(tx);

    while let Some(Ok(msg)) = user_ws_rx.next().await {
        if let Ok(text) = msg.to_str() {
            broadcast_message(text.to_string(), &users).await;
        }
    }
}

async fn broadcast_message(msg: String, users: &Users) {
    let mut locked = users.lock().unwrap();
    locked.retain(|user| user.send(Message::text(msg.clone())).is_ok());
}
