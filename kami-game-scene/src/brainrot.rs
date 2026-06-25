//! Brainrot-evolution data tier — `kami-game`'s Pokémon-style evolution chains
//! (`island_gen::brainrot_evolution_chains()`) as parity-tested EDN.
//!
//! Evolution dispatch / mesh generation stays native Rust; only the init-time
//! **description** — the per-character stage table (gates, scale, appearance overrides) —
//! moves to EDN (ADR-0046 / ADR-0038). [`chains_from_edn`] rebuilds the real
//! [`kami_game::island_gen::BrainrotEvolution`] list, asserted chain-for-chain,
//! stage-for-stage `==` the compiled-in `brainrot_evolution_chains()` in
//! `tests/brainrot_parity.rs`.
//!
//! `BrainrotEvolution` / `EvolutionStage` derive no `PartialEq`, so the data tier carries
//! local [`EvolutionChainSpec`] / [`EvolutionStageSpec`] `PartialEq` mirrors (`:character`
//! ↔ a hyphenated [`BrainrotCharacter`] keyword id).

use std::collections::BTreeMap;

use kami_game::brainrot_mesh::BrainrotCharacter;
use kami_game::island_gen::{brainrot_evolution_chains, BrainrotEvolution, EvolutionStage};
use kami_scene::{kw_key, mget, num, root_map, EdnValue};

/// The canonical brainrot-evolution CONFIG shipped with this crate.
pub const BRAINROT_EVOLUTION_EDN: &str = include_str!("../data/brainrot_evolution.edn");

/// Errors raised while loading the brainrot-evolution table from EDN.
#[derive(Debug, thiserror::Error)]
pub enum BrainrotError {
    /// The EDN source did not parse to a top-level map.
    #[error("brainrot-evolution EDN root is not a map")]
    NotAMap,
    /// The `:game/brainrot-evolution` table was missing or not a vector.
    #[error("`:game/brainrot-evolution` missing or not a vector")]
    NoTable,
}

/// PartialEq mirror of [`EvolutionStage`] (which derives none), for parity.
#[derive(Debug, Clone, PartialEq)]
pub struct EvolutionStageSpec {
    /// Stage index (0 = base form).
    pub stage: u8,
    /// Form name (e.g. `"Skibidi Tank"`).
    pub form_name: String,
    /// Social (Well-Becoming rank) gate; `""` when none.
    pub social_gate: String,
    /// Domain achievement gate; `""` when none.
    pub domain_gate: String,
    /// Mesh scale at this stage.
    pub scale: f32,
    /// Body-build appearance override, if any.
    pub body_override: Option<String>,
    /// Accessory appearance override, if any.
    pub accessory_override: Option<String>,
}

/// PartialEq mirror of [`BrainrotEvolution`].
#[derive(Debug, Clone, PartialEq)]
pub struct EvolutionChainSpec {
    /// Stable character id (e.g. `"char-skibidi-commander"`).
    pub character_id: String,
    /// Hyphenated [`BrainrotCharacter`] keyword id (e.g. `"skibidi"`).
    pub character: String,
    /// Ordered evolution stages.
    pub stages: Vec<EvolutionStageSpec>,
}

/// The hyphenated `:character` keyword id for a [`BrainrotCharacter`] variant.
pub fn character_id(c: BrainrotCharacter) -> &'static str {
    match c {
        BrainrotCharacter::Skibidi => "skibidi",
        BrainrotCharacter::Sigma => "sigma",
        BrainrotCharacter::Ohio => "ohio",
        BrainrotCharacter::Grimace => "grimace",
        BrainrotCharacter::Rizz => "rizz",
        BrainrotCharacter::Fanum => "fanum",
    }
}

/// Inverse of [`character_id`]; unknown ids fall back to `Skibidi` (tolerant).
pub fn character_from_id(id: &str) -> BrainrotCharacter {
    match id {
        "sigma" => BrainrotCharacter::Sigma,
        "ohio" => BrainrotCharacter::Ohio,
        "grimace" => BrainrotCharacter::Grimace,
        "rizz" => BrainrotCharacter::Rizz,
        "fanum" => BrainrotCharacter::Fanum,
        _ => BrainrotCharacter::Skibidi,
    }
}

type Map = BTreeMap<EdnValue, EdnValue>;

fn opt_str(m: &Map, key: &str) -> Option<String> {
    mget(m, key).and_then(|v| v.as_string()).map(str::to_string)
}
fn str_or_empty(m: &Map, key: &str) -> String {
    opt_str(m, key).unwrap_or_default()
}

impl EvolutionStageSpec {
    /// Read the spec straight off the real engine stage (the parity oracle source).
    pub fn from_stage(s: &EvolutionStage) -> Self {
        Self {
            stage: s.stage,
            form_name: s.form_name.clone(),
            social_gate: s.social_gate.clone(),
            domain_gate: s.domain_gate.clone(),
            scale: s.scale,
            body_override: s.body_override.clone(),
            accessory_override: s.accessory_override.clone(),
        }
    }

