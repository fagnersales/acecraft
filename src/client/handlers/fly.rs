use std::time::Instant;
use tokio::time::sleep;

use crate::{
    ace_state::AceState,
    instructions::{Instruction, Looking},
    TICKRATE_DURATION,
};
use enigo::*;

pub async fn handle_fly_horizontal(
    ace_state: &mut AceState,
    instruction: Instruction,
    distance: f64,
) {
    if !ace_state.is_turning && !ace_state.is_moving {
        let (delay_to_release, delay_to_fresh, key) = match instruction.looking {
            Looking::Direction(_) => (TICKRATE_DURATION * 3, TICKRATE_DURATION * 12, Key::A),

            Looking::Back => {
                if distance < 3.0 {
                    (TICKRATE_DURATION * 3, TICKRATE_DURATION * 10, Key::S)
                } else {
                    (TICKRATE_DURATION * 5, TICKRATE_DURATION * 10, Key::S)
                }
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
            tokio::task::spawn(async move {
                let mut enigo = Enigo::new();

                enigo.key_down(key);
                if distance > 6.0 {
                    enigo.key_down(Key::Control);
                }
                sleep(delay_to_release).await;
                if distance > 6.0 {
                    enigo.key_up(Key::Control)
                }
                enigo.key_up(key);
            });

            ace_state.last_flight = Instant::now();
        }
    }
}

pub async fn handle_fly_vertical(
    enigo: &mut Enigo,
    ace_state: &mut AceState,
    _instruction: Instruction,
    distance: f64,
) {
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
