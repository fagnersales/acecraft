mod utils;

use toml::{map::Map, Value};

use self::utils::{get_action, get_destination, get_hand_slot, get_looking};

#[derive(Debug, Clone, Copy)]
pub enum Looking {
    Front,
    Back,
    Direction(f64),
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Fly,
    Walk,
    RightClick,
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub destination: [f64; 3],
    pub hand_slot: char,
    pub reset_hand_stack: bool,
    pub looking: Looking,
    pub action: Action,
}

pub fn list_instructions() -> Vec<Instruction> {
    let toml: Value = toml::from_str(&include_str!("../../../instructions.toml")).unwrap();

    toml.get("instruction")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|value| {
            let table = value.as_table().unwrap();
            let hand_slot = get_hand_slot(table);
            path_to_instructions(table, hand_slot)
        })
        .collect()
}

fn path_to_instructions(value: &Map<String, Value>, hand_slot: char) -> Vec<Instruction> {
    value
        .get("path")
        .unwrap()
        .as_array()
        .iter()
        .flat_map(|value| {
            value.iter().map(|value| {
                let path_item = value.as_table().unwrap();
                path_item_to_instruction(path_item, hand_slot)
            })
        })
        .collect()
}

fn path_item_to_instruction(value: &Map<String, Value>, hand_slot: char) -> Instruction {
    let action = get_action(value);
    let destination = get_destination(value);
    let looking = get_looking(value);

    let reset_hand_stack = match action {
        Action::RightClick => true,
        _ => false,
    };

    Instruction {
        action,
        destination,
        looking,
        reset_hand_stack,
        hand_slot,
    }
}
