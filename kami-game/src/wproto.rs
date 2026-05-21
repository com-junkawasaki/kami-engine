//! W Protocol CQRS integration for KAMI games.
//!
//! Write path: Game events → WRecord() → AT Record → MDAG → yata sync
//! Read path:  G() (Sql) → game state queries
//!
//! This replaces direct DO SQLite for cross-island analytics while
//! keeping DO SQLite for operational per-island state.

use serde::{Deserialize, Serialize};

/// W Protocol record kinds for KAMI.
///
/// AT Lexicon mapping (dot notation → ai.gftd.apps.kami.*):
///   "kami.islandDef"     → ai.gftd.apps.kami.islandDef
///   "kami.character"     → ai.gftd.apps.kami.character
///   "kami.matchSummary"  → ai.gftd.apps.kami.matchSummary
///   etc.
pub mod kinds {
    // ── gftd:kami/island ──
    pub const ISLAND_DEF: &str = "kami.islandDef";
    pub const PORTAL: &str = "kami.portal";

    // ── gftd:kami/scene ──
    pub const SCENE_VERSION: &str = "kami.sceneVersion";

    // ── gftd:kami/character ──
    pub const CHARACTER: &str = "kami.character";

    // ── gftd:kami/publish ──
    pub const BUILD_RESULT: &str = "kami.buildResult";
    pub const PUBLISH_RESULT: &str = "kami.publishResult";

    // ── gftd:kami-battle-royale/match-lifecycle ──
    pub const MATCH_SUMMARY: &str = "kami.matchSummary";
    pub const PLAYER_RESULT: &str = "kami.playerResult";

    // ── gftd:kami-battle-royale/ranked-queue ──
    pub const RANKED_PROFILE: &str = "kami.rankedProfile";
    pub const SEASON_INFO: &str = "kami.seasonInfo";

    // ── gftd:kami-battle-royale/match-state ──
    pub const KILL_EVENT: &str = "kami.killEvent";

    // ── gftd:kami/catalog ──
    pub const LISTING: &str = "kami.listing";
    pub const COLLECTION: &str = "kami.collection";

    // ── gftd:kami/player ──
    pub const PLAYER_PROFILE: &str = "kami.playerProfile";
    pub const ACHIEVEMENT_DEF: &str = "kami.achievement";
    pub const ACHIEVEMENT_UNLOCK: &str = "kami.achievementUnlock";
    pub const PLAY_SESSION: &str = "kami.playSession";

    // ── gftd:kami/ranking ──
    pub const LEADERBOARD: &str = "kami.leaderboard";
    pub const LEADERBOARD_ENTRY: &str = "kami.leaderboardEntry";
    pub const SEASON_PASS: &str = "kami.seasonPass";

    // ── gftd:kami/economy ──
    pub const ITEM_DEF: &str = "kami.itemDef";
    pub const TRADE: &str = "kami.trade";

    // ── gftd:kami/emote ──
    pub const EMOTE_DEF: &str = "kami.emoteDef";
    pub const EMOTE_GRANT: &str = "kami.emoteGrant";
    pub const EMOTE_LOADOUT: &str = "kami.emoteLoadout";
    pub const EMOTE_PLAY: &str = "kami.game.emotePlay";

    // ── gftd:kami/physics ──
    pub const COLLISION_EVENT: &str = "kami.game.collision";

    // ── gftd:kami/trigger ──
    pub const TRIGGER_ZONE: &str = "kami.triggerZone";
    pub const TRIGGER_EVENT: &str = "kami.game.triggerEvent";

    // ── gftd:kami/npc ──
    pub const NPC_DEF: &str = "kami.npcDef";
    pub const NPC_INTERACTION: &str = "kami.game.npcInteraction";
    pub const QUEST_DEF: &str = "kami.questDef";
    pub const QUEST_PROGRESS: &str = "kami.questProgress";

    // ── gftd:kami/inventory ──
    pub const INVENTORY_EVENT: &str = "kami.game.inventoryEvent";

    // ── gftd:kami/terrain ──
    pub const TERRAIN_CONFIG: &str = "kami.terrainConfig";
    pub const TERRAIN_EDIT: &str = "kami.game.terrainEdit";

    // ── gftd:kami/pokoa ──
    pub const POKOA_TRAINER: &str = "kami.pokoaTrainer";
    pub const POKOA_BATTLE: &str = "kami.game.pokoaBattle";
    pub const POKOA_CAPTURE: &str = "kami.game.pokoaCapture";
    pub const POKOA_EVOLVE: &str = "kami.game.pokoaEvolve";

