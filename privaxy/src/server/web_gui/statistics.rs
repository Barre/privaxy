use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::sleep;
use warp::ws::{Message, WebSocket};

use crate::statistics::Statistics;

pub(super) async fn statistics(websocket: WebSocket, statistics: Statistics) {
    let (mut tx, mut rx) = websocket.split();

    // To handle Ping / Pong messages
    tokio::spawn(async move { while let Some(_message) = rx.next().await {} });

    // When the client first visits the dashboard, we want to send a first message immediately.
    let mut last_message =
        Message::text(serde_json::to_string(&statistics.get_serialized()).unwrap());

    let _result = tx.send(last_message.clone()).await;

    loop {
        let message = Message::text(serde_json::to_string(&statistics.get_serialized()).unwrap());

        // Let's not send the same message over and over again.
        if message != last_message && tx.send(message.clone()).await.is_err() {
            break;
        }

        last_message = message;

        sleep(Duration::from_millis(500)).await;
    }
}
