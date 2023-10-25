use toml::{map::Map, Value};

use super::{Action, Looking};

pub fn get_action(table: &Map<String, Value>) -> Action {
    let action = table.get("action").unwrap().as_str().unwrap();

    match action {
        "walk" => Action::Walk,
        "fly" => Action::Fly,
        _ => Action::RightClick,
    }
}

pub fn get_destination(table: &Map<String, Value>) -> [f64; 3] {
    table
        .get("destination")
        .and_then(|coords| {
            let coords = coords
                .as_array()
                .unwrap()
                .iter()
                .map(|coord| coord.as_float().unwrap())
                .collect::<Vec<f64>>();

            assert_eq!(coords.len(), 3);

            Some([coords[0], coords[1], coords[2]])
        })
        .unwrap()
}

pub fn get_looking(table: &Map<String, Value>) -> Looking {
    let looking = table.get("looking");

    if looking.is_none() {
        return Looking::Front;
    }

    let looking = looking.unwrap();

    if let Some(direction) = looking.as_float() {
        return Looking::Direction(direction);
    }

    if let Some(direction) = looking.as_str() {
        return match direction {
            "back" => Looking::Back,
            _ => Looking::Front,
        };
    }

    panic!("direction could not be parsed");
}

pub fn get_hand_slot(table: &Map<String, Value>) -> char {
    table
        .get("hand_slot")
        .and_then(|item| item.as_str().unwrap().chars().next())
        .unwrap_or('1')
}