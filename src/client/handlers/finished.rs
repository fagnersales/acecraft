use enigo::*;

use crate::{
    ace_state::AceState,
    instructions::{Instruction, Looking},
};

pub fn handle_finished(enigo: &mut Enigo, ace_state: &mut AceState, instruction: &Instruction) {
    ace_state.hand_stack_reseted = false;

    println!("Releasing All (Instruction Finished)");
    if ace_state.is_pressing_right_click {
        enigo.mouse_up(MouseButton::Right);
        ace_state.is_pressing_right_click = false;
    };

    enigo.key_up(Key::Control);

    match instruction.looking {
        Looking::Back => enigo.key_up(Key::S),
        Looking::Direction(_) => enigo.key_up(Key::A),
        _ => enigo.key_up(Key::W),
    };
}
