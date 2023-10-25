use crate::ace_state::AceState;
use enigo::*;

pub fn handle_head_movement(
    enigo: &mut Enigo,
    ace_state: &mut AceState,
    horizontal_force: i32,
    vertical_force: i32,
) {
    enigo.mouse_move_relative(horizontal_force * -1, vertical_force * -1);
    ace_state.is_turning = horizontal_force != 0 || vertical_force != 0;
}
