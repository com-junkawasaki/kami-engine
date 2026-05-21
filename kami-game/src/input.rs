//! Input system: keyboard → movement velocity → physics body.

use glam::Vec3;

/// Input state (polled from winit keyboard events).
#[derive(Debug, Default, Clone)]
pub struct InputState {
    pub forward: bool,  // W
    pub backward: bool, // S
    pub left: bool,     // A
    pub right: bool,    // D
    pub jump: bool,     // Space
    pub interact: bool, // E
    pub chat: bool,     // Enter
}

/// Player controller: converts input to velocity.
pub struct PlayerController {
    pub move_speed: f32,
    pub jump_impulse: f32,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            jump_impulse: 8.0,
        }
    }
}

impl PlayerController {
    /// Compute movement velocity from input state + camera yaw.
    pub fn movement_velocity(&self, input: &InputState, camera_yaw: f32) -> Vec3 {
        let mut dir = Vec3::ZERO;
        if input.forward {
            dir.z -= 1.0;
        }
        if input.backward {
            dir.z += 1.0;
        }
        if input.left {
            dir.x -= 1.0;
        }
        if input.right {
            dir.x += 1.0;
        }

        if dir.length_squared() < 0.001 {
            return Vec3::ZERO;
        }
        dir = dir.normalize();

        // Rotate by camera yaw
        let cos = camera_yaw.cos();
        let sin = camera_yaw.sin();
        let rotated = Vec3::new(dir.x * cos - dir.z * sin, 0.0, dir.x * sin + dir.z * cos);

        rotated * self.move_speed
    }
}

/// Serialize position for KNP Channel 0 send.
pub fn position_to_bytes(pos: Vec3) -> Vec<u8> {
    let mut buf = Vec::with_capacity(12);
    buf.extend_from_slice(&pos.x.to_le_bytes());
    buf.extend_from_slice(&pos.y.to_le_bytes());
    buf.extend_from_slice(&pos.z.to_le_bytes());
    buf
}

/// Deserialize position from KNP Channel 0.
pub fn position_from_bytes(data: &[u8]) -> Option<Vec3> {
    if data.len() < 12 {
        return None;
    }
    Some(Vec3::new(
        f32::from_le_bytes(data[0..4].try_into().ok()?),
        f32::from_le_bytes(data[4..8].try_into().ok()?),
        f32::from_le_bytes(data[8..12].try_into().ok()?),
    ))
}
