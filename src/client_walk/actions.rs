pub mod try_action {
    use std::time::Duration;

    use enigo::*;

    use crate::ace_state::AceState;

    pub mod stop {
        use std::time::Duration;

        use enigo::*;

        use crate::ace_state::AceState;

        pub fn walk(ace_state: &mut AceState, enigo: &mut Enigo) {
            if ace_state.is_walking {
                println!("Releasing W");
                enigo.key_up(Key::W);
                ace_state.is_walking = false;
            }
        }

        pub fn sneak(ace_state: &mut AceState, enigo: &mut Enigo) {
            if ace_state.is_sneaking {
                println!("Releasing Shift");
                enigo.key_up(Key::Shift);
                ace_state.is_sneaking = false;
            }
        }

        pub async fn run(ace_state: &mut AceState, enigo: &mut Enigo, tickrate_duration: Duration) {
            if ace_state.is_running {
                println!("Releasing Control");
                enigo.key_up(Key::Control);
                println!("Repressing W");
                enigo.key_up(Key::W);
                tokio::time::sleep(tickrate_duration).await;
                enigo.key_down(Key::W);
                tokio::time::sleep(tickrate_duration).await;
                ace_state.is_running = false;
            }
        }
    }

    pub fn walk(ace_state: &mut AceState, enigo: &mut Enigo) {
        if !ace_state.is_walking {
            println!("Pressing W");
            enigo.key_down(Key::W);
            ace_state.is_walking = true;
        }
    }

    pub fn sneak(ace_state: &mut AceState, enigo: &mut Enigo) {
        if !ace_state.is_sneaking {
            println!("Pressing Shift");
            enigo.key_down(Key::Shift);
            ace_state.is_sneaking = true;
        };
    }

    pub fn run(ace_state: &mut AceState, enigo: &mut Enigo) {
        if !ace_state.is_running {
            println!("Pressing Control");
            enigo.key_down(Key::Control);
            ace_state.is_running = true;
        };
    }

    pub async fn reset_hand_stack(
        ace_state: &mut AceState,
        enigo: &mut Enigo,
        tickrate_duration: Duration,
    ) {
        if !ace_state.hand_stack_reseted {
            println!("Reseting Hand Stack");
            enigo.key_click(Key::E);
            tokio::time::sleep(tickrate_duration * 2).await;

            for _ in 0..4 {
                enigo.mouse_move_relative(0, 32);
                tokio::time::sleep(tickrate_duration).await;
            }

            tokio::time::sleep(tickrate_duration * 3).await;
            enigo.mouse_click(MouseButton::Left);
            tokio::time::sleep(tickrate_duration * 2).await;
            enigo.mouse_click(MouseButton::Left);

            tokio::time::sleep(tickrate_duration * 2).await;
            enigo.mouse_click(MouseButton::Left);
            enigo.key_click(Key::E);
            ace_state.hand_stack_reseted = true;
        }
    }
}
