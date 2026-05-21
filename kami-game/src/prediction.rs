//! Client-side prediction + server reconciliation.
//!
//! Client applies inputs immediately (predicted state).
//! Server sends authoritative state at server tick.
//! Client replays unconfirmed inputs from last confirmed tick.
//! Remote entities are interpolated between known states.

use glam::Vec3;

const BUFFER_SIZE: usize = 128;
const SNAP_THRESHOLD: f32 = 5.0; // teleport if delta > threshold

/// One input + resulting state snapshot.
#[derive(Debug, Clone, Copy)]
pub struct InputSnapshot {
    pub tick: u32,
    pub input_velocity: Vec3,
    pub predicted_position: Vec3,
}

/// Client-side prediction buffer for the local player.
pub struct PredictionBuffer {
    buffer: [Option<InputSnapshot>; BUFFER_SIZE],
    last_confirmed_tick: u32,
    last_confirmed_position: Vec3,
}

impl PredictionBuffer {
    pub fn new() -> Self {
        Self {
            buffer: [None; BUFFER_SIZE],
            last_confirmed_tick: 0,
            last_confirmed_position: Vec3::ZERO,
        }
    }

    /// Record a local input + predicted position.
    pub fn push(&mut self, tick: u32, input_velocity: Vec3, predicted_position: Vec3) {
        let idx = tick as usize % BUFFER_SIZE;
        self.buffer[idx] = Some(InputSnapshot {
            tick,
            input_velocity,
            predicted_position,
        });
    }

    /// Server confirmed state at a tick. Reconcile.
    /// Returns corrected position (replayed from confirmed state).
    pub fn reconcile(
        &mut self,
        server_tick: u32,
        server_position: Vec3,
        current_tick: u32,
        dt: f32,
    ) -> Vec3 {
        // Snap if too far off
        let predicted = self.predicted_at(server_tick);
        if let Some(pred) = predicted {
            if pred.distance(server_position) > SNAP_THRESHOLD {
                self.last_confirmed_tick = server_tick;
                self.last_confirmed_position = server_position;
                return server_position;
            }
        }

        self.last_confirmed_tick = server_tick;
        self.last_confirmed_position = server_position;

        // Replay unconfirmed inputs
        let mut pos = server_position;
        for tick in (server_tick + 1)..=current_tick {
            let idx = tick as usize % BUFFER_SIZE;
            if let Some(snap) = &self.buffer[idx] {
                if snap.tick == tick {
                    pos += snap.input_velocity * dt;
                }
            }
        }
        pos
    }

    fn predicted_at(&self, tick: u32) -> Option<Vec3> {
        let idx = tick as usize % BUFFER_SIZE;
        self.buffer[idx]
            .as_ref()
            .filter(|s| s.tick == tick)
            .map(|s| s.predicted_position)
    }

    pub fn last_confirmed_position(&self) -> Vec3 {
        self.last_confirmed_position
    }
}

impl Default for PredictionBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Interpolation state for remote entities (other players).
pub struct RemoteInterpolation {
    prev_position: Vec3,
    target_position: Vec3,
    prev_tick: u32,
    target_tick: u32,
}

impl RemoteInterpolation {
    pub fn new() -> Self {
        Self {
            prev_position: Vec3::ZERO,
            target_position: Vec3::ZERO,
            prev_tick: 0,
            target_tick: 0,
        }
    }

    /// Update with new server state.
    pub fn push_state(&mut self, tick: u32, position: Vec3) {
        self.prev_position = self.target_position;
        self.prev_tick = self.target_tick;
        self.target_position = position;
        self.target_tick = tick;
    }

    /// Interpolate between prev and target. `alpha` = 0.0..1.0 within tick.
    pub fn interpolate(&self, alpha: f32) -> Vec3 {
        self.prev_position
            .lerp(self.target_position, alpha.clamp(0.0, 1.0))
    }
}

impl Default for RemoteInterpolation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prediction_reconcile_no_correction() {
        let mut buf = PredictionBuffer::new();
        let dt = 1.0 / 60.0;

        // Client predicts tick 1-3
        buf.push(1, Vec3::X, Vec3::new(dt, 0.0, 0.0));
        buf.push(2, Vec3::X, Vec3::new(dt * 2.0, 0.0, 0.0));
        buf.push(3, Vec3::X, Vec3::new(dt * 3.0, 0.0, 0.0));

        // Server confirms tick 1 matches prediction
        let corrected = buf.reconcile(1, Vec3::new(dt, 0.0, 0.0), 3, dt);
        // Should replay ticks 2,3 from confirmed pos
        assert!((corrected.x - dt * 3.0).abs() < 0.001);
    }

    #[test]
    fn prediction_snap_on_large_error() {
        let mut buf = PredictionBuffer::new();
        buf.push(1, Vec3::X, Vec3::new(1.0, 0.0, 0.0));

        // Server says we're at (100, 0, 0) — huge discrepancy
        let corrected = buf.reconcile(1, Vec3::new(100.0, 0.0, 0.0), 1, 1.0 / 60.0);
        assert_eq!(corrected, Vec3::new(100.0, 0.0, 0.0)); // snap
    }

    #[test]
    fn remote_interpolation() {
        let mut interp = RemoteInterpolation::new();
        interp.push_state(0, Vec3::ZERO);
        interp.push_state(1, Vec3::new(10.0, 0.0, 0.0));

        let mid = interp.interpolate(0.5);
        assert!((mid.x - 5.0).abs() < 0.001);
    }
}
