//! GameShell: KAMI-specific game UI layer.
//!
//! HUD overlay rendered on top of WebGPU canvas:
//!   - HP bar, gems counter, ammo
//!   - Minimap (top-right)
//!   - Chat panel (bottom-left)
//!   - Portal indicator
//!   - Scoreboard (Tab)
//!   - Inventory grid (I)
//!
//! GameShell is data-only — rendering is done by kami-render (wgpu) or Svelte overlay (web).

use serde::{Deserialize, Serialize};

/// Complete HUD state, serialized per frame for overlay rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudState {
    pub hp: HpBar,
    pub gems: u64,
    pub ammo: u32,
    pub minimap: MinimapState,
    pub chat: Vec<ChatLine>,
    pub portal_indicator: Option<PortalIndicator>,
    pub scoreboard: Vec<ScoreRow>,
    pub inventory_open: bool,
    pub inventory_slots: Vec<InventorySlotView>,
    pub fps: u32,
    pub ping_ms: u32,
    pub player_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpBar {
    pub current: u16,
    pub max: u16,
}

impl HpBar {
    pub fn ratio(&self) -> f32 {
        if self.max == 0 {
            return 0.0;
        }
        self.current as f32 / self.max as f32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimapState {
    pub player_x: f32,
    pub player_z: f32,
    pub entities: Vec<MinimapEntity>,
    pub map_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimapEntity {
    pub x: f32,
    pub z: f32,
    pub kind: MinimapEntityKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MinimapEntityKind {
    Player,
    OtherPlayer,
    Npc,
    Item,
    Portal,
    Enemy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatLine {
    pub sender: String,
    pub content: String,
    pub tick: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalIndicator {
    pub island_name: String,
    pub distance: f32,
    pub direction_angle: f32, // radians from camera forward
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreRow {
    pub name: String,
    pub kills: u32,
    pub deaths: u32,
    pub gems: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlotView {
    pub item_name: String,
    pub quantity: u32,
    pub equipped: bool,
    pub rarity: String,
}

impl HudState {
    /// Create empty HUD.
    pub fn new() -> Self {
        Self {
            hp: HpBar {
                current: 100,
                max: 100,
            },
            gems: 0,
            ammo: 0,
            minimap: MinimapState {
                player_x: 0.0,
                player_z: 0.0,
                entities: Vec::new(),
                map_size: 60.0,
            },
            chat: Vec::new(),
            portal_indicator: None,
            scoreboard: Vec::new(),
            inventory_open: false,
            inventory_slots: Vec::new(),
            fps: 60,
            ping_ms: 0,
            player_count: 1,
        }
    }

    /// Add chat message. Keep last 20.
    pub fn push_chat(&mut self, sender: &str, content: &str, tick: u32) {
        self.chat.push(ChatLine {
            sender: sender.into(),
            content: content.into(),
            tick,
        });
        if self.chat.len() > 20 {
            self.chat.remove(0);
        }
    }

    /// Serialize to JSON for Svelte overlay (web) or wgpu text rendering (native).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl Default for HudState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_json_roundtrip() {
        let mut hud = HudState::new();
        hud.gems = 42;
        hud.push_chat("Alice", "hello", 100);
        let json = hud.to_json();
        let parsed: HudState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.gems, 42);
        assert_eq!(parsed.chat.len(), 1);
    }

    #[test]
    fn hp_bar_ratio() {
        let hp = HpBar {
            current: 75,
            max: 100,
        };
        assert!((hp.ratio() - 0.75).abs() < 0.001);
    }
}
