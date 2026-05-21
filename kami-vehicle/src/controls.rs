//! Driver inputs.
//!
//! All inputs are normalised to `[0, 1]` (or `[-1, 1]` for steering). Higher-
//! level systems (autopilot, replay) feed these structs each tick; the vehicle
//! integrator translates them into engine throttle, clutch engagement, brake
//! torque, and tie-rod hydro extension.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Controls {
    /// `[0, 1]` — accelerator pedal.
    pub throttle: f32,
    /// `[0, 1]` — brake pedal (foot brake).
    pub brake: f32,
    /// `[0, 1]` — handbrake (locks rear wheels only).
    pub handbrake: f32,
    /// `[0, 1]` — clutch pedal (1 = pressed = clutch open).
    pub clutch_pedal: f32,
    /// `[-1, 1]` — steering wheel position.
    pub steer: f32,
    /// Driver-requested gear (manual-shift only).
    pub requested_gear: i32,
    /// Ignition state.
    pub ignition: bool,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            brake: 0.0,
            handbrake: 0.0,
            clutch_pedal: 0.0,
            steer: 0.0,
            requested_gear: 1,
            ignition: true,
        }
    }
}

impl Controls {
    pub fn coast() -> Self {
        Self::default()
    }

    /// Saturate every channel into its declared range. Useful when piping in
    /// raw HID data.
    pub fn clamp_inputs(&mut self) {
        self.throttle = self.throttle.clamp(0.0, 1.0);
        self.brake = self.brake.clamp(0.0, 1.0);
        self.handbrake = self.handbrake.clamp(0.0, 1.0);
        self.clutch_pedal = self.clutch_pedal.clamp(0.0, 1.0);
        self.steer = self.steer.clamp(-1.0, 1.0);
    }
}
