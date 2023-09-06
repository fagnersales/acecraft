use crate::minecraft_resource::PlayerPosition;

use std::ops::Sub;

#[derive(Clone)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<PlayerPosition> for Vector3D {
    fn from(value: PlayerPosition) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<[f64; 3]> for Vector3D {
    fn from(value: [f64; 3]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl Sub for Vector3D {
    type Output = Vector3D;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub for &Vector3D {
    type Output = Vector3D;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Vector3D {
    pub fn magnitude(&self) -> f64 {
        f64::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }

    pub fn normalized(&self) -> Vector3D {
        let magnitude = self.magnitude();

        Vector3D {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
        }
    }

    pub fn angle(&self) -> f64 {
        let angle_radians = f64::atan2(self.x, self.z);
        let angle_degrees = angle_radians.to_degrees();

        let adjusted_angle_degrees = if angle_degrees > 180.0 {
            angle_degrees - 360.0
        } else if angle_degrees < -180.0 {
            angle_degrees + 360.0
        } else {
            angle_degrees
        };

        adjusted_angle_degrees * -1.
    }

    pub fn distance_to(&self, other: &Vector3D) -> f64 {
        let me = Vector3D {
            x: self.x,
            y: 0.,
            z: self.z,
        };

        let other = Vector3D {
            x: other.x,
            y: 0.,
            z: other.z,
        };

        (me - other).magnitude()
    }

    pub fn y_distance_to(&self, other: &Vector3D) -> f64 {
        self.y - other.y
    }

    pub fn horizontal_angle_distance_to(&self, goal: &Vector3D, yaw: f64) -> f64 {
        yaw - ((goal - self).normalized().angle())
    }
}

pub trait CalculateAngleForce {
    fn calculate_angle_force(&self, min: i32, max: i32) -> i32;
}

impl CalculateAngleForce for f64 {
    fn calculate_angle_force(&self, min: i32, max: i32) -> i32 {
        let mut result = self.clone();

        while result > 180.0 {
            result -= 360.0;
        }

        while result < -180.0 {
            result += 360.0;
        }

        (result as i32).max(max).min(min)
    }
}
