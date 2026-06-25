//! kami-audio: Spatial audio engine.
//!
//! 3D positional audio with HRTF panning, distance attenuation,
//! mixer with priority channels, and procedural synthesis.

use glam::Vec3;

pub mod binaural;
pub mod wav;

/// Audio source in 3D space.
#[derive(Debug, Clone)]
pub struct AudioSource {
    pub id: u64,
    pub position: Vec3,
    pub volume: f32, // 0..1
    pub pitch: f32,  // 1.0 = normal
    pub looping: bool,
    pub max_distance: f32, // beyond this: silent
    pub rolloff: Rolloff,
    pub priority: u8, // higher = more important (for voice limiting)
    pub channel: Channel,
}

#[derive(Debug, Clone, Copy)]
pub enum Rolloff {
    Linear,
    Inverse,
    Exponential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Master,
    Music,
    SFX,
    Voice,
    Ambient,
}

/// Listener (camera/player ears).
#[derive(Debug, Clone)]
pub struct Listener {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
}

impl Default for Listener {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            forward: -Vec3::Z,
            up: Vec3::Y,
        }
    }
}

/// Audio mixer state.
pub struct AudioMixer {
    pub listener: Listener,
    pub sources: Vec<AudioSource>,
    pub channel_volumes: [f32; 5], // per Channel
    pub master_volume: f32,
    pub max_voices: usize,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            listener: Listener::default(),
            sources: Vec::new(),
            channel_volumes: [1.0; 5],
            master_volume: 0.8,
            max_voices: 32,
        }
    }

    /// Calculate stereo pan + volume for a source.
    pub fn spatialize(&self, source: &AudioSource) -> (f32, f32, f32) {
        let diff = source.position - self.listener.position;
        let dist = diff.length();
        if dist > source.max_distance {
            return (0.0, 0.0, 0.0);
        }

        // Distance attenuation
        let attenuation = match source.rolloff {
            Rolloff::Linear => 1.0 - (dist / source.max_distance).min(1.0),
            Rolloff::Inverse => 1.0 / (1.0 + dist),
            Rolloff::Exponential => (-(dist * 0.1)).exp(),
        };

        // Stereo panning (simplified HRTF: dot product with listener right vector)
        let right = self.listener.forward.cross(self.listener.up).normalize();
        let dir = if dist > 0.001 {
            diff / dist
        } else {
            Vec3::ZERO
        };
        let pan = dir.dot(right).clamp(-1.0, 1.0); // -1 = left, +1 = right

        let vol = source.volume
            * attenuation
            * self.master_volume
            * self.channel_volumes[source.channel as usize];
        let left = vol * (1.0 - pan.max(0.0));
        let right_vol = vol * (1.0 + pan.min(0.0));

        (left, right_vol, pan)
    }

    /// Sort sources by priority, limit to max_voices.
    pub fn active_voices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.sources.len()).collect();
        indices.sort_by(|&a, &b| self.sources[b].priority.cmp(&self.sources[a].priority));
        indices.truncate(self.max_voices);
        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatialize() {
        let mixer = AudioMixer::new();
        let source = AudioSource {
            id: 1,
            position: Vec3::new(5.0, 0.0, 0.0),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            max_distance: 100.0,
            rolloff: Rolloff::Linear,
            priority: 5,
            channel: Channel::SFX,
        };
        let (l, r, pan) = mixer.spatialize(&source);
        assert!(
            r > l,
            "source to the right should be louder in right channel"
        );
        assert!(pan > 0.0);
    }
}
