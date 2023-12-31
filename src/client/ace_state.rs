use std::{time::Instant, sync::Arc};
use futures_util::lock::Mutex;
pub struct AceState {
    pub last_flight: Instant,
    pub is_turning: bool,
    pub is_moving: bool,

    pub last_space_press: Instant,
    pub last_shift_press: Instant,

    pub is_pressing_right_click: bool,
    pub hand_stack_reseted: bool,
}

impl AceState {
    pub fn new_shared() -> Arc<Mutex<AceState>> {
        Arc::new(Mutex::new(Self {
            last_flight: Instant::now(),
            last_space_press: Instant::now(),
            last_shift_press: Instant::now(),

            is_turning: false,
            is_moving: false,

            is_pressing_right_click: false,
            hand_stack_reseted: false,
        }))
    }
}