    // ── gftd:kami/gacha ──
    pub const GACHA_BANNER: &str = "kami.gachaBanner";
    pub const GACHA_ROLL_RESULT: &str = "kami.game.gachaRoll";

    // ── Game telemetry events ──
    pub const SCORE_SUBMIT: &str = "kami.game.score";
    pub const ITEM_PICKUP: &str = "kami.game.itemPickup";
    pub const ITEM_EQUIP: &str = "kami.game.itemEquip";
    pub const ECONOMY_TX: &str = "kami.game.transaction";
    pub const GACHA_ROLL: &str = "kami.game.gacha";
    pub const ACHIEVEMENT: &str = "kami.game.achievement";
    pub const SESSION_START: &str = "kami.game.sessionStart";
    pub const SESSION_END: &str = "kami.game.sessionEnd";
    pub const PORTAL_TRAVERSE: &str = "kami.game.portalTraverse";
    pub const PLAYER_KILL: &str = "kami.game.kill";
    pub const NPC_DIALOGUE: &str = "kami.game.dialogue";
}

/// W Protocol record payload for score submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreRecord {
    pub island_id: String,
    pub user_id: String,
    pub score: i64,
    pub game_slug: String,
    pub metadata: String,
}

/// W Protocol record for economy transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub user_id: String,
    pub amount: i64,
    pub currency: String,
    pub reason: String,
    pub island_id: String,
}

/// W Protocol record for session tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub user_id: String,
    pub island_id: String,
    pub duration_secs: u64,
    pub score: i64,
    pub items_collected: u32,
    pub kills: u32,
}

/// W Protocol record for portal traversal (cross-island analytics).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalRecord {
    pub user_id: String,
    pub from_island: String,
    pub to_island: String,
}

/// W Protocol record for emote definition (catalog).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmoteDefRecord {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub animation: String, // animation-preset enum value
    pub duration_ms: u32,
    pub looping: bool,
    pub particle: String, // particle-effect enum value
    pub sound_ref: Option<String>,
    pub color_tint: Option<String>,
    pub rarity: String, // item-rarity enum value
    pub game_id: Option<String>,
    pub tradeable: bool,
    pub preview_cid: Option<String>,
}

/// W Protocol record for emote grant (player inventory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmoteGrantRecord {
    pub user_id: String,
    pub emote_slug: String,
    pub source: String, // "purchase", "reward", "achievement", "trade", "default"
}

/// W Protocol record for emote loadout (quick-select wheel).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmoteLoadoutRecord {
    pub user_id: String,
    pub slots: Vec<Option<String>>,
    pub active_index: u32,
}

/// W Protocol record for emote play event (telemetry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotePlayRecord {
    pub user_id: String,
    pub emote_slug: String,
    pub trigger: String, // emote-trigger enum value
    pub island_id: String,
    pub position: [f32; 3],
}

/// W Protocol record for collision audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionRecord {
    pub entity_a: String,
    pub entity_b: String,
    pub kind: String,
    pub impulse: f32,
    pub island_id: String,
    pub position: [f32; 3],
}

/// W Protocol record for trigger zone event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEventRecord {
    pub zone_id: String,
    pub player_did: String,
    pub kind: String,
    pub data_json: String,
    pub island_id: String,
}

/// W Protocol record for NPC interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcInteractionRecord {
    pub npc_id: String,
    pub player_did: String,
    pub interaction_type: String,
    pub dialogue_node_id: Option<String>,
    pub choice_id: Option<String>,
    pub island_id: String,
}

/// W Protocol record for quest progress update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestProgressRecord {
    pub quest_id: String,
    pub player_did: String,
    pub status: String,
    pub objectives_progress_json: String,
}

/// W Protocol record for inventory change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryEventRecord {
    pub player_did: String,
    pub item_slug: String,
    pub action: String,
    pub quantity: u32,
    pub island_id: String,
    pub position: Option<[f32; 3]>,
}

/// W Protocol record for terrain edit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainEditRecord {
    pub island_id: String,
    pub position: [i32; 3],
    pub block_type: u16,
    pub player_did: String,
}

/// W Protocol record for Pokoa battle result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokoaBattleRecord {
    pub battle_id: String,
    pub battle_type: String,
    pub player_did: String,
    pub outcome: String,
    pub turns: u32,
    pub player_species: String,
    pub opponent_species: String,
    pub island_id: String,
}

/// W Protocol record for Pokoa capture event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokoaCaptureRecord {
    pub player_did: String,
    pub species_id: u16,
    pub species_name: String,
    pub level: u8,
    pub ball_type: String,
    pub island_id: String,
}

