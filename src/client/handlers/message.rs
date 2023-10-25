use crate::instructions::Instruction;

#[derive(Debug, Clone)]
pub enum Message {
    InstructionFinished(Instruction),

    HeadMovement {
        horizontal_force: i32,
        vertical_force: i32,
    },

    Hand(Instruction),

    FlyHorizontal {
        instruction: Instruction,
        distance: f64,
    },

    FlyVertical {
        instruction: Instruction,
        distance: f64,
    },
}
