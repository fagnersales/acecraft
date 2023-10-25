use crate::{
    ace_state::AceState,
    instructions::{Action, Instruction},
    TICKRATE_DURATION,
};

use enigo::*;

pub async fn handle_hand(enigo: &mut Enigo, ace_state: &mut AceState, instruction: Instruction) {
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

    match instruction.action {
        Action::RightClick => {
            if !ace_state.is_moving && !ace_state.is_turning {
                enigo.mouse_click(MouseButton::Middle);
                enigo.mouse_click(MouseButton::Right);
            }
        }
        _ => (),
    }
}
