use enigo::*;
use futures_util::lock::Mutex;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

use crate::{
    ace_state::AceState,
    handlers::{
        finished::handle_finished,
        fly::{handle_fly_horizontal, handle_fly_vertical},
        hand::handle_hand,
        head_movement::handle_head_movement,
        message::Message,
    },
};

pub fn spawn(ace_state: Arc<Mutex<AceState>>, tx: Sender<Message>) {
    let enigo = Arc::new(Mutex::new(Enigo::new()));
    let mut receiver = tx.subscribe();

    tokio::spawn(async move {
        loop {
            let mut enigo = enigo.lock().await;
            let mut ace_state = ace_state.lock().await;
            let message = receiver.recv().await.unwrap();

            match message {
                Message::HeadMovement {
                    horizontal_force,
                    vertical_force,
                } => {
                    handle_head_movement(
                        &mut enigo,
                        &mut ace_state,
                        horizontal_force,
                        vertical_force,
                    );
                }

                Message::Hand(instruction) => {
                    handle_hand(&mut enigo, &mut ace_state, instruction).await;
                }

                Message::FlyHorizontal {
                    instruction,
                    distance,
                } => {
                    handle_fly_horizontal(&mut ace_state, instruction, distance).await;
                }

                Message::FlyVertical {
                    instruction,
                    distance,
                } => {
                    handle_fly_vertical(&mut enigo, &mut ace_state, instruction, distance).await;
                }

                Message::InstructionFinished(instruction) => {
                    handle_finished(&mut enigo, &mut ace_state, &instruction);
                }
            }
        }
    });
}
