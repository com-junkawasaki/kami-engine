//! Spatial audio mixer for WebRTC participants.
//!
//! Maps peer positions to kami-audio AudioSource instances,
//! producing per-peer stereo pan + volume for HRTF spatialization.

use glam::Vec3;
use kami_audio::{AudioMixer, AudioSource, Channel, Listener, Rolloff};

use crate::peer::Peer;

/// Spatial audio mixer that bridges WebRTC peers to kami-audio.
///
/// Each peer with an active audio track becomes an AudioSource
/// positioned in 3D space. The listener (local user) hears
/// directional audio based on peer positions.
pub struct SpatialMixer {
    mixer: AudioMixer,
}

/// Per-peer spatialization result.
#[derive(Debug, Clone, Copy)]
pub struct SpatialResult {
    /// Left channel volume (0..1).
    pub left: f32,
    /// Right channel volume (0..1).
    pub right: f32,
    /// Stereo pan (-1 = left, +1 = right).
    pub pan: f32,
}

impl SpatialMixer {
    /// Create a new spatial mixer.
    pub fn new() -> Self {
        let mut mixer = AudioMixer::new();
        mixer.max_voices = 64; // support large briefings
        Self { mixer }
    }

    /// Update the listener (local user) position and orientation.
    pub fn set_listener(&mut self, position: Vec3, forward: Vec3, up: Vec3) {
        self.mixer.listener = Listener {
            position,
            forward,
            up,
        };
    }

    /// Spatialize all peers with active audio tracks.
    /// Returns (peer_id, SpatialResult) pairs for the JS layer
    /// to apply as Web Audio API gain/pan values.
    pub fn spatialize_peers<'a>(&mut self, peers: &'a [Peer]) -> Vec<(&'a str, SpatialResult)> {
        self.mixer.sources.clear();

        for (i, peer) in peers.iter().enumerate() {
            if !peer.has_audio() || !peer.spatial_audio {
                continue;
            }
            self.mixer.sources.push(AudioSource {
                id: i as u64,
                position: peer.position_vec3(),
                volume: 1.0,
                pitch: 1.0,
                looping: true, // continuous voice stream
                max_distance: 50.0,
                rolloff: Rolloff::Inverse,
                priority: 10,
                channel: Channel::Voice,
            });
        }

        let mut results = Vec::new();
        for (i, peer) in peers.iter().enumerate() {
            if !peer.has_audio() || !peer.spatial_audio {
                continue;
            }
            if let Some(source) = self.mixer.sources.iter().find(|s| s.id == i as u64) {
                let (left, right, pan) = self.mixer.spatialize(source);
                results.push((peer.id.as_str(), SpatialResult { left, right, pan }));
            }
        }

        results
    }

    /// Set master volume (0..1).
    pub fn set_master_volume(&mut self, volume: f32) {
        self.mixer.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set voice channel volume (0..1).
    pub fn set_voice_volume(&mut self, volume: f32) {
        self.mixer.channel_volumes[Channel::Voice as usize] = volume.clamp(0.0, 1.0);
    }
}

impl Default for SpatialMixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::MediaKind;
    use crate::peer::Peer;

    #[test]
    fn spatialize_two_peers() {
        let mut mixer = SpatialMixer::new();
        mixer.set_listener(Vec3::ZERO, -Vec3::Z, Vec3::Y);

        let mut alice = Peer::new("alice".into(), "Alice".into());
        alice.add_track("a-audio".into(), MediaKind::Audio);
        alice.set_position([5.0, 0.0, 0.0]); // right side

        let mut bob = Peer::new("bob".into(), "Bob".into());
        bob.add_track("b-audio".into(), MediaKind::Audio);
        bob.set_position([-5.0, 0.0, 0.0]); // left side

        let peers = [alice, bob];
        let results = mixer.spatialize_peers(&peers);
        assert_eq!(results.len(), 2);

        // Alice is to the right: right volume > left volume
        let alice_result = results.iter().find(|(id, _)| *id == "alice").unwrap().1;
        assert!(
            alice_result.right > alice_result.left,
            "alice right={} left={}",
            alice_result.right,
            alice_result.left
        );
        assert!(alice_result.pan > 0.0);

        // Bob is to the left: left volume > right volume
        let bob_result = results.iter().find(|(id, _)| *id == "bob").unwrap().1;
        assert!(
            bob_result.left > bob_result.right,
            "bob left={} right={}",
            bob_result.left,
            bob_result.right
        );
        assert!(bob_result.pan < 0.0);
    }

    #[test]
    fn muted_peer_excluded() {
        let mut mixer = SpatialMixer::new();

        let mut alice = Peer::new("alice".into(), "Alice".into());
        alice.add_track("a-audio".into(), MediaKind::Audio);
        alice.set_track_state("a-audio", crate::media::TrackState::Muted);

        let peers = [alice];
        let results = mixer.spatialize_peers(&peers);
        assert_eq!(results.len(), 0); // muted = excluded
    }
}
