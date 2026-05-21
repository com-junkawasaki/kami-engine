//! Ranked system: ELO/MMR for KAMI Battle Royale.
//!
//! Placement-based scoring (kills + placement) with seasonal ranks.
//! Bronze → Silver → Gold → Platinum → Diamond → Champion → Unreal.

use serde::{Deserialize, Serialize};

// ── Rank Tiers ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RankTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
    Champion,
    Unreal,
}

impl RankTier {
    pub fn from_mmr(mmr: i32) -> Self {
        match mmr {
            ..=499 => RankTier::Bronze,
            500..=999 => RankTier::Silver,
            1000..=1499 => RankTier::Gold,
            1500..=1999 => RankTier::Platinum,
            2000..=2999 => RankTier::Diamond,
            3000..=4999 => RankTier::Champion,
            _ => RankTier::Unreal,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            RankTier::Bronze => "Bronze",
            RankTier::Silver => "Silver",
            RankTier::Gold => "Gold",
            RankTier::Platinum => "Platinum",
            RankTier::Diamond => "Diamond",
            RankTier::Champion => "Champion",
            RankTier::Unreal => "Unreal",
        }
    }

    pub fn division_count(self) -> u8 {
        match self {
            RankTier::Bronze | RankTier::Silver | RankTier::Gold | RankTier::Platinum => 3,
            RankTier::Diamond => 3,
            RankTier::Champion | RankTier::Unreal => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RankDivision {
    pub tier: RankTier,
    pub division: u8, // 1-3 (III, II, I), 0 for Champion/Unreal
}

impl RankDivision {
    pub fn from_mmr(mmr: i32) -> Self {
        let tier = RankTier::from_mmr(mmr);
        let division = match tier {
            RankTier::Bronze => match mmr {
                ..=166 => 3,
                167..=333 => 2,
                _ => 1,
            },
            RankTier::Silver => match mmr {
                ..=666 => 3,
                667..=833 => 2,
                _ => 1,
            },
            RankTier::Gold => match mmr {
                ..=1166 => 3,
                1167..=1333 => 2,
                _ => 1,
            },
            RankTier::Platinum => match mmr {
                ..=1666 => 3,
                1667..=1833 => 2,
                _ => 1,
            },
            RankTier::Diamond => match mmr {
                ..=2333 => 3,
                2334..=2666 => 2,
                _ => 1,
            },
            RankTier::Champion | RankTier::Unreal => 0,
        };
        Self { tier, division }
    }

    pub fn display(&self) -> String {
        if self.division == 0 {
            self.tier.display_name().to_string()
        } else {
            let div_str = match self.division {
                3 => "III",
                2 => "II",
                1 => "I",
                _ => "",
            };
            format!("{} {}", self.tier.display_name(), div_str)
        }
    }
}

// ── Player Ranked Profile ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedProfile {
    pub player_did: String,
    pub display_name: String,
    pub mmr: i32,
    pub peak_mmr: i32,
    pub rank: RankDivision,
    pub season: u16,
    pub matches_played: u32,
    pub wins: u32,
    pub top5: u32,
    pub top10: u32,
    pub top25: u32,
    pub total_kills: u32,
    pub total_damage: u64,
    pub total_builds: u32,
    pub avg_placement: f32,
    pub kd_ratio: f32,
    pub win_rate: f32,
    pub current_streak: i32, // positive = win streak, negative = loss streak
    pub best_streak: u32,
    pub demotion_shield: bool,
}

impl RankedProfile {
    pub fn new(player_did: &str, display_name: &str, season: u16) -> Self {
        Self {
            player_did: player_did.to_string(),
            display_name: display_name.to_string(),
            mmr: 0,
            peak_mmr: 0,
            rank: RankDivision::from_mmr(0),
            season,
            matches_played: 0,
            wins: 0,
            top5: 0,
            top10: 0,
            top25: 0,
            total_kills: 0,
            total_damage: 0,
            total_builds: 0,
            avg_placement: 0.0,
            kd_ratio: 0.0,
            win_rate: 0.0,
            current_streak: 0,
            best_streak: 0,
            demotion_shield: false,
        }
    }

    pub fn update_rank(&mut self) {
        self.rank = RankDivision::from_mmr(self.mmr);
        if self.mmr > self.peak_mmr {
            self.peak_mmr = self.mmr;
        }
    }
}

// ── Match Scoring ──

/// Points awarded by placement (100-player lobby).
pub fn placement_points(placement: u16, total_players: u16) -> i32 {
    match placement {
        1 => 120,
        2 => 85,
        3 => 70,
        4 => 60,
        5 => 55,
        6..=10 => 40,
        11..=15 => 25,
        16..=20 => 15,
        21..=25 => 10,
        26..=30 => 5,
        31..=40 => 2,
        41..=50 => 0,
        _ => -10 - ((placement as i32 - 50) / 10).min(20), // bus fee
    }
}

/// Points per elimination.
pub fn kill_points(kills: u16, current_rank: RankTier) -> i32 {
    let base = match current_rank {
        RankTier::Bronze | RankTier::Silver => 20,
        RankTier::Gold | RankTier::Platinum => 18,
        RankTier::Diamond => 15,
        RankTier::Champion => 12,
        RankTier::Unreal => 10,
    };
    kills as i32 * base
}

/// Entry fee (bus fee) — higher ranks pay more.
pub fn entry_fee(rank: RankTier) -> i32 {
    match rank {
        RankTier::Bronze => 0,
        RankTier::Silver => -10,
        RankTier::Gold => -20,
        RankTier::Platinum => -30,
        RankTier::Diamond => -50,
        RankTier::Champion => -60,
        RankTier::Unreal => -70,
    }
}

/// Calculate total MMR change for a match result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub placement: u16,
    pub total_players: u16,
    pub kills: u16,
    pub assists: u16,
    pub damage_dealt: u32,
    pub builds_placed: u32,
    pub survival_time_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MMRChange {
    pub placement_points: i32,
    pub kill_points: i32,
    pub bus_fee: i32,
    pub bonus: i32,
    pub total: i32,
    pub new_mmr: i32,
    pub old_rank: RankDivision,
    pub new_rank: RankDivision,
    pub promoted: bool,
    pub demoted: bool,
}

pub fn calculate_mmr_change(profile: &RankedProfile, result: &MatchResult) -> MMRChange {
    let old_rank = profile.rank;
    let bus_fee = entry_fee(old_rank.tier);
    let placement_pts = placement_points(result.placement, result.total_players);
    let kill_pts = kill_points(result.kills, old_rank.tier);

    // Bonus for high damage / assists
    let damage_bonus = (result.damage_dealt / 500) as i32;
    let assist_bonus = result.assists as i32 * 5;
    let bonus = damage_bonus + assist_bonus;

    let total = bus_fee + placement_pts + kill_pts + bonus;

    // Demotion protection at tier floors
    let new_mmr_raw = profile.mmr + total;
    let tier_floor = match old_rank.tier {
        RankTier::Bronze => 0,
        RankTier::Silver => 480,
        RankTier::Gold => 980,
        RankTier::Platinum => 1480,
        RankTier::Diamond => 1980,
        RankTier::Champion => 2980,
        RankTier::Unreal => 4980,
    };
    let new_mmr = if profile.demotion_shield && new_mmr_raw < tier_floor {
        tier_floor
    } else {
        new_mmr_raw.max(0)
    };

    let new_rank = RankDivision::from_mmr(new_mmr);
    let promoted = new_rank.tier > old_rank.tier
        || (new_rank.tier == old_rank.tier && new_rank.division < old_rank.division);
    let demoted = new_rank.tier < old_rank.tier
        || (new_rank.tier == old_rank.tier && new_rank.division > old_rank.division);

    MMRChange {
        placement_points: placement_pts,
        kill_points: kill_pts,
        bus_fee,
        bonus,
        total,
        new_mmr,
        old_rank,
        new_rank,
        promoted,
        demoted,
    }
}

/// Apply match result to player profile.
pub fn apply_match_result(profile: &mut RankedProfile, result: &MatchResult) -> MMRChange {
    let change = calculate_mmr_change(profile, result);

    profile.mmr = change.new_mmr;
    profile.matches_played += 1;
    profile.total_kills += result.kills as u32;
    profile.total_damage += result.damage_dealt as u64;
    profile.total_builds += result.builds_placed;

    // Update win/top stats
    if result.placement == 1 {
        profile.wins += 1;
    }
    if result.placement <= 5 {
        profile.top5 += 1;
    }
    if result.placement <= 10 {
        profile.top10 += 1;
    }
    if result.placement <= 25 {
        profile.top25 += 1;
    }

    // Average placement
    let n = profile.matches_played as f32;
    profile.avg_placement = ((profile.avg_placement * (n - 1.0)) + result.placement as f32) / n;

    // K/D ratio
    let deaths = profile.matches_played - profile.wins;
    profile.kd_ratio = if deaths > 0 {
        profile.total_kills as f32 / deaths as f32
    } else {
        profile.total_kills as f32
    };

    // Win rate
    profile.win_rate = profile.wins as f32 / profile.matches_played as f32 * 100.0;

    // Streak
    if result.placement == 1 {
        if profile.current_streak >= 0 {
            profile.current_streak += 1;
        } else {
            profile.current_streak = 1;
        }
    } else if result.placement > 50 {
        if profile.current_streak <= 0 {
            profile.current_streak -= 1;
        } else {
            profile.current_streak = -1;
        }
    }
    if profile.current_streak > 0 {
        profile.best_streak = profile.best_streak.max(profile.current_streak as u32);
    }

    // Demotion shield: one free game after promotion
    profile.demotion_shield = change.promoted;

    profile.update_rank();
    change
}

// ── Matchmaking ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchmakingEntry {
    pub player_did: String,
    pub mmr: i32,
    pub rank: RankTier,
    pub queue_time: f32,
}

/// Group players into a lobby by MMR proximity.
pub fn matchmake(
    queue: &[MatchmakingEntry],
    target_size: usize,
    max_mmr_spread: i32,
) -> Vec<Vec<usize>> {
    if queue.is_empty() {
        return Vec::new();
    }

    let mut sorted_indices: Vec<usize> = (0..queue.len()).collect();
    sorted_indices.sort_by_key(|&i| queue[i].mmr);

    let mut lobbies = Vec::new();
    let mut current_lobby = Vec::new();
    let mut lobby_min_mmr = queue[sorted_indices[0]].mmr;

    for &idx in &sorted_indices {
        let entry = &queue[idx];
        // Widen bracket if player waited >60s
        let spread = if entry.queue_time > 60.0 {
            max_mmr_spread * 2
        } else {
            max_mmr_spread
        };

        if entry.mmr - lobby_min_mmr <= spread && current_lobby.len() < target_size {
            current_lobby.push(idx);
        } else if current_lobby.len() >= target_size / 2 {
            lobbies.push(std::mem::take(&mut current_lobby));
            current_lobby.push(idx);
            lobby_min_mmr = entry.mmr;
        } else {
            current_lobby.push(idx);
        }
    }
    if !current_lobby.is_empty() && current_lobby.len() >= target_size / 2 {
        lobbies.push(current_lobby);
    }
    lobbies
}

// ── Season ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    pub season_number: u16,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub active: bool,
    pub soft_reset_ratio: f32, // e.g. 0.5 = new MMR = old/2
}