/// W Protocol record for gacha roll result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GachaRollRecord {
    pub banner_id: String,
    pub player_did: String,
    pub result_slug: String,
    pub rarity: String,
    pub pity_count: u32,
    pub is_rate_up: bool,
}

/// Sql graph queries for cross-island analytics (read path).
pub mod queries {
    /// Top scores across all islands.
    pub const GLOBAL_LEADERBOARD: &str =
        "MATCH (s:Score) RETURN s.user_id, s.score, s.island_id ORDER BY s.score DESC LIMIT $limit";

    /// Player's cross-island stats.
    pub const PLAYER_STATS: &str = "MATCH (p:Player {user_id: $user_id})-[:PLAYED]->(i:Island) \
         RETURN i.name, p.total_score, p.sessions, p.play_time_secs";

    /// Most popular islands by session count.
    pub const POPULAR_ISLANDS: &str = "MATCH (s:Session)-[:ON]->(i:Island) \
         RETURN i.island_id, i.name, COUNT(s) as sessions \
         ORDER BY sessions DESC LIMIT $limit";

    /// Island-to-island portal flow (which portals are most used).
    pub const PORTAL_FLOW: &str = "MATCH (p:Portal)-[:FROM]->(a:Island), (p)-[:TO]->(b:Island) \
         RETURN a.name, b.name, COUNT(p) as traversals \
         ORDER BY traversals DESC LIMIT $limit";

    /// Player gem balance across all games.
    pub const PLAYER_GEMS: &str = "MATCH (w:Wallet {user_id: $user_id, currency: 'gems'}) \
         RETURN w.balance";

    /// Most used emotes across all islands.
    pub const POPULAR_EMOTES: &str = "MATCH (e:EmotePlay) \
         RETURN e.emote_slug, COUNT(e) as plays \
         ORDER BY plays DESC LIMIT $limit";

    /// Player's emote inventory.
    pub const PLAYER_EMOTES: &str = "MATCH (g:EmoteGrant {user_id: $user_id}) \
         RETURN g.emote_slug, g.source";

    /// Emote usage by trigger type (analytics).
    pub const EMOTE_TRIGGER_STATS: &str = "MATCH (e:EmotePlay {emote_slug: $emote_slug}) \
         RETURN e.trigger, COUNT(e) as count \
         ORDER BY count DESC";

    /// Emote usage heatmap by island.
    pub const EMOTE_ISLAND_HEATMAP: &str = "MATCH (e:EmotePlay) \
         RETURN e.island_id, e.emote_slug, COUNT(e) as plays \
         ORDER BY plays DESC LIMIT $limit";

    /// NPC interaction frequency.
    pub const NPC_INTERACTION_STATS: &str = "MATCH (n:NpcInteraction) \
         RETURN n.npc_id, n.interaction_type, COUNT(n) as count \
         ORDER BY count DESC LIMIT $limit";

    /// Quest completion rates.
    pub const QUEST_COMPLETION_RATE: &str = "MATCH (q:QuestProgress {status: 'completed'}) \
         RETURN q.quest_id, COUNT(q) as completions \
         ORDER BY completions DESC LIMIT $limit";

    /// Pokoa species popularity (captures).
    pub const POKOA_CAPTURE_STATS: &str = "MATCH (c:PokoaCapture) \
         RETURN c.species_name, COUNT(c) as captures \
         ORDER BY captures DESC LIMIT $limit";

    /// Pokoa battle win rates by species.
    pub const POKOA_WIN_RATES: &str = "MATCH (b:PokoaBattle {outcome: 'player_win'}) \
         RETURN b.player_species, COUNT(b) as wins \
         ORDER BY wins DESC LIMIT $limit";

    /// Gacha roll rarity distribution.
    pub const GACHA_RARITY_DIST: &str = "MATCH (g:GachaRoll {banner_id: $banner_id}) \
         RETURN g.rarity, COUNT(g) as count \
         ORDER BY count DESC";

    /// Terrain edit hotspots (building activity).
    pub const TERRAIN_EDIT_HOTSPOTS: &str = "MATCH (t:TerrainEdit) \
         RETURN t.island_id, COUNT(t) as edits \
         ORDER BY edits DESC LIMIT $limit";

    /// Trigger zone fire frequency.
    pub const TRIGGER_FIRE_STATS: &str = "MATCH (t:TriggerEvent) \
         RETURN t.kind, t.island_id, COUNT(t) as fires \
         ORDER BY fires DESC LIMIT $limit";

    /// Inventory item popularity (pickups).
    pub const INVENTORY_POPULAR_ITEMS: &str = "MATCH (i:InventoryEvent {action: 'pickup'}) \
         RETURN i.item_slug, COUNT(i) as pickups \
         ORDER BY pickups DESC LIMIT $limit";
}

