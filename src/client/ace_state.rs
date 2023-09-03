pub struct AceState {
    pub allow_run: bool,
    pub allow_sneak: bool,

    pub is_walking: bool,
    pub is_running: bool,
    pub is_sneaking: bool,

    pub hand_slot_changed: bool,
}

impl AceState {
    pub fn new() -> Self {
        Self {
            allow_run: false,
            allow_sneak: false,

            is_walking: false,
            is_running: false,
            is_sneaking: false,

            hand_slot_changed: false,
        }
    }
}
