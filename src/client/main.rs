mod ace_state;
mod handlers;
mod instructions;
mod instructions_task;
mod messages_task;
mod minecraft_resource;
mod vectors;

use std::time::Duration;

use ace_state::AceState;
use actix_web::web::Bytes;
use actix_web_actors::ws;
use futures_util::{SinkExt, StreamExt as _};
use handlers::message::Message;
use minecraft_resource::MinecraftResource;
use serde_json::Value;
use tokio::{
    select,
    sync::broadcast,
    time::{sleep, timeout},
};

use crate::minecraft_resource::{PlayerHead, PlayerPosition};

const TIMEOUT_DURATION: Duration = Duration::from_secs(1200);
const TICKRATE_DURATION: Duration = Duration::from_millis(60);
const WEBSOCKET_URL: &str = "ws://127.0.0.1:8080/ws";
const DISTANCE_TO_FINISH_INSTRUCTION: f64 = 0.3;

#[actix_web::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Message>(100);

    let (_, mut ws) = awc::Client::new()
        .ws(WEBSOCKET_URL)
        .connect()
        .await
        .unwrap();

    let ace_state_shared = AceState::new_shared();
    let minecraft_resource_shared = MinecraftResource::new_shared();

    let ws_minecraft_resource = minecraft_resource_shared.clone();

    let main_task = async move {
        messages_task::spawn(ace_state_shared.clone(), tx.clone());
        instructions_task::spawn(minecraft_resource_shared.clone(), tx.clone());

        loop {
            select! {
              Some(websocket_message) = ws.next() => {
                match websocket_message {
                  Ok(ws::Frame::Ping(_)) => ws.send(ws::Message::Pong(Bytes::new())).await.unwrap(),
                  Ok(ws::Frame::Text(text_received)) => {
                    let output: Value = serde_json::from_slice(&text_received).unwrap();

                    let mut minecraft_resource = ws_minecraft_resource.lock().await;

                    minecraft_resource.player_position = PlayerPosition::from((
                      output["coords"]["x"].as_f64().unwrap(),
                      output["coords"]["y"].as_f64().unwrap(),
                      output["coords"]["z"].as_f64().unwrap(),
                    ));

                    minecraft_resource.player_head = PlayerHead {
                      yaw: output["head"]["yaw"].as_f64().unwrap(),
                      y: output["head"]["y"].as_f64().unwrap(),
                    };
                  },

                  _ => ()
                }
              },

              _ = sleep(TIMEOUT_DURATION) => {
                break;
              }
            }
        }
    };

    if let Err(_) = timeout(TIMEOUT_DURATION, main_task).await {
        println!("Terminated after timeout.");
    }
}
