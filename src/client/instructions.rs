use toml::Value;

#[derive(Debug, Clone)]
pub struct Walk {
    pub to: [f64; 3],
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub name: String,
    pub walk: Walk,
    pub allow_run: bool,
    pub allow_sneak: bool,
    pub rotate_before_walk: bool,
    pub look_downwards: bool,
    pub repeat_right_click: bool,
    pub change_hand_slot_to: i64,
}

pub fn list_instructions() -> Vec<Instruction> {
    let value: Value = toml::from_str(&include_str!("../../instructions.toml")).unwrap();

    value
        .get("instruction")
        .expect("Expect at least one instruction")
        .as_array()
        .expect("Instruction is not an array")
        .iter()
        .map(|raw_instruction| {
            let table = raw_instruction
                .as_table()
                .expect("Could not parse instruction as table");

            let name = table
                .get("name")
                .and_then(|name| name.as_str())
                .unwrap_or("Unknown")
                .to_string();

            let allow_run = table
                .get("allow_run")
                .and_then(|item| item.as_bool())
                .unwrap_or(false);

            let allow_sneak = table
                .get("allow_sneak")
                .and_then(|item| item.as_bool())
                .unwrap_or(false);

            let rotate_before_walk = table
                .get("rotate_before_walk")
                .and_then(|item| item.as_bool())
                .unwrap_or(false);

            let look_downwards = table
                .get("look_downwards")
                .and_then(|item| item.as_bool())
                .unwrap_or(false);

            let repeat_right_click = table
                .get("repeat_right_click")
                .and_then(|item| item.as_bool())
                .unwrap_or(false);

            let change_hand_slot_to = table
                .get("change_hand_slot_to")
                .and_then(|item| item.as_integer())
                .unwrap_or(1);

            let walk_to: [f64; 3] = table
                .get("walk_to")
                .and_then(|walk_to| {
                    let coords = walk_to
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|coord| coord.as_float().unwrap())
                        .collect::<Vec<f64>>();

                    match coords.len() == 3 {
                        true => Some([coords[0], coords[1], coords[2]]),
                        false => None,
                    }
                })
                .unwrap();

            Instruction {
                name,
                allow_run,
                allow_sneak,
                rotate_before_walk,
                look_downwards,
                repeat_right_click,
                change_hand_slot_to,
                walk: Walk { to: walk_to },
            }
        })
        .collect()
}
