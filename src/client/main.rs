mod ace_state;
mod instructions;
mod minecraft_resource;
mod vectors;

use std::{sync::Arc, time::Duration};

use ace_state::AceState;
use actix_web::web::Bytes;
use awc::ws;
use enigo::*;
use futures_util::{lock::Mutex, SinkExt, StreamExt as _};
use instructions::{list_instructions, Instruction};
use minecraft_resource::{MinecraftResource, PlayerPosition};
use serde_json::Value;
use tokio::{
    select,
    sync::broadcast,
    time::{sleep, timeout},
};

use crate::{
    minecraft_resource::PlayerHead,
    vectors::{CalculateAngleForce, Vector3D},
};

const TIMEOUT_SECONDS_DURATION: Duration = Duration::from_secs(50);
const TICKRATE_DURATION: Duration = Duration::from_millis(50);
const WEBSOCKET_URL: &str = "ws://127.0.0.1:8080/ws";

#[derive(Debug, Clone)]
enum Message {
    InstructionFinished,
    HeadMovement {
        horizontal_force: i32,
        vertical_force: i32,
    },
    Walk {
        distance: f64,
        head_movement_force: (i32, i32),
        instruction: Instruction,
    },
    Hand {
        instruction: Instruction,
    },
}

#[actix_web::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Message>(100);

    let (_, mut ws) = awc::Client::new()
        .ws(WEBSOCKET_URL)
        .connect()
        .await
        .unwrap();

    let ace_state = Arc::new(Mutex::new(AceState::new()));
    let enigo = Arc::new(Mutex::new(Enigo::new()));
    let minecraft_resource = Arc::new(Mutex::new(MinecraftResource::new()));

    let mut instructions = list_instructions();
    instructions.reverse();
    let instructions = Arc::new(Mutex::new(instructions));

    let mut foot_receiver = tx.subscribe();

    let foot_enigo = enigo.clone();

    let minecraft_resource1 = minecraft_resource.clone();
    let minecraft_resource2 = minecraft_resource.clone();

    let foot_ace_state = ace_state.clone();

    let main_task = async move {
        // thread for foot movement
        tokio::spawn(async move {
            loop {
                let message = foot_receiver.recv().await.unwrap();
                let mut enigo = foot_enigo.lock().await;
                let mut ace_state = foot_ace_state.lock().await;

                match message {
                    Message::HeadMovement {
                        horizontal_force,
                        vertical_force,
                    } => {
                        enigo.mouse_move_relative(horizontal_force * -1, vertical_force * -1);
                    }

                    Message::Walk {
                        distance,
                        instruction,
                        head_movement_force,
                    } => {
                        if instruction.rotate_before_walk
                            && (head_movement_force.0 != 0 || head_movement_force.1 != 0)
                        {
                            continue;
                        }

                        if distance < 0.2 && ace_state.is_walking {
                            println!("Releasing W");
                            enigo.key_up(Key::W);
                            ace_state.is_walking = false;
                        }

                        if distance > 0.2 && !ace_state.is_walking {
                            if instruction.allow_run && !ace_state.is_running && distance > 5. {
                                println!("Pressing Control");
                                enigo.key_down(Key::Control);
                                ace_state.is_running = true;
                            }

                            if instruction.allow_sneak && !ace_state.is_sneaking && distance < 3. {
                                println!("Pressing Shift");
                                enigo.key_down(Key::Shift);
                                ace_state.is_sneaking = true;
                            }

                            println!("Pressing W");
                            enigo.key_down(Key::W);
                            ace_state.is_walking = true;
                        }

                        if instruction.allow_run && ace_state.is_running && distance < 5. {
                            println!("Releasing Control");
                            enigo.key_up(Key::Control);
                            println!("Repressing W");
                            enigo.key_up(Key::W);
                            sleep(TICKRATE_DURATION).await;
                            enigo.key_down(Key::W);
                            sleep(TICKRATE_DURATION).await;
                            ace_state.is_running = false;
                        }

                        if instruction.allow_sneak {
                            if ace_state.is_sneaking && distance > 3. {
                                println!("Releasing Shift");
                                enigo.key_up(Key::Shift);
                                ace_state.is_sneaking = false;
                            }

                            if !ace_state.is_sneaking && distance < 3. {
                                println!("Pressing Shift");
                                enigo.key_down(Key::Shift);
                                ace_state.is_sneaking = true;
                            }
                        }
                    }

                    Message::Hand { instruction } => {
                        if instruction.repeat_right_click {
                            enigo.mouse_click(MouseButton::Right);
                        }

                        if !ace_state.hand_slot_changed {
                            let ch = char::from_digit(instruction.change_hand_slot_to as u32, 10)
                                .unwrap_or_default();
                            enigo.key_click(Key::Layout(ch));
                        }
                    }

                    Message::InstructionFinished => {
                        println!("Releasing All (Instruction Finished)");

                        if ace_state.is_walking {
                            println!("Releasing W (Instruction Finished)");
                            enigo.key_up(Key::W);
                            ace_state.is_walking = false;
                        }

                        if ace_state.is_running {
                            println!("Releasing Control (Instruction Finished)");
                            enigo.key_up(Key::Control);
                            println!("Repressing W");
                            enigo.key_up(Key::W);
                            sleep(TICKRATE_DURATION).await;
                            enigo.key_down(Key::W);
                            sleep(TICKRATE_DURATION).await;
                            ace_state.is_running = false;
                        }

                        if ace_state.is_sneaking {
                            println!("Releasing Shift (Instruction Finished)");
                            enigo.key_up(Key::Shift);
                            ace_state.is_sneaking = false;
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            let mut instructions = instructions.lock().await;
            while let Some(instruction) = instructions.last().cloned() {
                sleep(TICKRATE_DURATION).await;

                let minecraft_resource = minecraft_resource2.lock().await;

                let my_position = Vector3D::from(minecraft_resource.player_position);
                let goal_position = Vector3D::from(instruction.walk.to);

                let horizontal_force = my_position
                    .horizontal_angle_distance_to(
                        &goal_position,
                        minecraft_resource.player_head.yaw,
                    )
                    .calculate_angle_force(32, -32);

                let vertical_angle = if instruction.look_downwards {
                    85.0
                } else {
                    0.0
                };

                let vertical_force = (minecraft_resource.player_head.y - vertical_angle)
                    .calculate_angle_force(32, -32);

                tx.send(Message::HeadMovement {
                    horizontal_force,
                    vertical_force,
                })
                .unwrap();

                let distance = my_position.distance_to(&goal_position);

                tx.send(Message::Walk {
                    distance,
                    instruction: instruction.clone(),
                    head_movement_force: (horizontal_force, vertical_force),
                })
                .unwrap();

                tx.send(Message::Hand {
                    instruction: instruction.clone(),
                })
                .unwrap();

                if distance < 0.1 {
                    instructions.pop();
                    tx.send(Message::InstructionFinished).unwrap();
                }
            }
        });

        loop {
            select! {
              Some(websocket_message) = ws.next() => {
                match websocket_message {
                  Ok(ws::Frame::Ping(_)) => ws.send(ws::Message::Pong(Bytes::new())).await.unwrap(),
                  Ok(ws::Frame::Text(text_received)) => {
                    let output: Value = serde_json::from_slice(&text_received).unwrap();

                    let mut minecraft_resource = minecraft_resource1.lock().await;

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

              _ = sleep(TIMEOUT_SECONDS_DURATION) => {
                break;
              }
            }
        }
    };

    if let Err(_) = timeout(TIMEOUT_SECONDS_DURATION, main_task).await {
        println!("Terminated after timeout.");
    }
}
