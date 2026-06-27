//! Game Quality Evaluator — Nintendo-grade quality scoring for KAMI Engine games.
//!
//! Evaluates game scenes against the 7 Design Principles (CLAUDE.md)
//! and produces a letter grade (S/A/B/C/D/F) with actionable feedback.
//!
//! Run via: `cargo test -p kami-game --lib quality`
//! Integrate via: `gftd build` post-build gate.

use serde::{Deserialize, Serialize};

/// Quality grade: S (Nintendo-level) through F (unshippable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Grade {
    F = 0,
    D = 1,
    C = 2,
    B = 3,
    A = 4,
    S = 5,
}

impl Grade {
    pub fn from_score(score: f32) -> Self {
        match score as u32 {
            90..=100 => Grade::S,
            75..=89 => Grade::A,
            60..=74 => Grade::B,
            45..=59 => Grade::C,
            25..=44 => Grade::D,
            _ => Grade::F,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Grade::S => "S (Nintendo Quality)",
            Grade::A => "A (Polished)",
            Grade::B => "B (Good)",
            Grade::C => "C (Needs Work)",
            Grade::D => "D (Incomplete)",
            Grade::F => "F (Unshippable)",
        }
    }
}

/// Individual axis evaluation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisResult {
    pub name: String,
    pub weight: f32,
    pub score: f32,
    pub max: f32,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Full game quality report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub game_name: String,
    pub overall_score: f32,
    pub grade: Grade,
    pub axes: Vec<AxisResult>,
    pub blocking_issues: Vec<String>,
}

/// Behavioral liveness signals (Co-Scientist iter-02). Populated from a checked-in
/// `liveness.edn` sidecar that CI recomputes from a committed `events.edn` fixture, so
/// `evaluate` stays a pure fn with no network. A TREND monitor — these influence the
/// Liveness score but are NEVER a hard ship blocker (build-time has no traffic).
#[derive(Debug, Clone, Default)]
pub struct LivenessMeta {
    pub d1_rate: f32,           // day-1 retention (0..1)
    pub d7_rate: f32,           // day-7 retention (0..1)
    pub median_fork_depth: f32, // fork-graph lineage depth (virality)
    pub remix_cta_clicks: u32,  // Remix/fork CTA engagement
}

/// Game scene metadata extracted for quality evaluation.
#[derive(Debug, Clone, Default)]
pub struct GameSceneMeta {
    pub name: String,
    pub entity_count: usize,
    pub sfx_count: usize,
    pub has_bgm: bool,
    pub character_count: usize,
    pub zone_count: usize,
    pub has_spawn: bool,
    pub has_ambient: bool,
    pub has_sun: bool,
    pub genre: String,

    // feedback density
    pub sfx_triggers: Vec<String>,
    pub has_combo_sfx: bool,
    pub has_clear_sfx: bool,
    pub has_fail_sfx: bool,

    // haptic
    pub has_haptic_light: bool,
    pub has_haptic_medium: bool,
    pub has_haptic_heavy: bool,
    pub has_haptic_combo: bool,

    // sound categories
    pub has_ui_sounds: bool,
    pub has_action_sounds: bool,
    pub has_reward_sounds: bool,
    pub has_ambient_sounds: bool,

    // visual effects
    pub has_particles: bool,
    pub has_screen_shake: bool,
    pub has_flash_effect: bool,
    pub has_sparkle_effect: bool,
    pub has_float_text: bool,
    pub has_confetti: bool,

    // game design
    pub has_difficulty_curve: bool,
    pub has_combo_system: bool,
    pub has_score_system: bool,
    pub has_leaderboard: bool,
    pub has_tutorial_hint: bool,
    pub has_time_pressure: bool,
    pub difficulty_levels: usize,
    pub item_variety: usize,

    // input
    pub has_touch_input: bool,
    pub has_keyboard_input: bool,
    pub has_mouse_input: bool,
    pub fullscreen_canvas: bool,
    pub responsive_scaling: bool,

    // ── Liveness (AAA live-service / fork-graph flywheel) — Co-Scientist iter-02 ──
    // Static presence sub-signals, build-checkable from the scene EDN. These HARD-gate the
    // S/A grade: no scene is "AAA-as-a-service" without the retention+distribution flywheel.
    pub has_daily_seed: bool,        // a daily-seed.edn return hook (mc-h1)
    pub has_return_cadence: bool,    // streak / persistent box — a reason to return tomorrow
    pub has_fork_cta: bool,          // Remix/fork CTA wired in the manifest
    pub has_replay_proof: bool,      // server re-derives the score (mc-h3) — metrics are honest
    pub has_collection_spine: bool,  // gacha/collection progression (PANDD-H1)
    /// Behavioral trend signals (sidecar). None ⇒ scored 0 + a suggestion (never faked).
    pub liveness: Option<LivenessMeta>,
}

