use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::sync::broadcast;
use warp::ws::{Message, WebSocket};

#[derive(Debug, Serialize, Clone)]
pub(crate) struct Event {
    pub now: DateTime<Utc>,
    pub method: String,
    pub url: String,
    pub is_request_blocked: bool,
}

pub(super) async fn events(websocket: WebSocket, events_sender: broadcast::Sender<Event>) {
    let mut events_receiver = events_sender.subscribe();

    let (mut tx, mut rx) = websocket.split();

    // To handle Ping / Pong messages
    tokio::spawn(async move { while let Some(_message) = rx.next().await {} });

    while let Ok(event) = events_receiver.recv().await {
        let message = Message::text(serde_json::to_string(&event).unwrap());

        if let Err(_err) = tx.send(message).await {
            break;
        }
    }
}
