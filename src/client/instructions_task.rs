use std::sync::Arc;

use crate::{
    handlers::message::Message,
    instructions::{list_instructions, Looking},
    minecraft_resource::MinecraftResource,
    vectors::{CalculateAngleForce, Vector3D},
    DISTANCE_TO_FINISH_INSTRUCTION, TICKRATE_DURATION,
};

use futures_util::lock::Mutex;
use tokio::{sync::broadcast::Sender, time::sleep};

pub fn spawn(minecraft_resource: Arc<Mutex<MinecraftResource>>, tx: Sender<Message>) -> () {
    tokio::spawn(async move {
        let mut instructions = list_instructions();
        instructions.reverse();

        while let Some(instruction) = instructions.last().cloned() {
            sleep(TICKRATE_DURATION).await;

            let minecraft_resource = minecraft_resource.lock().await;

            let my_position = Vector3D::from(minecraft_resource.player_position);
            let goal_position = Vector3D::from(instruction.destination);

            let mut horizontal_force = my_position
                .horizontal_angle_distance_to(&goal_position, minecraft_resource.player_head.yaw);

            match instruction.looking {
                Looking::Back => match horizontal_force.is_sign_negative() {
                    true => horizontal_force += 180.0,
                    false => horizontal_force += -180.0,
                },

                Looking::Direction(direction) => {
                    horizontal_force = minecraft_resource.player_head.yaw - direction
                }

                _ => (),
            };

            let vertical_angle = 5.0;

            let vertical_force =
                (minecraft_resource.player_head.y - vertical_angle).calculate_angle_force(32, -32);

            let head_movement = Message::HeadMovement {
                horizontal_force: horizontal_force.calculate_angle_force(32, -32),
                vertical_force,
            };

            tx.send(head_movement).unwrap();
            tx.send(Message::Hand(instruction.clone())).unwrap();

            let distance = my_position.distance_to(&goal_position);

            let fly_horizontal = Message::FlyHorizontal {
                distance,
                instruction: instruction.clone(),
            };

            let fly_vertical = Message::FlyVertical {
                distance: my_position.y_distance_to(&goal_position),
                instruction: instruction.clone(),
            };

            tx.send(fly_horizontal).unwrap();
            tx.send(fly_vertical).unwrap();

            if distance < DISTANCE_TO_FINISH_INSTRUCTION {
                instructions.pop();
                let tx = tx.clone();

                tokio::task::spawn(async move {
                    sleep(TICKRATE_DURATION * 10).await;

                    let message = Message::InstructionFinished(instruction.clone());
                    tx.send(message).unwrap();
                });
            }
        }
    });
}