/// Evaluate game quality against Nintendo-grade standards.
pub fn evaluate(meta: &GameSceneMeta) -> QualityReport {
    let mut axes = Vec::new();

    // ── Axis 1: Engagement (30%) — Tension curve, feedback density, BGM/SFX ──
    {
        let mut score = 0.0_f32;
        let max = 30.0;
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // SFX coverage (0-8)
        let sfx_score = (meta.sfx_count as f32).min(10.0) * 0.8;
        score += sfx_score;
        if meta.sfx_count < 5 {
            issues.push("Fewer than 5 sound effects — game feels silent".into());
            suggestions
                .push("Add: spray-hit, zone-clear, combo, item-complete, fail sounds".into());
        }

        // BGM (0-3)
        if meta.has_bgm {
            score += 3.0
        } else {
            issues.push("No background music".into());
            suggestions.push("Add looping BGM (lo-fi, workshop ambiance, etc.)".into());
        }

        // Sound categories (0-4, 1 each)
        if meta.has_ui_sounds {
            score += 1.0
        } else {
            suggestions.push("Add UI sounds (hover, click, select)".into())
        }
        if meta.has_action_sounds {
            score += 1.0
        } else {
            issues.push("No action sounds".into())
        }
        if meta.has_reward_sounds {
            score += 1.0
        } else {
            suggestions.push("Add reward sounds (coin, success, fanfare)".into())
        }
        if meta.has_ambient_sounds {
            score += 1.0
        }

        // Combo SFX (0-2)
        if meta.has_combo_sfx {
            score += 2.0
        } else {
            suggestions.push("Add rising-pitch combo sound effect".into());
        }

        // Particles + visual feedback (0-6)
        let vfx = [
            meta.has_particles,
            meta.has_screen_shake,
            meta.has_flash_effect,
            meta.has_sparkle_effect,
            meta.has_float_text,
            meta.has_confetti,
        ];
        let vfx_count = vfx.iter().filter(|&&v| v).count();
        score += vfx_count as f32;
        if vfx_count < 3 {
            issues.push(format!(
                "Only {}/6 visual effect types — low juice",
                vfx_count
            ));
            suggestions
                .push("Add: particles, screen shake, flash, sparkle, float text, confetti".into());
        }

        // Haptic (0-4)
        let haptics = [
            meta.has_haptic_light,
            meta.has_haptic_medium,
            meta.has_haptic_heavy,
            meta.has_haptic_combo,
        ];
        let haptic_count = haptics.iter().filter(|&&v| v).count();
        score += haptic_count as f32;
        if haptic_count == 0 {
            issues.push("No haptic vibration — mobile feels dead".into());
            suggestions
                .push("Add navigator.vibrate() for light/medium/heavy/combo feedback".into());
        }

        axes.push(AxisResult {
            name: "Engagement".into(),
            weight: 0.30,
            score: score.min(max),
            max,
            issues,
            suggestions,
        });
    }

    // ── Axis 2: Competence (20%) — Tutorial, difficulty curve, mastery depth ──
    {
        let mut score = 0.0_f32;
        let max = 20.0;
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        if meta.has_tutorial_hint {
            score += 4.0
        } else {
            issues.push("No tutorial or control hints".into());
            suggestions.push("Show control hints on title screen".into());
        }
        if meta.has_difficulty_curve {
            score += 5.0
        } else {
            issues.push("No difficulty progression".into());
            suggestions
                .push("Items should increase in difficulty (more zones, harder rust types)".into());
        }
        if meta.difficulty_levels >= 3 {
            score += 3.0
        } else if meta.difficulty_levels >= 2 {
            score += 2.0
        } else {
            suggestions.push("Add at least 3 difficulty selections (Easy/Normal/Hard)".into())
        }

        if meta.has_combo_system {
            score += 4.0
        } else {
            suggestions.push("Add combo system for mastery depth".into());
        }
        if meta.item_variety >= 6 {
            score += 4.0
        } else if meta.item_variety >= 4 {
            score += 3.0
        } else {
            suggestions.push("Add more item variety (target: 6+)".into())
        }

        axes.push(AxisResult {
            name: "Competence".into(),
            weight: 0.20,
            score: score.min(max),
            max,
            issues,
            suggestions,
        });
    }

    // ── Axis 3: Contribution (15%) — Leaderboard, social share ──
    {
        let mut score = 0.0_f32;
        let max = 15.0;
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        if meta.has_leaderboard {
            score += 5.0
        } else {
            suggestions.push("Add leaderboard (XRPC submit + query)".into());
        }
        if meta.has_score_system {
            score += 5.0
        } else {
            issues.push("No scoring system".into());
        }
        // Grade display
        score += 3.0; // assume grade display exists in result screen
        score += 2.0; // AT Protocol social post on clear

        axes.push(AxisResult {
            name: "Contribution".into(),
            weight: 0.15,
            score: score.min(max),
            max,
            issues,
            suggestions,
        });
    }

    // ── Axis 4: Growth (20%) — Progression, replay value ──
    {
        let mut score = 0.0_f32;
        let max = 20.0;
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        if meta.has_time_pressure {
            score += 4.0
        } else {
            suggestions.push("Add time pressure for urgency".into());
        }
        if meta.item_variety >= 8 {
            score += 5.0
        } else if meta.item_variety >= 5 {
            score += 3.0
        } else {
            suggestions.push("More items = more replay value".into())
        }

        if meta.has_difficulty_curve {
            score += 4.0
        }
        if meta.has_combo_system {
            score += 3.0
        }
        score += 4.0; // perfect bonus system

        axes.push(AxisResult {
            name: "Growth".into(),
            weight: 0.20,
            score: score.min(max),
            max,
            issues,
            suggestions,
        });
    }

    // ── Axis 5: Resilience (15%) — Input quality, responsiveness ──
    {
        let mut score = 0.0_f32;
        let max = 15.0;
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        if meta.has_touch_input {
            score += 3.0
        } else {
            issues.push("No touch input — unplayable on mobile".into());
        }
        if meta.has_keyboard_input {
            score += 2.0
        }
        if meta.has_mouse_input {
            score += 2.0
        }
        if meta.fullscreen_canvas {
            score += 4.0
        } else {
            issues.push("Canvas is not fullscreen — wasted screen space".into());
            suggestions.push("Use window.innerWidth/Height with devicePixelRatio scaling".into());
        }
        if meta.responsive_scaling {
            score += 4.0
        } else {
            issues.push("No responsive scaling".into());
            suggestions.push("Scale all coordinates by Math.min(W/800, H/600)".into());
        }

        axes.push(AxisResult {
            name: "Resilience".into(),
            weight: 0.15,
            score: score.min(max),
            max,
            issues,
            suggestions,
        });
    }

    // ── Axis 6: Liveness (15%) — Retention cadence + fork-graph distribution (AAA-as-a-service) ──
    // The dimension the SDT-5 rubric is blind to: does the game give a reason to return, and
    // does it propagate as data? Static presence is the hard part of the gate; the sidecar's
    // behavioral signals are a trend layer that can never block a build.
    {
        let mut score = 0.0_f32;
        let max = 15.0;
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // Static presence (0-10, 2 each) — build-checkable.
        let statics = [
            (meta.has_daily_seed, "a daily-seed return hook (daily-seed.edn)"),
            (meta.has_return_cadence, "a return cadence (login streak / persistent box)"),
            (meta.has_fork_cta, "a Remix/fork CTA in the manifest"),
            (meta.has_replay_proof, "replay-proof score admission (no spoofable metrics)"),
            (meta.has_collection_spine, "a collection/gacha progression spine"),
        ];
        for (present, label) in statics {
            if present {
                score += 2.0
            } else {
                suggestions.push(format!("Add {}", label))
            }
        }

        // Behavioral trend (0-5) from the sidecar — NEVER a hard blocker.
        match &meta.liveness {
            Some(l) => {
                score += l.d1_rate.clamp(0.0, 1.0) * 2.0; // 0-2
                score += l.d7_rate.clamp(0.0, 1.0) * 1.5; // 0-1.5
                score += (l.median_fork_depth / 4.0).clamp(0.0, 1.0) * 1.5; // 0-1.5
            }
            None => suggestions
                .push("No liveness.edn sidecar — D1/D7/fork-depth unmeasured (trend only)".into()),
        }

        if !meta.has_replay_proof {
            issues.push("No replay-proof — retention metrics would be spoofable".into());
        }

        axes.push(AxisResult {
            name: "Liveness".into(),
            weight: 0.15,
            score: score.min(max),
            max,
            issues,
            suggestions,
        });
    }

    // ── Compute Overall ──
    // Normalize by Σweight so adding an axis (Liveness pushes Σweight to 1.15) never inflates
    // the achievable max past 100 — otherwise every S/A threshold would silently drift.
    let total_w: f32 = axes.iter().map(|a| a.weight).sum();
    let overall_raw: f32 = axes
        .iter()
        .map(|a| (a.score / a.max) * a.weight * 100.0)
        .sum();
    let mut overall = if total_w > 0.0 { overall_raw / total_w } else { 0.0 };

    let mut blocking_issues: Vec<String> = axes
        .iter()
        .filter(|a| a.score / a.max < 0.3)
        .map(|a| format!("{}: score {:.0}/{:.0} — BLOCKING", a.name, a.score, a.max))
        .collect();

    // ── Liveness HARD gate ── the static flywheel must be wired to reach S/A. Behavioral
    // signals never gate; only presence of replay-proof + daily-seed + fork-CTA does.
    let flywheel_wired = meta.has_replay_proof && meta.has_daily_seed && meta.has_fork_cta;
    if !flywheel_wired && overall > 74.0 {
        overall = 74.0;
        blocking_issues
            .push("Liveness gate: not S/A without replay-proof + daily-seed + fork-CTA wired".into());
    }
    let grade = Grade::from_score(overall);

    QualityReport {
        game_name: meta.name.clone(),
        overall_score: overall,
        grade,
        axes,
        blocking_issues,
    }
}