impl Season {
    pub fn soft_reset_mmr(&self, old_mmr: i32) -> i32 {
        (old_mmr as f32 * self.soft_reset_ratio) as i32
    }
}

// ── Ranked Leaderboard ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank_position: u32,
    pub player_did: String,
    pub display_name: String,
    pub mmr: i32,
    pub rank: RankDivision,
    pub wins: u32,
    pub kills: u32,
    pub kd_ratio: f32,
    pub matches_played: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_tiers() {
        assert_eq!(RankTier::from_mmr(0), RankTier::Bronze);
        assert_eq!(RankTier::from_mmr(500), RankTier::Silver);
        assert_eq!(RankTier::from_mmr(1000), RankTier::Gold);
        assert_eq!(RankTier::from_mmr(1500), RankTier::Platinum);
        assert_eq!(RankTier::from_mmr(2000), RankTier::Diamond);
        assert_eq!(RankTier::from_mmr(3000), RankTier::Champion);
        assert_eq!(RankTier::from_mmr(5000), RankTier::Unreal);
    }

    #[test]
    fn rank_division_display() {
        assert_eq!(RankDivision::from_mmr(100).display(), "Bronze III");
        assert_eq!(RankDivision::from_mmr(250).display(), "Bronze II");
        assert_eq!(RankDivision::from_mmr(400).display(), "Bronze I");
        assert_eq!(RankDivision::from_mmr(3500).display(), "Champion");
        assert_eq!(RankDivision::from_mmr(5500).display(), "Unreal");
    }

    #[test]
    fn placement_scoring() {
        assert_eq!(placement_points(1, 100), 120);
        assert_eq!(placement_points(2, 100), 85);
        assert_eq!(placement_points(10, 100), 40);
        assert_eq!(placement_points(25, 100), 10);
        assert!(placement_points(75, 100) < 0); // bus fee punishment
    }

    #[test]
    fn mmr_calculation_victory() {
        let profile = RankedProfile::new("did:test:1", "Player1", 1);
        let result = MatchResult {
            placement: 1,
            total_players: 100,
            kills: 8,
            assists: 3,
            damage_dealt: 1500,
            builds_placed: 50,
            survival_time_seconds: 1200.0,
        };
        let change = calculate_mmr_change(&profile, &result);
        assert_eq!(change.placement_points, 120);
        assert!(change.kill_points > 0);
        assert_eq!(change.bus_fee, 0); // Bronze has no bus fee
        assert!(change.total > 200);
    }

    #[test]
    fn mmr_progression_bronze_to_silver() {
        let mut profile = RankedProfile::new("did:test:1", "Player1", 1);
        assert_eq!(profile.rank.tier, RankTier::Bronze);

        // Win 5 matches with kills
        for _ in 0..5 {
            let result = MatchResult {
                placement: 1,
                total_players: 100,
                kills: 5,
                assists: 2,
                damage_dealt: 800,
                builds_placed: 30,
                survival_time_seconds: 900.0,
            };
            apply_match_result(&mut profile, &result);
        }

        assert!(
            profile.mmr >= 500,
            "MMR should be >= 500, got {}",
            profile.mmr
        );
        assert!(profile.rank.tier >= RankTier::Silver);
        assert_eq!(profile.wins, 5);
        assert_eq!(profile.win_rate, 100.0);
    }

    #[test]
    fn demotion_shield() {
        let mut profile = RankedProfile::new("did:test:1", "Player1", 1);
        profile.mmr = 500; // Silver floor
        profile.demotion_shield = true;
        profile.update_rank();

        let result = MatchResult {
            placement: 90,
            total_players: 100,
            kills: 0,
            assists: 0,
            damage_dealt: 50,
            builds_placed: 0,
            survival_time_seconds: 30.0,
        };
        let change = apply_match_result(&mut profile, &result);

        // Should not drop below 480 (Silver floor with protection)
        assert!(
            profile.mmr >= 480,
            "Shield should prevent demotion, got MMR={}",
            profile.mmr
        );
        assert!(!profile.demotion_shield); // consumed
    }

    #[test]
    fn high_rank_bus_fee() {
        let mut profile = RankedProfile::new("did:test:1", "Player1", 1);
        profile.mmr = 3500;
        profile.update_rank();
        assert_eq!(profile.rank.tier, RankTier::Champion);

        let result = MatchResult {
            placement: 50,
            total_players: 100,
            kills: 2,
            assists: 1,
            damage_dealt: 400,
            builds_placed: 10,
            survival_time_seconds: 600.0,
        };
        let change = calculate_mmr_change(&profile, &result);
        assert_eq!(change.bus_fee, -60); // Champion bus fee
    }

    #[test]
    fn matchmaking_groups() {
        let queue: Vec<MatchmakingEntry> = (0..200)
            .map(|i| MatchmakingEntry {
                player_did: format!("did:test:{}", i),
                mmr: i * 25,
                rank: RankTier::from_mmr(i * 25),
                queue_time: 5.0,
            })
            .collect();

        let lobbies = matchmake(&queue, 100, 500);
        assert!(!lobbies.is_empty());
        for lobby in &lobbies {
            assert!(lobby.len() >= 50); // at least half full
        }
    }

    #[test]
    fn season_soft_reset() {
        let season = Season {
            season_number: 2,
            name: "KAMI Season 2".into(),
            start_date: "2026-04-01".into(),
            end_date: "2026-06-30".into(),
            active: true,
            soft_reset_ratio: 0.5,
        };
        assert_eq!(season.soft_reset_mmr(3000), 1500);
        assert_eq!(season.soft_reset_mmr(0), 0);
    }

    #[test]
    fn kd_ratio_calculation() {
        let mut profile = RankedProfile::new("did:test:1", "Player1", 1);

        // 3 wins with kills
        for _ in 0..3 {
            apply_match_result(
                &mut profile,
                &MatchResult {
                    placement: 1,
                    total_players: 100,
                    kills: 5,
                    assists: 0,
                    damage_dealt: 500,
                    builds_placed: 0,
                    survival_time_seconds: 900.0,
                },
            );
        }
        // 2 losses
        for _ in 0..2 {
            apply_match_result(
                &mut profile,
                &MatchResult {
                    placement: 50,
                    total_players: 100,
                    kills: 2,
                    assists: 0,
                    damage_dealt: 200,
                    builds_placed: 0,
                    survival_time_seconds: 300.0,
                },
            );
        }

        // 15+4=19 kills, 2 deaths (5 games - 3 wins)
        assert_eq!(profile.total_kills, 19);
        assert!(profile.kd_ratio > 9.0); // 19/2 = 9.5
    }
}
