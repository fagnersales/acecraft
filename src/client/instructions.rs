use toml::{map::Map, Value};

#[derive(Debug, Clone)]
pub struct Walk {
    pub to: [f64; 3],
}

impl From<[f64; 3]> for Walk {
    fn from(value: [f64; 3]) -> Self {
        Self { to: value }
    }
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
    pub change_hand_slot_to: char,
}

struct InstructionDefaults {
    pub allow_run: bool,
    pub allow_sneak: bool,
    pub rotate_before_walk: bool,
    pub look_downwards: bool,
    pub repeat_right_click: bool,
    pub change_hand_slot_to: char,
}

fn get_allow_run(table: &Map<String, Value>, default: Option<bool>) -> bool {
    table
        .get("allow_run")
        .and_then(|item| item.as_bool())
        .unwrap_or(default.unwrap_or(false))
}

fn get_allow_sneak(table: &Map<String, Value>, default: Option<bool>) -> bool {
    table
        .get("allow_sneak")
        .and_then(|item| item.as_bool())
        .unwrap_or(default.unwrap_or(false))
}

fn get_look_downwards(table: &Map<String, Value>, default: Option<bool>) -> bool {
    table
        .get("look_downwards")
        .and_then(|item| item.as_bool())
        .unwrap_or(default.unwrap_or(false))
}

fn get_rotate_before_walk(table: &Map<String, Value>, default: Option<bool>) -> bool {
    table
        .get("rotate_before_walk")
        .and_then(|item| item.as_bool())
        .unwrap_or(default.unwrap_or(false))
}

fn get_repeat_right_click(table: &Map<String, Value>, default: Option<bool>) -> bool {
    table
        .get("repeat_right_click")
        .and_then(|item| item.as_bool())
        .unwrap_or(default.unwrap_or(false))
}

fn get_change_hand_slot_to(table: &Map<String, Value>, default: Option<char>) -> char {
    table
        .get("change_hand_slot_to")
        .and_then(|item| item.as_str().unwrap().chars().next())
        .unwrap_or(default.unwrap_or('1'))
}

fn get_reverse(table: &Map<String, Value>, default: Option<bool>) -> bool {
    table
        .get("reverse")
        .and_then(|item| item.as_bool())
        .unwrap_or(default.unwrap_or(false))
}

fn get_to(table: &Map<String, Value>) -> [f64; 3] {
    table
        .get("to")
        .and_then(|coords| parse_to_coords(coords).into())
        .unwrap()
}

fn get_action(table: &Map<String, Value>) -> &str {
    table
        .get("action")
        .and_then(|action| action.as_str().unwrap().into())
        .unwrap()
}

fn parse_to_coords(value: &Value) -> [f64; 3] {
    let coords = value
        .as_array()
        .unwrap()
        .iter()
        .map(|coord| coord.as_float().unwrap())
        .collect::<Vec<f64>>();

    assert_eq!(coords.len(), 3);

    coords.try_into().unwrap()
}

pub fn list_instructions() -> Vec<Instruction> {
    let value: Value = toml::from_str(&include_str!("../../instructions.toml")).unwrap();

    let defaults = value
        .get("default")
        .unwrap()
        .as_table()
        .and_then(|table| {
            InstructionDefaults {
                allow_run: get_allow_run(table, None),
                allow_sneak: get_allow_sneak(table, None),
                look_downwards: get_look_downwards(table, None),
                rotate_before_walk: get_rotate_before_walk(table, None),
                change_hand_slot_to: get_change_hand_slot_to(table, None),
                repeat_right_click: get_repeat_right_click(table, None),
            }
            .into()
        })
        .unwrap();

    let instructions: Vec<Instruction> = value
        .get("instruction")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|table| {
            let table = table.as_table().unwrap();

            let name = table
                .get("name")
                .and_then(|name| name.as_str().unwrap().into())
                .unwrap_or("Unknown");

            let reverse = get_reverse(table, None);

            let change_hand_slot =
                get_change_hand_slot_to(table, defaults.change_hand_slot_to.into());

            let repeat_right_click =
                get_repeat_right_click(table, defaults.repeat_right_click.into());

            let coords_to_instruction = |coords: [f64; 3]| Instruction {
                allow_run: defaults.allow_run,
                allow_sneak: defaults.allow_sneak,
                change_hand_slot_to: change_hand_slot,
                look_downwards: defaults.look_downwards,
                rotate_before_walk: defaults.rotate_before_walk,
                repeat_right_click,
                name: name.to_string(),
                walk: Walk::from(coords),
            };

            let mut instructions = table
                .get("path")
                .unwrap()
                .as_array()
                .iter()
                .flat_map(|path_value| {
                    path_value.iter().map(|path_value_item| {
                        let table = path_value_item.as_table().unwrap();

                        let action = get_action(table);
                        let coords = get_to(table);

                        let mut instruction = coords_to_instruction(coords);

                        instruction.repeat_right_click = (action == "right_clicking" && !reverse)
                            || (action == "walking" && reverse);

                        instruction
                    })
                })
                .collect::<Vec<Instruction>>();

            if reverse {
                instructions.reverse();
            }

            instructions
        })
        .collect();

    return instructions;
}