impl std::fmt::Display for QualityReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "╔══════════════════════════════════════════╗")?;
        writeln!(f, "║  KAMI Game Quality Report                ║")?;
        writeln!(f, "╠══════════════════════════════════════════╣")?;
        writeln!(f, "║  Game: {:<33}║", self.game_name)?;
        writeln!(f, "║  Grade: {:<32}║", self.grade.label())?;
        writeln!(
            f,
            "║  Score: {:.1}/100                          ║",
            self.overall_score
        )?;
        writeln!(f, "╠══════════════════════════════════════════╣")?;
        for axis in &self.axes {
            let pct = if axis.max > 0.0 {
                axis.score / axis.max * 100.0
            } else {
                0.0
            };
            let bar_len = (pct / 5.0) as usize;
            let bar: String = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
            writeln!(
                f,
                "║ {:12} {:.0}/{:.0} ({:.0}%) {} ║",
                axis.name, axis.score, axis.max, pct, bar
            )?;
            for issue in &axis.issues {
                writeln!(f, "║   ✗ {:<36}║", issue)?;
            }
        }
        if !self.blocking_issues.is_empty() {
            writeln!(f, "╠══════════════════════════════════════════╣")?;
            writeln!(f, "║  BLOCKING ISSUES:                        ║")?;
            for b in &self.blocking_issues {
                writeln!(f, "║  ⚠ {:<37}║", b)?;
            }
        }
        writeln!(f, "╚══════════════════════════════════════════╝")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sabiotoshi_meta() -> GameSceneMeta {
        GameSceneMeta {
            name: "Sabi-Otoshi!!".into(),
            entity_count: 25,
            sfx_count: 14,
            has_bgm: false, // TODO: add BGM loop
            character_count: 1,
            zone_count: 44,
            has_spawn: true,
            has_ambient: true,
            has_sun: true,
            genre: "puzzle".into(),
            sfx_triggers: vec![
                "sprayStart",
                "sprayLoop",
                "sprayHit",
                "rustCrack",
                "zoneClear",
                "comboDing",
                "itemComplete",
                "perfectFinish",
                "allClear",
                "nozzleSwitch",
                "titleSelect",
                "titleHover",
                "timeout",
                "tick",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            has_combo_sfx: true,
            has_clear_sfx: true,
            has_fail_sfx: true,
            has_haptic_light: true,
            has_haptic_medium: true,
            has_haptic_heavy: true,
            has_haptic_combo: true,
            has_ui_sounds: true,
            has_action_sounds: true,
            has_reward_sounds: true,
            has_ambient_sounds: false,
            has_particles: true,
            has_screen_shake: true,
            has_flash_effect: true,
            has_sparkle_effect: true,
            has_float_text: true,
            has_confetti: true,
            has_difficulty_curve: true,
            has_combo_system: true,
            has_score_system: true,
            has_leaderboard: true,
            has_tutorial_hint: true,
            has_time_pressure: true,
            difficulty_levels: 3,
            item_variety: 8,
            has_touch_input: true,
            has_keyboard_input: true,
            has_mouse_input: true,
            fullscreen_canvas: true,
            responsive_scaling: true,
            // Liveness: the retention/distribution substrate (mc-h1/mc-h3/ls-h1/PANDD-H1) is
            // now shipped, so the reference game wires the flywheel.
            has_daily_seed: true,
            has_return_cadence: true,
            has_fork_cta: true,
            has_replay_proof: true,
            has_collection_spine: true,
            liveness: Some(LivenessMeta {
                d1_rate: 0.4,
                d7_rate: 0.2,
                median_fork_depth: 3.0,
                remix_cta_clicks: 120,
            }),
        }
    }

    #[test]
    fn sabiotoshi_quality_grade_a_or_above() {
        let meta = sabiotoshi_meta();
        let report = evaluate(&meta);
        println!("{}", report);
        assert!(
            report.grade >= Grade::A,
            "Sabi-Otoshi!! should be grade A or above, got {:?} ({:.1})",
            report.grade,
            report.overall_score
        );
        assert!(
            report.blocking_issues.is_empty(),
            "No blocking issues expected: {:?}",
            report.blocking_issues
        );
    }

    #[test]
    fn empty_game_grades_f() {
        let meta = GameSceneMeta::default();
        let report = evaluate(&meta);
        assert_eq!(report.grade, Grade::F);
        assert!(!report.blocking_issues.is_empty());
    }

    #[test]
    fn grade_from_score_boundaries() {
        assert_eq!(Grade::from_score(95.0), Grade::S);
        assert_eq!(Grade::from_score(80.0), Grade::A);
        assert_eq!(Grade::from_score(65.0), Grade::B);
        assert_eq!(Grade::from_score(50.0), Grade::C);
        assert_eq!(Grade::from_score(30.0), Grade::D);
        assert_eq!(Grade::from_score(10.0), Grade::F);
    }

    #[test]
    fn missing_haptic_lowers_engagement() {
        let mut meta = sabiotoshi_meta();
        meta.has_haptic_light = false;
        meta.has_haptic_medium = false;
        meta.has_haptic_heavy = false;
        meta.has_haptic_combo = false;
        let report = evaluate(&meta);
        let engagement = report.axes.iter().find(|a| a.name == "Engagement").unwrap();
        assert!(!engagement.issues.is_empty(), "Should flag missing haptics");
    }

    #[test]
    fn no_touch_input_blocks_resilience() {
        let mut meta = sabiotoshi_meta();
        meta.has_touch_input = false;
        let report = evaluate(&meta);
        let resilience = report.axes.iter().find(|a| a.name == "Resilience").unwrap();
        assert!(resilience.issues.iter().any(|i| i.contains("touch")));
    }

    #[test]
    fn liveness_axis_present_and_overall_capped_at_100() {
        // Max out every liveness signal; the 6th axis (Σweight now 1.15) must not push the
        // normalized overall past 100.
        let mut meta = sabiotoshi_meta();
        meta.liveness = Some(LivenessMeta {
            d1_rate: 1.0,
            d7_rate: 1.0,
            median_fork_depth: 20.0,
            remix_cta_clicks: 9999,
        });
        let report = evaluate(&meta);
        assert!(
            report.axes.iter().any(|a| a.name == "Liveness"),
            "Liveness axis should be present"
        );
        assert!(
            report.overall_score <= 100.0,
            "overall {} must stay <= 100 after adding the 0.15 axis",
            report.overall_score
        );
    }

    #[test]
    fn liveness_gate_blocks_s_a_without_flywheel() {
        // A game missing replay-proof cannot be AAA-as-a-service, regardless of polish.
        let mut meta = sabiotoshi_meta();
        meta.has_replay_proof = false;
        let report = evaluate(&meta);
        assert!(
            report.grade < Grade::A,
            "must not reach A/S without replay-proof, got {:?} ({:.1})",
            report.grade,
            report.overall_score
        );
        assert!(
            report
                .blocking_issues
                .iter()
                .any(|i| i.contains("Liveness gate")),
            "expected the Liveness gate blocking issue"
        );
    }

    #[test]
    fn liveness_missing_sidecar_degrades_honestly() {
        let mut meta = sabiotoshi_meta();
        meta.liveness = None; // no behavioral data
        let report = evaluate(&meta);
        let liveness = report.axes.iter().find(|a| a.name == "Liveness").unwrap();
        assert!(
            liveness.suggestions.iter().any(|s| s.contains("sidecar")),
            "should suggest adding the liveness.edn sidecar, not fake the data"
        );
    }

    #[test]
    fn small_canvas_penalizes() {
        let mut meta = sabiotoshi_meta();
        meta.fullscreen_canvas = false;
        meta.responsive_scaling = false;
        let before = evaluate(&sabiotoshi_meta());
        let after = evaluate(&meta);
        assert!(after.overall_score < before.overall_score);
    }
}
