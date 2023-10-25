mod ace_state;
mod actions;
mod instructions;
mod minecraft_resource;
mod vectors;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use ace_state::AceState;
use actix_web::web::Bytes;
use awc::ws;
use enigo::*;
use futures_util::{lock::Mutex, SinkExt, StreamExt as _};
use instructions::{list_instructions, Instruction, Looking};
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

const TIMEOUT_DURATION: Duration = Duration::from_secs(1200);
const TICKRATE_DURATION: Duration = Duration::from_millis(60);
const WEBSOCKET_URL: &str = "ws://127.0.0.1:8080/ws";
const DISTANCE_TO_FINISH_INSTRUCTION: f64 = 0.3;

#[derive(Debug, Clone)]
enum Message {
    InstructionFinished {
        instruction: Instruction,
    },
    HeadMovement {
        horizontal_force: i32,
        vertical_force: i32,
    },
    Hand {
        instruction: Instruction,
    },
    FlyX {
        instruction: Instruction,
        distance: f64,
    },
    FlyY {
        instruction: Instruction,
        distance: f64,
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

    let ace_state = AceState::new_shared();
    let enigo = Arc::new(Mutex::new(Enigo::new()));
    let minecraft_resource = Arc::new(Mutex::new(MinecraftResource::new()));

    let main_task = async move {
        let mut receiver = tx.subscribe();
        let enigo = enigo.clone();
        let ace_state = ace_state.clone();

        tokio::spawn(async move {
            loop {
                let message = receiver.recv().await.unwrap();
                let mut enigo = enigo.lock().await;
                let mut ace_state = ace_state.lock().await;

                match message {
                    Message::HeadMovement {
                        horizontal_force,
                        vertical_force,
                    } => {
                        enigo.mouse_move_relative(horizontal_force * -1, vertical_force * -1);
                        ace_state.is_turning = horizontal_force != 0 || vertical_force != 0;
                    }

                    Message::Hand { instruction } => {
                        if instruction.reset_hand_stack && !ace_state.hand_stack_reseted {
                            println!("Reseting Hand Stack");
                            enigo.key_click(Key::E);
                            tokio::time::sleep(TICKRATE_DURATION * 2).await;

                            for _ in 0..4 {
                                enigo.mouse_move_relative(0, 32);
                                tokio::time::sleep(TICKRATE_DURATION).await;
                            }

                            tokio::time::sleep(TICKRATE_DURATION * 3).await;
                            enigo.mouse_click(MouseButton::Left);
                            tokio::time::sleep(TICKRATE_DURATION * 2).await;
                            enigo.mouse_click(MouseButton::Left);

                            tokio::time::sleep(TICKRATE_DURATION * 2).await;
                            enigo.mouse_click(MouseButton::Left);
                            enigo.key_click(Key::E);
                            tokio::time::sleep(TICKRATE_DURATION).await;
                            ace_state.hand_stack_reseted = true;
                        }

                        if instruction.repeat_right_click
                            && !ace_state.is_moving
                            && !ace_state.is_turning
                        {
                            enigo.mouse_click(MouseButton::Middle);
                            enigo.mouse_click(MouseButton::Right);
                        }
                    }

                    Message::FlyY { distance, .. } => {
                        ace_state.is_moving = distance.abs() > 0.25;

                        if distance.abs() > 0.25 {
                            if distance.is_sign_negative() {
                                let elapsed = ace_state.last_space_press.elapsed();
                                let is_fresh = elapsed >= TICKRATE_DURATION * 10;

                                if is_fresh {
                                    println!("UP {distance}");

                                    tokio::task::spawn(async move {
                                        let mut enigo = Enigo::new();

                                        // if it is too close to the goal
                                        if distance < 1.5 {
                                            return enigo.key_click(Key::Space);
                                        }

                                        enigo.key_down(Key::Space);
                                        sleep(TICKRATE_DURATION * 5).await;
                                        enigo.key_up(Key::Space);
                                    });

                                    ace_state.last_space_press = Instant::now();
                                };
                            } else {
                                let elapsed = ace_state.last_shift_press.elapsed();
                                let is_fresh = elapsed >= TICKRATE_DURATION * 10;

                                if is_fresh {
                                    println!("DOWN {distance}");
                                    enigo.key_click(Key::Shift);
                                    ace_state.last_shift_press = Instant::now();
                                }
                            }
                        };
                    }

                    Message::FlyX {
                        instruction,
                        distance,
                    } => {
                        if !ace_state.is_turning && !ace_state.is_moving {
                            let (delay_to_release, delay_to_fresh, key) = match instruction.looking
                            {
                                Looking::Back => {
                                    if distance < 3.0 {
                                        (TICKRATE_DURATION * 3, TICKRATE_DURATION * 10, Key::S)
                                    } else {
                                        (TICKRATE_DURATION * 5, TICKRATE_DURATION * 10, Key::S)
                                    }
                                }
                                Looking::Direction(_) => {
                                    (TICKRATE_DURATION * 3, TICKRATE_DURATION * 12, Key::A)
                                }
                                _ => {
                                    if distance < 2.0 {
                                        (TICKRATE_DURATION, TICKRATE_DURATION * 10, Key::W)
                                    } else if distance < 3.0 {
                                        (TICKRATE_DURATION * 2, TICKRATE_DURATION * 10, Key::W)
                                    } else if distance < 6.0 {
                                        (TICKRATE_DURATION * 3, TICKRATE_DURATION * 10, Key::W)
                                    } else {
                                        (TICKRATE_DURATION * 6, TICKRATE_DURATION * 10, Key::W)
                                    }
                                }
                            };

                            let elapsed = ace_state.last_flight.elapsed();
                            let is_fresh = elapsed >= delay_to_fresh;

                            if is_fresh {
                                let run_allowed = instruction.allow_run;

                                tokio::task::spawn(async move {
                                    let mut enigo = Enigo::new();

                                    enigo.key_down(key);
                                    if run_allowed && distance > 6.0 {
                                        enigo.key_down(Key::Control);
                                    }
                                    sleep(delay_to_release).await;
                                    if run_allowed && distance > 6.0 {
                                        enigo.key_up(Key::Control)
                                    }
                                    enigo.key_up(key);
                                });

                                ace_state.last_flight = Instant::now();
                            }
                        }
                    }

                    Message::InstructionFinished { instruction } => {
                        ace_state.hand_stack_reseted = false;

                        println!("Releasing All (Instruction Finished)");
                        if ace_state.is_pressing_right_click {
                            enigo.mouse_up(MouseButton::Right);
                            ace_state.is_pressing_right_click = false;
                        };

                        if instruction.allow_run {
                            enigo.key_up(Key::Control);
                        }

                        match instruction.looking {
                            Looking::Back => enigo.key_up(Key::S),
                            Looking::Direction(_) => enigo.key_up(Key::A),
                            _ => enigo.key_up(Key::W),
                        };
                    }
                }
            }
        });

        let main_minecraft_resource = minecraft_resource.clone();

        let mut instructions = list_instructions();
        instructions.reverse();
        let instructions = Arc::new(Mutex::new(instructions));

        tokio::spawn(async move {
            let mut instructions = instructions.lock().await;
            while let Some(instruction) = instructions.last().cloned() {
                sleep(TICKRATE_DURATION).await;

                let minecraft_resource = main_minecraft_resource.lock().await;

                let my_position = Vector3D::from(minecraft_resource.player_position);
                let goal_position = Vector3D::from(instruction.destination);

                let mut horizontal_force = my_position.horizontal_angle_distance_to(
                    &goal_position,
                    minecraft_resource.player_head.yaw,
                );

                match instruction.looking {
                    Looking::Back => match horizontal_force.is_sign_negative() {
                        true => horizontal_force += 180.0,
                        false => horizontal_force += -180.0,
                    },

                    Looking::Direction(direction) => {
                        horizontal_force = minecraft_resource.player_head.yaw - direction
                    }

                    _ => (),
                };

                let vertical_angle = 5.0;

                let vertical_force = (minecraft_resource.player_head.y - vertical_angle)
                    .calculate_angle_force(32, -32);

                tx.send(Message::HeadMovement {
                    horizontal_force: horizontal_force.calculate_angle_force(32, -32),
                    vertical_force,
                })
                .unwrap();

                let distance = my_position.distance_to(&goal_position);

                tx.send(Message::Hand {
                    instruction: instruction.clone(),
                })
                .unwrap();

                tx.send(Message::FlyX {
                    distance,
                    instruction: instruction.clone(),
                })
                .unwrap();

                let y_distance = my_position.y_distance_to(&goal_position);

                tx.send(Message::FlyY {
                    distance: y_distance,
                    instruction: instruction.clone(),
                })
                .unwrap();

                if distance < DISTANCE_TO_FINISH_INSTRUCTION {
                    instructions.pop();
                    let tx = tx.clone();
                    tokio::task::spawn(async move {
                        sleep(TICKRATE_DURATION * 10).await;
                        tx.send(Message::InstructionFinished {
                            instruction: instruction.clone(),
                        })
                        .unwrap();
                    });
                }
            }
        });

        let ws_minecraft_resource = minecraft_resource.clone();

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
