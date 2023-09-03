use std::time::Duration;

use actix_web::web::Bytes;
use awc::ws;
use futures_util::{SinkExt, StreamExt as _};
use tokio::{
    select,
    time::{sleep, timeout},
};

const TIMEOUT_SECONDS: Duration = Duration::from_secs(30);

#[actix_web::main]
async fn main() {
    let (_, mut ws) = awc::Client::new()
        .ws("ws://127.0.0.1:8080/ws")
        .connect()
        .await
        .unwrap();

    let main_task = async move {
        loop {
            select! {
              Some(websocket_message) = ws.next() => {
                match websocket_message {
                  Ok(ws::Frame::Ping(_)) => {
                    ws.send(ws::Message::Pong(Bytes::new())).await.unwrap();
                  }

                  Ok(ws::Frame::Text(text_received)) => {
                    println!("{:?}", text_received);
                  },

                  _ => ()
                }
              },

              _ = sleep(TIMEOUT_SECONDS) => {
                break;
              }
            }
        }
    };

    if let Err(_) = timeout(TIMEOUT_SECONDS, main_task).await {
        println!("Terminated after timeout.");
    }
}