    /// Build a spec from one stage's EDN map (tolerant: missing → default/None).
    pub fn from_map(m: &Map) -> Self {
        Self {
            stage: mget(m, "stage").and_then(|v| v.as_integer()).unwrap_or(0).clamp(0, 255) as u8,
            form_name: str_or_empty(m, "form"),
            social_gate: str_or_empty(m, "social-gate"),
            domain_gate: str_or_empty(m, "domain-gate"),
            scale: num(mget(m, "scale")),
            body_override: opt_str(m, "body"),
            accessory_override: opt_str(m, "accessory"),
        }
    }
}

impl EvolutionChainSpec {
    /// Read the spec straight off the real engine chain (the parity oracle source).
    pub fn from_chain(c: &BrainrotEvolution) -> Self {
        Self {
            character_id: c.character_id.clone(),
            character: character_id(c.character_enum).to_string(),
            stages: c.stages.iter().map(EvolutionStageSpec::from_stage).collect(),
        }
    }

    /// Build a spec from one chain's EDN map.
    pub fn from_map(m: &Map) -> Self {
        let stages = mget(m, "stages")
            .and_then(|v| v.as_vector())
            .unwrap_or(&[])
            .iter()
            .filter_map(|s| s.as_map().map(EvolutionStageSpec::from_map))
            .collect();
        Self {
            character_id: str_or_empty(m, "character-id"),
            character: mget(m, "character").and_then(kw_key).unwrap_or_default(),
            stages,
        }
    }
}

/// Reconstruct the real [`EvolutionStage`] from a spec.
pub fn spec_to_stage(s: &EvolutionStageSpec) -> EvolutionStage {
    EvolutionStage {
        stage: s.stage,
        form_name: s.form_name.clone(),
        social_gate: s.social_gate.clone(),
        domain_gate: s.domain_gate.clone(),
        scale: s.scale,
        body_override: s.body_override.clone(),
        accessory_override: s.accessory_override.clone(),
    }
}

/// Reconstruct the real [`BrainrotEvolution`] from a spec.
pub fn spec_to_chain(s: &EvolutionChainSpec) -> BrainrotEvolution {
    BrainrotEvolution {
        character_id: s.character_id.clone(),
        character_enum: character_from_id(&s.character),
        stages: s.stages.iter().map(spec_to_stage).collect(),
    }
}

/// Parse the `:game/brainrot-evolution` table from EDN `src` into ordered specs.
pub fn chain_specs_from_edn(src: &str) -> Result<Vec<EvolutionChainSpec>, BrainrotError> {
    let root = root_map(src).ok_or(BrainrotError::NotAMap)?;
    let chains = mget(&root, "game/brainrot-evolution")
        .and_then(|v| v.as_vector())
        .ok_or(BrainrotError::NoTable)?;
    Ok(chains
        .iter()
        .filter_map(|c| c.as_map().map(EvolutionChainSpec::from_map))
        .collect())
}

/// Parse the table from EDN `src` into the real [`BrainrotEvolution`] list.
pub fn chains_from_edn(src: &str) -> Result<Vec<BrainrotEvolution>, BrainrotError> {
    Ok(chain_specs_from_edn(src)?.iter().map(spec_to_chain).collect())
}

/// The compiled-in oracle: `brainrot_evolution_chains()` projected into specs.
pub fn builtin_chain_specs() -> Vec<EvolutionChainSpec> {
    brainrot_evolution_chains()
        .iter()
        .map(EvolutionChainSpec::from_chain)
        .collect()
}

/// Convenience: the chains from the crate-shipped [`BRAINROT_EVOLUTION_EDN`].
pub fn shipped_chains() -> Result<Vec<BrainrotEvolution>, BrainrotError> {
    chains_from_edn(BRAINROT_EVOLUTION_EDN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_has_six_chains() {
        let specs = chain_specs_from_edn(BRAINROT_EVOLUTION_EDN).expect("parses");
        assert_eq!(specs.len(), 6);
        assert_eq!(specs.len(), builtin_chain_specs().len());
        let total: usize = specs.iter().map(|c| c.stages.len()).sum();
        assert_eq!(total, 23, "23 stages across all chains");
    }

    #[test]
    fn character_id_round_trips() {
        for c in [
            BrainrotCharacter::Skibidi,
            BrainrotCharacter::Sigma,
            BrainrotCharacter::Ohio,
            BrainrotCharacter::Grimace,
            BrainrotCharacter::Rizz,
            BrainrotCharacter::Fanum,
        ] {
            assert_eq!(character_from_id(character_id(c)), c);
        }
    }

    #[test]
    fn non_map_root_is_an_error() {
        assert!(matches!(chain_specs_from_edn("42"), Err(BrainrotError::NotAMap)));
    }

    #[test]
    fn missing_table_is_an_error() {
        assert!(matches!(chain_specs_from_edn("{:x 1}"), Err(BrainrotError::NoTable)));
    }
}
