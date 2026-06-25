//! Wave data tier — `kami-terrain`'s default Gerstner ocean waves
//! (`water::default_waves()`) as parity-tested EDN.
//!
//! The per-vertex wave displacement (the Gerstner sum) stays native Rust; only the
//! init-time **description** — the wave-train table — moves to EDN (ADR-0046 / ADR-0038).
//! [`waves_from_edn`] rebuilds the real [`kami_terrain::water::GerstnerWave`] list,
//! asserted wave-for-wave `==` the compiled-in `default_waves()` in
//! `tests/waves_parity.rs`.
//!
//! `GerstnerWave` is `Pod`/`Zeroable` (no `PartialEq`/`Serialize`), so the data tier
//! carries a local [`GerstnerWaveSpec`] `PartialEq` mirror (the GPU-alignment `_pad` is
//! not data — omitted, reconstructed as `[0, 0]`).

use std::collections::BTreeMap;

use kami_scene::{mget, num, root_map, EdnValue};
use kami_terrain::water::{default_waves, GerstnerWave};

/// The canonical wave CONFIG shipped with this crate.
pub const WAVES_EDN: &str = include_str!("../data/waves.edn");

/// Errors raised while loading the wave table from EDN.
#[derive(Debug, thiserror::Error)]
pub enum WaveError {
    /// The EDN source did not parse to a top-level map.
    #[error("waves EDN root is not a map")]
    NotAMap,
    /// The `:terrain/waves` table was missing or not a vector.
    #[error("`:terrain/waves` missing or not a vector")]
    NoTable,
}

/// PartialEq mirror of [`GerstnerWave`] (minus the GPU-alignment `_pad`).
#[derive(Debug, Clone, PartialEq)]
pub struct GerstnerWaveSpec {
    /// Wave direction (normalized XZ).
    pub direction: [f32; 2],
    /// Amplitude (world units).
    pub amplitude: f32,
    /// Wavelength (world units).
    pub wavelength: f32,
    /// Speed (world units / second).
    pub speed: f32,
    /// Steepness Q in [0, 1] (0 = sine, 1 = sharp crest).
    pub steepness: f32,
}

/// Read a 2-vector `[x y]`; missing components default to `0.0`.
fn vec2(v: Option<&EdnValue>) -> [f32; 2] {
    let s = v.and_then(|x| x.as_vector()).unwrap_or(&[]);
    let g = |i: usize| s.get(i).map(|x| num(Some(x))).unwrap_or(0.0);
    [g(0), g(1)]
}

impl GerstnerWaveSpec {
    /// Read the spec straight off the real engine wave (the parity oracle source).
    pub fn from_wave(w: &GerstnerWave) -> Self {
        Self {
            direction: w.direction,
            amplitude: w.amplitude,
            wavelength: w.wavelength,
            speed: w.speed,
            steepness: w.steepness,
        }
    }

    /// Build a spec from one wave's EDN map (tolerant: missing → 0.0).
    pub fn from_map(m: &BTreeMap<EdnValue, EdnValue>) -> Self {
        Self {
            direction: vec2(mget(m, "direction")),
            amplitude: num(mget(m, "amplitude")),
            wavelength: num(mget(m, "wavelength")),
            speed: num(mget(m, "speed")),
            steepness: num(mget(m, "steepness")),
        }
    }
}

/// Reconstruct the real [`GerstnerWave`] from a spec (`_pad` = `[0, 0]`).
pub fn spec_to_wave(s: &GerstnerWaveSpec) -> GerstnerWave {
    GerstnerWave {
        direction: s.direction,
        amplitude: s.amplitude,
        wavelength: s.wavelength,
        speed: s.speed,
        steepness: s.steepness,
        _pad: [0.0; 2],
    }
}

/// Parse the `:terrain/waves` table from EDN `src` into ordered specs.
pub fn wave_specs_from_edn(src: &str) -> Result<Vec<GerstnerWaveSpec>, WaveError> {
    let root = root_map(src).ok_or(WaveError::NotAMap)?;
    let waves = mget(&root, "terrain/waves")
        .and_then(|v| v.as_vector())
        .ok_or(WaveError::NoTable)?;
    Ok(waves
        .iter()
        .filter_map(|w| w.as_map().map(GerstnerWaveSpec::from_map))
        .collect())
}

/// Parse the `:terrain/waves` table from EDN `src` into the real [`GerstnerWave`] list.
pub fn waves_from_edn(src: &str) -> Result<Vec<GerstnerWave>, WaveError> {
    Ok(wave_specs_from_edn(src)?.iter().map(spec_to_wave).collect())
}

/// The compiled-in oracle: `default_waves()` projected into specs.
pub fn builtin_wave_specs() -> Vec<GerstnerWaveSpec> {
    default_waves().iter().map(GerstnerWaveSpec::from_wave).collect()
}

/// Convenience: the waves from the crate-shipped [`WAVES_EDN`].
pub fn shipped_waves() -> Result<Vec<GerstnerWave>, WaveError> {
    waves_from_edn(WAVES_EDN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_has_four_waves() {
        let specs = wave_specs_from_edn(WAVES_EDN).expect("waves.edn parses");
        assert_eq!(specs.len(), 4);
        assert_eq!(specs.len(), builtin_wave_specs().len());
    }

    #[test]
    fn reconstructed_wave_pads_zero() {
        let w = &waves_from_edn(WAVES_EDN).unwrap()[0];
        assert_eq!(w._pad, [0.0; 2]);
    }

    #[test]
    fn non_map_root_is_an_error() {
        assert!(matches!(wave_specs_from_edn("42"), Err(WaveError::NotAMap)));
    }

    #[test]
    fn missing_table_is_an_error() {
        assert!(matches!(wave_specs_from_edn("{:x 1}"), Err(WaveError::NoTable)));
    }
}
