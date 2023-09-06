mod get;
mod parse;

use toml::Value;

#[derive(Debug, Clone)]
pub enum Looking {
    Front,
    Direction(f64),
    Back,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub name: String,
    pub destination: [f64; 3],
    pub looking: Looking,
    pub rotate_before_walk: bool,
    pub repeat_right_click: bool,
    pub allow_run: bool,
    pub reset_hand_stack: bool,
}

struct InstructionDefaults {
    pub rotate_before_walk: bool,
    pub repeat_right_click: bool,
    pub reset_hand_stack: bool,
}

pub fn list_instructions() -> Vec<Instruction> {
    let value: Value = toml::from_str(&include_str!("../../../instructions.toml")).unwrap();

    let defaults = value
        .get("default")
        .unwrap()
        .as_table()
        .and_then(|table| {
            InstructionDefaults {
                rotate_before_walk: get::rotate_before_walk(table, None),
                repeat_right_click: get::repeat_right_click(table, None),
                reset_hand_stack: get::reset_hand_stack(table, None),
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

            let repeat_right_click =
                get::repeat_right_click(table, defaults.repeat_right_click.into());

            let reset_hand_stack = get::reset_hand_stack(table, defaults.reset_hand_stack.into());

            let coords_to_instruction = |destination: [f64; 3]| Instruction {
                rotate_before_walk: defaults.rotate_before_walk,
                repeat_right_click,
                reset_hand_stack,
                name: name.to_string(),
                destination,
                allow_run: false,
                looking: Looking::Front,
            };

            let instructions = table
                .get("path")
                .unwrap()
                .as_array()
                .iter()
                .flat_map(|path_value| {
                    path_value.iter().map(|path_value_item| {
                        let table = path_value_item.as_table().unwrap();

                        let action = get::action(table);
                        let coords = get::to(table);
                        let allow_run = get::allow_run(table);

                        let mut instruction = coords_to_instruction(coords);

                        instruction.repeat_right_click = action == "right_clicking";
                        instruction.allow_run = allow_run;
                        instruction.looking = get::looking(table).unwrap();

                        instruction
                    })
                })
                .collect::<Vec<Instruction>>();

            instructions
        })
        .collect();

    return instructions;
}
