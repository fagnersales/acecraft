use core::panic;

use toml::{map::Map, Value};

use crate::instructions::parse;

use super::Looking;

pub fn reset_hand_stack(table: &Map<String, Value>, default: Option<bool>) -> bool {
    table
        .get("reset_hand_stack")
        .and_then(|item| item.as_bool())
        .unwrap_or(default.unwrap_or(false))
}

pub fn rotate_before_walk(table: &Map<String, Value>, default: Option<bool>) -> bool {
    table
        .get("rotate_before_walk")
        .and_then(|item| item.as_bool())
        .unwrap_or(default.unwrap_or(false))
}

pub fn repeat_right_click(table: &Map<String, Value>, default: Option<bool>) -> bool {
    table
        .get("repeat_right_click")
        .and_then(|item| item.as_bool())
        .unwrap_or(default.unwrap_or(false))
}

pub fn allow_run(table: &Map<String, Value>) -> bool {
    table
        .get("allow_run")
        .and_then(|item| item.as_bool())
        .unwrap_or(false)
}

pub fn action(table: &Map<String, Value>) -> &str {
    table
        .get("action")
        .and_then(|action| action.as_str().unwrap().into())
        .unwrap()
}

pub fn looking(table: &Map<String, Value>) -> Option<Looking> {
    let looking = table.get("looking");

    if looking.is_none() {
        return Some(Looking::Front);
    }

    let looking = looking.unwrap();

    if let Some(direction) = looking.as_float() {
        return Some(Looking::Direction(direction));
    }

    if let Some(direction) = looking.as_str() {
        return Some(match direction {
            "back" => Looking::Back,
            _ => Looking::Front,
        });
    }

    return None;
}

pub fn to(table: &Map<String, Value>) -> [f64; 3] {
    table
        .get("to")
        .and_then(|coords| parse::to_coords(coords).into())
        .unwrap()
}
