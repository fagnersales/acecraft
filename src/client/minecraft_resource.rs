use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct PlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<(f64, f64, f64)> for PlayerPosition {
    fn from(value: (f64, f64, f64)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct PlayerHead {
    pub yaw: f64,
    pub y: f64,
}

/// Minecraft Resource are data that comes from the minecraft mod via websocket connection
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct MinecraftResource {
    pub player_position: PlayerPosition,
    pub player_head: PlayerHead,
}

impl MinecraftResource {
    pub fn new() -> Self {
        Self {
            player_position: PlayerPosition {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            player_head: PlayerHead { yaw: 0., y: 0. },
        }
    }
}
