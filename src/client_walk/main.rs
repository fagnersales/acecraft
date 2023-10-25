mod ace_state;
mod actions;
mod instructions;
mod minecraft_resource;
mod vectors;

use std::{sync::Arc, time::Duration};

use ace_state::AceState;
use actions::try_action;
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

const TIMEOUT_SECONDS_DURATION: Duration = Duration::from_secs(20000);
const TICKRATE_DURATION: Duration = Duration::from_millis(60);
const WEBSOCKET_URL: &str = "ws://127.0.0.1:8080/ws";

const SNEAK_END_AT: f64 = 3.0;
const WALK_START_AT: f64 = 0.2;
const RUN_START_AT: f64 = 5.0;

#[derive(Debug, Clone)]
enum Message {
    InstructionFinished,
    HeadMovement {
        horizontal_force: i32,
        vertical_force: i32,
    },
    Walk {
        distance: f64,
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
                        ace_state.is_turning = horizontal_force != 0 || vertical_force != 0;
                    }

                    Message::Walk {
                        distance,
                        instruction,
                    } => {
                        if instruction.rotate_before_walk && ace_state.is_turning {
                            try_action::stop::walk(&mut ace_state, &mut enigo);
                            try_action::stop::sneak(&mut ace_state, &mut enigo);
                            try_action::stop::run(&mut ace_state, &mut enigo, TICKRATE_DURATION)
                                .await;
                            continue;
                        }

                        if ace_state.is_walking {
                            if distance < WALK_START_AT {
                                try_action::stop::walk(&mut ace_state, &mut enigo);
                            }
                        } else {
                            try_action::walk(&mut ace_state, &mut enigo);
                        }

                        if instruction.allow_run {
                            if distance < RUN_START_AT {
                                try_action::stop::run(
                                    &mut ace_state,
                                    &mut enigo,
                                    TICKRATE_DURATION,
                                )
                                .await;
                            } else {
                                try_action::run(&mut ace_state, &mut enigo);
                            }
                        }

                        if instruction.allow_sneak {
                            if distance > SNEAK_END_AT {
                                try_action::stop::sneak(&mut ace_state, &mut enigo);
                            } else {
                                try_action::sneak(&mut ace_state, &mut enigo);
                            }
                        }
                    }

                    Message::Hand { instruction } => {
                        if !ace_state.hand_slot_changed {
                            println!("Changing Hand Slot");
                            enigo.key_click(Key::Layout(instruction.change_hand_slot_to));
                            ace_state.hand_slot_changed = true;
                        }

                        if instruction.reset_hand_stack {
                            try_action::reset_hand_stack(
                                &mut ace_state,
                                &mut enigo,
                                TICKRATE_DURATION,
                            )
                            .await;
                        }

                        if instruction.repeat_right_click {
                            if !ace_state.is_pressing_right_click {
                                println!("Pressing Mouse Right Button");
                                enigo.mouse_down(MouseButton::Right);
                                ace_state.is_pressing_right_click = true;
                            }
                        }
                    }

                    Message::InstructionFinished => {
                        println!("Releasing All (Instruction Finished)");
                        ace_state.hand_slot_changed = false;
                        ace_state.hand_stack_reseted = false;

                        if ace_state.is_walking {
                            try_action::stop::walk(&mut ace_state, &mut enigo);
                        }

                        if ace_state.is_running {
                            try_action::stop::run(&mut ace_state, &mut enigo, TICKRATE_DURATION)
                                .await;
                        }

                        if ace_state.is_sneaking {
                            try_action::stop::sneak(&mut ace_state, &mut enigo);
                        }

                        if ace_state.is_pressing_right_click {
                            enigo.mouse_up(MouseButton::Right);
                            ace_state.is_pressing_right_click = false;
                        };
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
