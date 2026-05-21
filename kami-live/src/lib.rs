//! kami-live — Live music venue SDK.
//!
//! Domain layer for `live.etzhayyim.com`: turn a wgpu canvas into a live show
//! where music, dance, fans, stage, sound, and visual effects (VX) are
//! co-driven from a single shared clock — the beat grid.
//!
//! ```text
//!                       ┌─────────────────┐
//!                       │  ShowClock      │  master tempo (BPM, bar)
//!                       │  (BeatGrid)     │
//!                       └────────┬────────┘
//!                                │ tick(dt)
//!         ┌──────────────────────┼──────────────────────┐
//!         ▼                      ▼                      ▼
//!   ┌──────────┐          ┌────────────┐         ┌──────────┐
//!   │ Setlist  │ ─cue──▶  │ Lighting   │ ──────▶ │ VJDeck   │
//!   └────┬─────┘          │ Designer   │ ──────▶ │ (palette)│
//!        │                └─────┬──────┘         └──────────┘
//!        │                      │ beat
//!        ▼                      ▼
//!   ┌──────────┐          ┌──────────┐
//!   │ Performer│ ─pose─▶  │ Crowd    │ ─reaction (clap/jump)
//!   └──────────┘          └──────────┘
//! ```
//!
//! All modules are deterministic given a `(bpm, t0)` pair so the same
//! show replays identically across clients (federated co-presence).

pub mod audio;
pub mod beat;
pub mod cheer;
pub mod crowd;
pub mod lighting;
pub mod performer;
pub mod setlist;
pub mod show;
pub mod stage;
pub mod vj;

pub use audio::{midi_to_hz, AudioCue, AudioPattern, BassLine, BassNote, DrumPattern, DrumSlot};
pub use beat::{BeatEvent, BeatGrid, BeatPhase};
pub use cheer::{CheerAggregate, CheerKind, CheerSample};
pub use crowd::{Crowd, CrowdConfig, Fan, FanMood, FanSnapshot};
pub use lighting::{Envelope, LightingCue, LightingDesigner, LightingFixture, LightingFrame};
pub use performer::{DanceMove, DancePose, Performer};
pub use setlist::{CuePoint, CueKind, Setlist, Track, TrackId};
pub use show::{LiveShow, ShowEvent, ShowSnapshot};
pub use stage::{Stage, StagePreset, StageZone};
pub use vj::{Palette, VJDeck, VJPattern};