/// Generate the magatama-go command patterns for W Protocol CQRS.
/// This is the TinyGo code template that each KAMI game island uses.
pub fn cqrs_command_template(game_slug: &str) -> String {
    format!(
        r#"
// W Protocol CQRS commands for {game_slug}
// Write path: WRecord() → AT Record → MDAG → yata auto sync
// Read path: Q() (DO SQLite operational) + G() (Sql analytics)

// ── Write: Score submission (WRecord) ──
func cmdSubmitScore(ctx *magatama.AppContext, body []byte) ([]byte, error) {{
    var args struct {{
        Score    int64  `json:"score"`
        Metadata string `json:"metadata"`
    }}
    json.Unmarshal(body, &args)

    // Operational write: DO SQLite (per-island leaderboard)
    magatama.Q("scores").Insert(magatama.Row{{
        "game_id": "{game_slug}", "user_id": ctx.UserID, "org_id": ctx.OrgID,
        "actor_id": ctx.ActorID, "score": args.Score, "metadata": args.Metadata,
        "created_at": nowISO(),
    }})

    // Analytics write: WRecord → AT Record → MDAG → yata Sql sync
    payload, _ := json.Marshal(map[string]any{{
        "island_id": "{game_slug}", "user_id": ctx.UserID,
        "score": args.Score, "game_slug": "{game_slug}", "metadata": args.Metadata,
    }})
    magatama.WRecord("kami.game.score", payload)

    return json.Marshal(map[string]any{{"ok": true, "score": args.Score}})
}}

// ── Read: Leaderboard (Q for local, G for global) ──
func cmdGetRankings(ctx *magatama.AppContext, body []byte) ([]byte, error) {{
    // Local (this island): DO SQLite
    local, _ := magatama.Q("scores").
        Where(magatama.Eq{{"game_id": "{game_slug}", "org_id": ctx.OrgID}}).
        OrderBy("score DESC").Limit(50).Query()

    // Global (all islands): Sql graph
    global, _ := magatama.G("Score").
        Match(magatama.Eq{{"game_slug": "{game_slug}"}}).
        Return("user_id", "score", "island_id").
        OrderBy("score DESC").Limit(50).Query()

    return json.Marshal(map[string]any{{"local": json.RawMessage(local), "global": global}})
}}

// ── Write: Economy transaction (WRecord) ──
func cmdPurchase(ctx *magatama.AppContext, body []byte) ([]byte, error) {{
    var args struct {{
        Amount int64  `json:"amount"`
        Reason string `json:"reason"`
    }}
    json.Unmarshal(body, &args)

    // DO SQLite: debit wallet
    magatama.Q("wallets").
        Where(magatama.Eq{{"user_id": ctx.UserID, "org_id": ctx.OrgID, "currency": "gems"}}).
        Update(magatama.Row{{"balance": magatama.Raw("balance - ?", args.Amount)}})

    // WRecord: analytics
    payload, _ := json.Marshal(map[string]any{{
        "user_id": ctx.UserID, "amount": -args.Amount,
        "currency": "gems", "reason": args.Reason, "island_id": "{game_slug}",
    }})
    magatama.WRecord("kami.game.transaction", payload)

    return json.Marshal(map[string]any{{"ok": true}})
}}
"#,
        game_slug = game_slug
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_record_serializable() {
        let r = ScoreRecord {
            island_id: "agar".into(),
            user_id: "u1".into(),
            score: 1500,
            game_slug: "agar".into(),
            metadata: "{}".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("1500"));
    }

    #[test]
    fn emote_def_record_serializable() {
        let r = EmoteDefRecord {
            slug: "skibidi-spin".into(),
            name: "Skibidi Spin".into(),
            description: "Dop dop yes yes spinning emote".into(),
            animation: "spinning".into(),
            duration_ms: 2000,
            looping: true,
            particle: "bubbles".into(),
            sound_ref: Some("sfx-skibidi-dop".into()),
            color_tint: None,
            rarity: "rare".into(),
            game_id: None,
            tradeable: true,
            preview_cid: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("skibidi-spin"));
        assert!(json.contains("spinning"));
        assert!(json.contains("2000"));
    }

    #[test]
    fn emote_play_record_serializable() {
        let r = EmotePlayRecord {
            user_id: "u1".into(),
            emote_slug: "sigma-stare".into(),
            trigger: "manual".into(),
            island_id: "urn:kami:island:sigma".into(),
            position: [10.0, 0.5, -5.0],
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("sigma-stare"));
        assert!(json.contains("manual"));
    }

    #[test]
    fn emote_loadout_record_serializable() {
        let r = EmoteLoadoutRecord {
            user_id: "u1".into(),
            slots: vec![
                Some("skibidi-spin".into()),
                Some("sigma-stare".into()),
                None,
                Some("grimace-wobble".into()),
                None,
                None,
                None,
                None,
            ],
            active_index: 0,
        };
        let json = serde_json::to_string(&r).unwrap();
        let parsed: EmoteLoadoutRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.slots.len(), 8);
        assert_eq!(parsed.slots[0], Some("skibidi-spin".into()));
        assert_eq!(parsed.slots[2], None);
    }

    #[test]
    fn collision_record_serializable() {
        let r = CollisionRecord {
            entity_a: "player-1".into(),
            entity_b: "wall-n".into(),
            kind: "enter".into(),
            impulse: 5.2,
            island_id: "agar".into(),
            position: [10.0, 0.5, -3.0],
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("player-1"));
    }

    #[test]
    fn npc_interaction_record_serializable() {
        let r = NpcInteractionRecord {
            npc_id: "guard-1".into(),
            player_did: "u1".into(),
            interaction_type: "dialogue".into(),
            dialogue_node_id: Some("node-1".into()),
            choice_id: Some("choice-a".into()),
            island_id: "dungeon".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("guard-1"));
    }

    #[test]
    fn pokoa_battle_record_serializable() {
        let r = PokoaBattleRecord {
            battle_id: "b-001".into(),
            battle_type: "wild".into(),
            player_did: "u1".into(),
            outcome: "player_win".into(),
            turns: 5,
            player_species: "Sigmachu".into(),
            opponent_species: "Toilettle".into(),
            island_id: "pokoa".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("Sigmachu"));
    }

    #[test]
    fn gacha_roll_record_serializable() {
        let r = GachaRollRecord {
            banner_id: "brainrot-banner-1".into(),
            player_did: "u1".into(),
            result_slug: "skibidi-skin-gold".into(),
            rarity: "legendary".into(),
            pity_count: 78,
            is_rate_up: true,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("legendary"));
        assert!(json.contains("78"));
    }

    #[test]
    fn terrain_edit_record_serializable() {
        let r = TerrainEditRecord {
            island_id: "minecraft-1".into(),
            position: [10, 5, -3],
            block_type: 4,
            player_did: "u1".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("minecraft-1"));
    }

    #[test]
    fn new_kind_constants_valid() {
        assert_eq!(kinds::COLLISION_EVENT, "kami.game.collision");
        assert_eq!(kinds::TRIGGER_ZONE, "kami.triggerZone");
        assert_eq!(kinds::TRIGGER_EVENT, "kami.game.triggerEvent");
        assert_eq!(kinds::NPC_DEF, "kami.npcDef");
        assert_eq!(kinds::NPC_INTERACTION, "kami.game.npcInteraction");
        assert_eq!(kinds::QUEST_DEF, "kami.questDef");
        assert_eq!(kinds::QUEST_PROGRESS, "kami.questProgress");
        assert_eq!(kinds::INVENTORY_EVENT, "kami.game.inventoryEvent");
        assert_eq!(kinds::TERRAIN_CONFIG, "kami.terrainConfig");
        assert_eq!(kinds::TERRAIN_EDIT, "kami.game.terrainEdit");
        assert_eq!(kinds::POKOA_TRAINER, "kami.pokoaTrainer");
        assert_eq!(kinds::POKOA_BATTLE, "kami.game.pokoaBattle");
        assert_eq!(kinds::POKOA_CAPTURE, "kami.game.pokoaCapture");
        assert_eq!(kinds::POKOA_EVOLVE, "kami.game.pokoaEvolve");
        assert_eq!(kinds::GACHA_BANNER, "kami.gachaBanner");
        assert_eq!(kinds::GACHA_ROLL_RESULT, "kami.game.gachaRoll");
    }

    #[test]
    fn emote_kind_constants() {
        assert_eq!(kinds::EMOTE_DEF, "kami.emoteDef");
        assert_eq!(kinds::EMOTE_GRANT, "kami.emoteGrant");
        assert_eq!(kinds::EMOTE_LOADOUT, "kami.emoteLoadout");
        assert_eq!(kinds::EMOTE_PLAY, "kami.game.emotePlay");
    }

    #[test]
    fn cqrs_template_generates() {
        let code = cqrs_command_template("snake");
        assert!(code.contains("snake"));
        assert!(code.contains("WRecord"));
        assert!(code.contains("magatama.Q"));
        assert!(code.contains("magatama.G"));
    }
}
