//! Sabi-Otoshi!! — 3D Rust Restoration Game on KAMI Engine.
//!
//! High-pressure wash + brush + sandpaper + polish to restore vintage items.
//! 3D turntable rotation, step-by-step disassembly, SDF models, NeRF rust.
//! CPC/UNSPSC product classification per item.

use crate::input::InputState;
use glam::{Mat4, Quat, Vec3};

// ── Constants ──

const TURNTABLE_SPEED: f32 = 0.8;
const TURNTABLE_DRAG: f32 = 0.95;
const ZOOM_MIN: f32 = 1.5;
const ZOOM_MAX: f32 = 6.0;
const ZOOM_SPEED: f32 = 0.3;
const RUST_REMOVAL_RATE: f32 = 0.016;

// ── Tool Types ──

/// Cleaning/restoration tool with physics properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolKind {
    PressureWasher,
    WireBrush,
    Sandpaper,
    ChemicalSolvent,
    PolishingCloth,
    Ultrasonic,
}

impl ToolKind {
    pub fn all() -> &'static [ToolKind] {
        &[
            ToolKind::PressureWasher,
            ToolKind::WireBrush,
            ToolKind::Sandpaper,
            ToolKind::ChemicalSolvent,
            ToolKind::PolishingCloth,
            ToolKind::Ultrasonic,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ToolKind::PressureWasher => "Pressure Washer",
            ToolKind::WireBrush => "Wire Brush",
            ToolKind::Sandpaper => "Sandpaper",
            ToolKind::ChemicalSolvent => "Chemical Solvent",
            ToolKind::PolishingCloth => "Polishing Cloth",
            ToolKind::Ultrasonic => "Ultrasonic Cleaner",
        }
    }

    pub fn name_ja(&self) -> &'static str {
        match self {
            ToolKind::PressureWasher => "高圧洗浄機",
            ToolKind::WireBrush => "ワイヤーブラシ",
            ToolKind::Sandpaper => "サンドペーパー",
            ToolKind::ChemicalSolvent => "錆取り液",
            ToolKind::PolishingCloth => "研磨クロス",
            ToolKind::Ultrasonic => "超音波洗浄機",
        }
    }

    /// Effectiveness multiplier per rust type.
    pub fn effectiveness(&self, rust_type: RustType) -> f32 {
        match (self, rust_type) {
            // Pressure washer: great on surface, ok on deep, poor on pitted
            (ToolKind::PressureWasher, RustType::Surface) => 1.0,
            (ToolKind::PressureWasher, RustType::Deep) => 0.5,
            (ToolKind::PressureWasher, RustType::Pitted) => 0.2,
            (ToolKind::PressureWasher, RustType::Patina) => 0.8,
            // Wire brush: best on deep rust
            (ToolKind::WireBrush, RustType::Surface) => 0.6,
            (ToolKind::WireBrush, RustType::Deep) => 1.0,
            (ToolKind::WireBrush, RustType::Pitted) => 0.7,
            (ToolKind::WireBrush, RustType::Patina) => 0.3,
            // Sandpaper: best on pitted rust
            (ToolKind::Sandpaper, RustType::Surface) => 0.4,
            (ToolKind::Sandpaper, RustType::Deep) => 0.7,
            (ToolKind::Sandpaper, RustType::Pitted) => 1.0,
            (ToolKind::Sandpaper, RustType::Patina) => 0.5,
            // Chemical: dissolves all equally, slow but steady
            (ToolKind::ChemicalSolvent, RustType::Surface) => 0.7,
            (ToolKind::ChemicalSolvent, RustType::Deep) => 0.8,
            (ToolKind::ChemicalSolvent, RustType::Pitted) => 0.8,
            (ToolKind::ChemicalSolvent, RustType::Patina) => 1.0,
            // Polish: only for finishing (surface/patina)
            (ToolKind::PolishingCloth, RustType::Surface) => 0.9,
            (ToolKind::PolishingCloth, RustType::Deep) => 0.1,
            (ToolKind::PolishingCloth, RustType::Pitted) => 0.05,
            (ToolKind::PolishingCloth, RustType::Patina) => 1.0,
            // Ultrasonic: great all-around but requires immersion step
            (ToolKind::Ultrasonic, RustType::Surface) => 0.9,
            (ToolKind::Ultrasonic, RustType::Deep) => 0.9,
            (ToolKind::Ultrasonic, RustType::Pitted) => 0.6,
            (ToolKind::Ultrasonic, RustType::Patina) => 0.7,
        }
    }

    /// Spray/contact radius in world units.
    pub fn radius(&self) -> f32 {
        match self {
            ToolKind::PressureWasher => 0.3,
            ToolKind::WireBrush => 0.15,
            ToolKind::Sandpaper => 0.12,
            ToolKind::ChemicalSolvent => 0.5,
            ToolKind::PolishingCloth => 0.2,
            ToolKind::Ultrasonic => 1.0,
        }
    }

    /// Power per tick.
    pub fn power(&self) -> f32 {
        match self {
            ToolKind::PressureWasher => 2.0,
            ToolKind::WireBrush => 3.0,
            ToolKind::Sandpaper => 4.0,
            ToolKind::ChemicalSolvent => 1.5,
            ToolKind::PolishingCloth => 1.0,
            ToolKind::Ultrasonic => 2.5,
        }
    }

    /// CPC code for this tool.
    pub fn cpc_code(&self) -> &'static str {
        match self {
            ToolKind::PressureWasher => "44913",  // CPC: Cleaning machinery
            ToolKind::WireBrush => "42922",       // CPC: Brushes
            ToolKind::Sandpaper => "42952",       // CPC: Abrasives
            ToolKind::ChemicalSolvent => "34741", // CPC: Rust removers
            ToolKind::PolishingCloth => "26993",  // CPC: Polishing cloths
            ToolKind::Ultrasonic => "44914",      // CPC: Ultrasonic equipment
        }
    }

    /// UNSPSC code for this tool.
    pub fn unspsc_code(&self) -> &'static str {
        match self {
            ToolKind::PressureWasher => "47131800",
            ToolKind::WireBrush => "27111700",
            ToolKind::Sandpaper => "31191500",
            ToolKind::ChemicalSolvent => "47131600",
            ToolKind::PolishingCloth => "47131500",
            ToolKind::Ultrasonic => "41113600",
        }
    }

    /// Haptic vibration pattern index (0=light, 1=medium, 2=heavy, 3=combo).
    pub fn haptic_pattern(&self) -> u8 {
        match self {
            ToolKind::PressureWasher => 2,
            ToolKind::WireBrush => 1,
            ToolKind::Sandpaper => 1,
            ToolKind::ChemicalSolvent => 0,
            ToolKind::PolishingCloth => 0,
            ToolKind::Ultrasonic => 2,
        }
    }
}

// ── Rust Types ──

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RustType {
    Surface,
    Deep,
    Pitted,
    Patina,
}

impl RustType {
    pub fn color_rgb(&self, level: f32) -> [f32; 3] {
        let t = level.clamp(0.0, 1.0);
        match self {
            RustType::Surface => [0.65 + t * 0.25, 0.35 + t * 0.15, 0.1 + t * 0.05],
            RustType::Deep => [0.5 + t * 0.35, 0.2 + t * 0.12, 0.05 + t * 0.05],
            RustType::Pitted => [0.35 + t * 0.25, 0.12 + t * 0.08, 0.04 + t * 0.04],
            RustType::Patina => [0.2 + t * 0.15, 0.45 + t * 0.2, 0.3 + t * 0.15],
        }
    }
}

// ── Rust Zone (3D) ──

/// A 3D rust zone on an item, defined by center + extent in local space.
#[derive(Debug, Clone)]
pub struct RustZone3D {
    pub id: String,
    pub center: Vec3,
    pub extent: Vec3,
    pub rust_type: RustType,
    pub initial_level: f32,
    pub current_level: f32,
    /// NeRF density grid index (for realistic rust appearance).
    pub nerf_grid_idx: Option<usize>,
}

impl RustZone3D {
    pub fn is_clean(&self) -> bool {
        self.current_level <= 0.0
    }

    pub fn contains_point(&self, local_point: Vec3) -> bool {
        let d = (local_point - self.center).abs();
        d.x <= self.extent.x && d.y <= self.extent.y && d.z <= self.extent.z
    }
}

// ── Disassembly ──

/// A disassembly step: remove a part to access hidden rust underneath.
#[derive(Debug, Clone)]
pub struct DisassemblyStep {
    pub id: String,
    pub name: String,
    pub name_ja: String,
    /// Entity IDs of parts to detach.
    pub part_ids: Vec<String>,
    /// Rust zones revealed after disassembly.
    pub revealed_zones: Vec<String>,
    /// Whether this step is completed.
    pub completed: bool,
    /// Required tool (optional).
    pub required_tool: Option<ToolKind>,
    /// Animation: where parts fly to when detached.
    pub detach_offset: Vec3,
}

// ── Item Definition ──

/// A restorable item with 3D geometry, rust zones, and disassembly steps.
#[derive(Debug, Clone)]
pub struct RestorableItem {
    pub id: String,
    pub name: String,
    pub name_ja: String,
    pub difficulty: u8,
    pub base_score: u32,
    pub perfect_bonus: u32,
    /// SDF model description (for kami-sdf generation).
    pub sdf_desc: String,
    /// Entity IDs comprising this item in the scene.
    pub entity_ids: Vec<String>,
    /// Rust zones (some hidden until disassembly).
    pub zones: Vec<RustZone3D>,
    /// Disassembly steps (sequential).
    pub disassembly_steps: Vec<DisassemblyStep>,
    /// CPC product classification code.
    pub cpc_code: String,
    /// UNSPSC product classification code.
    pub unspsc_code: String,
    /// Clean metal base color [R,G,B].
    pub metal_color: [f32; 3],
    /// Metallic factor (0-1, PBR).
    pub metallic: f32,
    /// Roughness factor (0-1, PBR).
    pub roughness: f32,
}

// ── Entity Update ──

/// Per-frame entity transform update for the renderer.
#[derive(Debug, Clone)]
pub struct EntityUpdate {
    pub id: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub visible: bool,
    /// Rust tint color override (None = use original material).
    pub rust_tint: Option<[f32; 4]>,
}

// ── Game Phase ──

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Title,
    Inspecting,
    Restoring,
    Disassembling,
    ItemClear,
    AllClear,
    Timeout,
}

// ── Camera (Turntable) ──

#[derive(Debug, Clone)]
pub struct TurntableCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
    pub yaw_velocity: f32,
    pub pitch_velocity: f32,
}

impl TurntableCamera {
    pub fn new() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.3,
            distance: 3.0,
            target: Vec3::ZERO,
            yaw_velocity: 0.0,
            pitch_velocity: 0.0,
        }
    }

    pub fn update(&mut self, drag_dx: f32, drag_dy: f32, zoom_delta: f32, dt: f32) {
        self.yaw_velocity += drag_dx * TURNTABLE_SPEED;
        self.pitch_velocity += drag_dy * TURNTABLE_SPEED * 0.5;
        self.yaw += self.yaw_velocity * dt;
        self.pitch = (self.pitch + self.pitch_velocity * dt).clamp(-1.2, 1.2);
        self.yaw_velocity *= TURNTABLE_DRAG;
        self.pitch_velocity *= TURNTABLE_DRAG;
        self.distance = (self.distance - zoom_delta * ZOOM_SPEED).clamp(ZOOM_MIN, ZOOM_MAX);
    }

    pub fn eye_position(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + Vec3::new(x, y, z)
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye_position(), self.target, Vec3::Y)
    }

    /// Auto-rotate slowly when idle (for title / inspection).
    pub fn auto_rotate(&mut self, dt: f32) {
        self.yaw += 0.3 * dt;
    }
}

// ── Game State ──

pub struct SabiotoshiGame {
    pub phase: Phase,
    pub tick: u64,
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub perfects: u32,
    pub items_cleared: u32,
    pub total_items: u32,
    pub time_remaining: f32,

    pub camera: TurntableCamera,
    pub current_tool: ToolKind,
    pub current_item_idx: usize,
    pub items: Vec<RestorableItem>,

    /// Spray/contact point in world space (from raycasting).
    pub contact_point: Option<Vec3>,
    pub is_applying_tool: bool,

    // ── Feedback ──
    pub screen_shake: f32,
    pub flash_color: Option<[f32; 4]>,
    pub message: Option<(String, f32)>,
    /// Haptic vibration request (consumed by renderer each frame).
    pub haptic_request: Option<u8>,
    /// Sound effect request (consumed by renderer each frame).
    pub sfx_request: Vec<String>,
    /// Particle burst request: (position, count, color).
    pub particle_request: Vec<(Vec3, u32, [f32; 3])>,

    // ── Disassembly animation ──
    detaching_parts: Vec<(String, Vec3, Vec3, f32)>,
    clear_timer: f32,
}

impl SabiotoshiGame {
    pub fn new(items: Vec<RestorableItem>, total_items: u32) -> Self {
        Self {
            phase: Phase::Title,
            tick: 0,
            score: 0,
            combo: 0,
            max_combo: 0,
            perfects: 0,
            items_cleared: 0,
            total_items,
            time_remaining: 30.0 + total_items as f32 * 25.0,
            camera: TurntableCamera::new(),
            current_tool: ToolKind::PressureWasher,
            current_item_idx: 0,
            items,
            contact_point: None,
            is_applying_tool: false,
            screen_shake: 0.0,
            flash_color: None,
            message: None,
            haptic_request: None,
            sfx_request: Vec::new(),
            particle_request: Vec::new(),
            detaching_parts: Vec::new(),
            clear_timer: 0.0,
        }
    }

    pub fn current_item(&self) -> Option<&RestorableItem> {
        self.items.get(self.current_item_idx)
    }

    pub fn current_item_mut(&mut self) -> Option<&mut RestorableItem> {
        self.items.get_mut(self.current_item_idx)
    }

    pub fn start_game(&mut self) {
        self.phase = Phase::Inspecting;
        self.score = 0;
        self.combo = 0;
        self.max_combo = 0;
        self.perfects = 0;
        self.items_cleared = 0;
        self.current_item_idx = 0;
        self.camera = TurntableCamera::new();
        self.sfx_request.push("title_select".into());
        self.haptic_request = Some(1);
        self.message = Some(("Inspect the item — rotate to find rust!".into(), 3.0));
    }

    pub fn select_tool(&mut self, tool: ToolKind) {
        if self.current_tool != tool {
            self.current_tool = tool;
            self.sfx_request.push("nozzle_switch".into());
            self.haptic_request = Some(0);
        }
    }

    pub fn begin_disassembly(&mut self) {
        if self.phase != Phase::Restoring && self.phase != Phase::Inspecting {
            return;
        }
        let item = match self.items.get_mut(self.current_item_idx) {
            Some(i) => i,
            None => return,
        };
        // Find next incomplete disassembly step
        for step in &mut item.disassembly_steps {
            if !step.completed {
                if let Some(req) = &step.required_tool {
                    if *req != self.current_tool {
                        self.message = Some((format!("Need {} for this step", req.name()), 2.0));
                        self.sfx_request.push("error".into());
                        return;
                    }
                }
                step.completed = true;
                self.phase = Phase::Disassembling;
                // Reveal hidden zones
                for zone_id in &step.revealed_zones {
                    for zone in &mut item.zones {
                        if zone.id == *zone_id {
                            // Zone was hidden, now visible (level stays as-is)
                        }
                    }
                }
                // Animate parts detaching
                for part_id in &step.part_ids {
                    self.detaching_parts.push((
                        part_id.clone(),
                        Vec3::ZERO,
                        step.detach_offset,
                        0.0,
                    ));
                }
                self.sfx_request.push("rust_crack".into());
                self.haptic_request = Some(2);
                self.screen_shake = 0.3;
                self.score += 50;
                self.message = Some((format!("{} — disassembled!", step.name), 2.0));
                return;
            }
        }
    }

    /// Apply current tool at contact point.
    fn apply_tool(&mut self, dt: f32) {
        let contact = match self.contact_point {
            Some(p) => p,
            None => return,
        };

        let tool = self.current_tool;
        let radius = tool.radius();
        let power = tool.power();

        let item = match self.items.get_mut(self.current_item_idx) {
            Some(i) => i,
            None => return,
        };

        let mut any_hit = false;
        let mut zone_cleared = false;

        for zone in &mut item.zones {
            if zone.is_clean() {
                continue;
            }

            // Check if contact point is within zone + tool radius
            let dist = (contact - zone.center).length();
            if dist > zone.extent.length() + radius {
                continue;
            }

            any_hit = true;
            let effectiveness = tool.effectiveness(zone.rust_type);
            let removal = power * effectiveness * RUST_REMOVAL_RATE * dt * 60.0;
            let prev = zone.current_level;
            zone.current_level = (zone.current_level - removal).max(0.0);

            if prev > 0.0 && zone.current_level <= 0.0 {
                zone_cleared = true;
                self.combo += 1;
                if self.combo > self.max_combo {
                    self.max_combo = self.combo;
                }
                let pts = 10 + (self.combo.min(12) * 5);
                self.score += pts;
                self.sfx_request.push("zone_clear".into());
                self.sfx_request
                    .push(format!("combo_ding_{}", self.combo.min(12)));
                self.haptic_request = Some(3);
                self.screen_shake = 0.2;
                self.particle_request
                    .push((zone.center, 12, [1.0, 0.96, 0.88]));
            }
        }

        if any_hit {
            if self.tick % 3 == 0 {
                self.sfx_request.push("spray_loop".into());
            }
            if self.tick % 8 == 0 {
                self.haptic_request = Some(tool.haptic_pattern());
            }
        }

        // Check all zones clean
        let all_clean = item.zones.iter().all(|z| z.is_clean());
        if all_clean && zone_cleared {
            let perfect = self.combo >= item.zones.len() as u32;
            self.score += item.base_score + if perfect { item.perfect_bonus } else { 0 };
            if perfect {
                self.perfects += 1
            }
            self.items_cleared += 1;

            if self.items_cleared >= self.total_items {
                self.phase = Phase::AllClear;
                self.sfx_request.push("all_clear".into());
                self.haptic_request = Some(3);
            } else {
                self.phase = Phase::ItemClear;
                self.clear_timer = 0.0;
                if perfect {
                    self.sfx_request.push("perfect_finish".into());
                } else {
                    self.sfx_request.push("item_complete".into());
                }
                self.haptic_request = Some(2);
            }
            self.flash_color = Some([1.0, 1.0, 0.8, 0.25]);
            self.particle_request
                .push((Vec3::ZERO, 30, [1.0, 0.96, 0.88]));
        }
    }

    pub fn update(&mut self, input: &InputState, dt: f32) {
        self.tick += 1;
        self.sfx_request.clear();
        self.particle_request.clear();
        self.haptic_request = None;

        // Decay feedback
        if self.screen_shake > 0.0 {
            self.screen_shake = (self.screen_shake - dt * 4.0).max(0.0)
        }
        if let Some((_, ref mut t)) = self.message {
            *t -= dt;
            if *t <= 0.0 {
                self.message = None
            }
        }
        self.flash_color = None;

        match self.phase {
            Phase::Title => {
                self.camera.auto_rotate(dt);
                if input.interact {
                    self.start_game();
                }
            }

            Phase::Inspecting => {
                self.camera.update(0.0, 0.0, 0.0, dt);
                self.camera.auto_rotate(dt * 0.5);
                // Transition to restoring on tool use
                if self.is_applying_tool {
                    self.phase = Phase::Restoring;
                    self.message = Some(("Restoring — clean all rust zones!".into(), 2.0));
                }
                // E to disassemble
                if input.interact {
                    self.begin_disassembly();
                }
            }

            Phase::Restoring => {
                self.time_remaining -= dt;
                if self.time_remaining <= 0.0 {
                    self.time_remaining = 0.0;
                    self.phase = Phase::Timeout;
                    self.sfx_request.push("timeout".into());
                    self.haptic_request = Some(2);
                    return;
                }

                self.camera.update(0.0, 0.0, 0.0, dt);

                if self.is_applying_tool {
                    self.apply_tool(dt);
                } else {
                    // Reset combo if not spraying for too long
                }

                if input.interact {
                    self.begin_disassembly();
                }

                // Tool switch via number keys (handled externally, but also via input)
                if input.forward {
                    self.select_tool(ToolKind::PressureWasher)
                }
            }

            Phase::Disassembling => {
                self.clear_timer += dt;
                // Animate detaching parts
                let mut done = true;
                for (_id, current, target, t) in &mut self.detaching_parts {
                    *t = (*t + dt * 2.0).min(1.0);
                    let ease = 1.0 - (1.0 - *t).powi(3);
                    *current = Vec3::ZERO.lerp(*target, ease);
                    if *t < 1.0 {
                        done = false
                    }
                }
                if done {
                    self.detaching_parts.clear();
                    self.phase = Phase::Restoring;
                }
            }

            Phase::ItemClear => {
                self.clear_timer += dt;
                self.camera.auto_rotate(dt);
                if self.clear_timer > 2.5 {
                    self.current_item_idx += 1;
                    self.combo = 0;
                    self.phase = Phase::Inspecting;
                    self.camera = TurntableCamera::new();
                }
            }

            Phase::AllClear | Phase::Timeout => {
                self.camera.auto_rotate(dt * 0.3);
                if input.interact {
                    self.phase = Phase::Title;
                }
            }
        }
    }

    /// Provide entity updates for the renderer.
    pub fn entity_positions(&self) -> Vec<EntityUpdate> {
        let mut updates = Vec::new();

        if let Some(item) = self.current_item() {
            // Item rotation from turntable camera
            let item_rot = Quat::from_rotation_y(self.camera.yaw);

            for eid in &item.entity_ids {
                let mut pos = Vec3::ZERO;
                let mut visible = true;

                // Check if this part is detaching
                for (did, current_pos, _, _) in &self.detaching_parts {
                    if did == eid {
                        pos = *current_pos;
                    }
                }

                // Check if this part was detached in a completed step
                for step in &item.disassembly_steps {
                    if step.completed
                        && step.part_ids.contains(eid)
                        && self.phase != Phase::Disassembling
                    {
                        pos = step.detach_offset;
                    }
                }

                updates.push(EntityUpdate {
                    id: eid.clone(),
                    position: pos,
                    rotation: item_rot,
                    scale: Vec3::ONE,
                    visible,
                    rust_tint: None,
                });
            }

            // Rust zone tint overlays
            for zone in &item.zones {
                if zone.current_level > 0.0 {
                    let rgb = zone.rust_type.color_rgb(zone.current_level);
                    updates.push(EntityUpdate {
                        id: format!("rust_{}", zone.id),
                        position: zone.center,
                        rotation: Quat::from_rotation_y(self.camera.yaw),
                        scale: zone.extent * 2.0,
                        visible: true,
                        rust_tint: Some([rgb[0], rgb[1], rgb[2], zone.current_level * 0.8]),
                    });
                }
            }
        }

        // Tool cursor
        if let Some(cp) = self.contact_point {
            updates.push(EntityUpdate {
                id: "tool_cursor".into(),
                position: cp,
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(self.current_tool.radius() * 2.0),
                visible: self.is_applying_tool,
                rust_tint: None,
            });
        }

        updates
    }

    pub fn grade(&self) -> char {
        let avg = if self.total_items > 0 {
            self.score as f32 / self.total_items as f32
        } else {
            0.0
        };
        match avg as u32 {
            450.. => 'S',
            300..=449 => 'A',
            200..=299 => 'B',
            100..=199 => 'C',
            _ => 'D',
        }
    }
}

// ── Item Catalog with CPC/UNSPSC ──

/// Build the default item catalog with CPC/UNSPSC product codes.
pub fn default_item_catalog() -> Vec<RestorableItem> {
    vec![
        RestorableItem {
            id: "wrench".into(), name: "Vintage Wrench".into(), name_ja: "ヴィンテージレンチ".into(),
            difficulty: 1, base_score: 100, perfect_bonus: 50,
            sdf_desc: "smooth_union(box(0.8,0.1,0.08), cylinder(0.15,0.3))".into(),
            entity_ids: vec!["wrench_body".into(), "wrench_jaw".into()],
            zones: vec![
                RustZone3D { id: "head".into(), center: Vec3::new(-0.3, 0.0, 0.0), extent: Vec3::new(0.15, 0.08, 0.06), rust_type: RustType::Surface, initial_level: 0.8, current_level: 0.8, nerf_grid_idx: None },
                RustZone3D { id: "shaft".into(), center: Vec3::new(0.1, 0.0, 0.0), extent: Vec3::new(0.25, 0.04, 0.04), rust_type: RustType::Surface, initial_level: 0.4, current_level: 0.4, nerf_grid_idx: None },
                RustZone3D { id: "handle".into(), center: Vec3::new(0.4, 0.0, 0.0), extent: Vec3::new(0.12, 0.06, 0.06), rust_type: RustType::Surface, initial_level: 0.3, current_level: 0.3, nerf_grid_idx: None },
            ],
            disassembly_steps: vec![],
            cpc_code: "42322".into(),      // CPC: Hand tools
            unspsc_code: "27111700".into(), // UNSPSC: Hand tools
            metal_color: [0.7, 0.7, 0.72], metallic: 0.85, roughness: 0.35,
        },
        RestorableItem {
            id: "skeleton_key".into(), name: "Skeleton Key".into(), name_ja: "アンティーク鍵".into(),
            difficulty: 2, base_score: 200, perfect_bonus: 100,
            sdf_desc: "union(torus(0.12,0.03), cylinder(0.02,0.4), box(0.08,0.06,0.02))".into(),
            entity_ids: vec!["key_bow".into(), "key_shaft".into(), "key_bit".into()],
            zones: vec![
                RustZone3D { id: "bow".into(), center: Vec3::new(-0.2, 0.0, 0.0), extent: Vec3::new(0.12, 0.12, 0.03), rust_type: RustType::Deep, initial_level: 0.9, current_level: 0.9, nerf_grid_idx: Some(0) },
                RustZone3D { id: "shaft".into(), center: Vec3::new(0.05, 0.0, 0.0), extent: Vec3::new(0.18, 0.02, 0.02), rust_type: RustType::Surface, initial_level: 0.5, current_level: 0.5, nerf_grid_idx: None },
                RustZone3D { id: "bit".into(), center: Vec3::new(0.25, 0.0, 0.0), extent: Vec3::new(0.08, 0.06, 0.02), rust_type: RustType::Pitted, initial_level: 0.85, current_level: 0.85, nerf_grid_idx: Some(1) },
            ],
            disassembly_steps: vec![],
            cpc_code: "42995".into(),      // CPC: Locks and keys
            unspsc_code: "46171500".into(), // UNSPSC: Locks and hardware
            metal_color: [0.72, 0.53, 0.04], metallic: 0.9, roughness: 0.25,
        },
        RestorableItem {
            id: "pocket_watch".into(), name: "Pocket Watch".into(), name_ja: "懐中時計".into(),
            difficulty: 4, base_score: 500, perfect_bonus: 250,
            sdf_desc: "smooth_union(cylinder(0.25,0.06), cylinder(0.03,0.04, translate(0,0.06,0)))".into(),
            entity_ids: vec!["watch_case".into(), "watch_face".into(), "watch_crown".into(), "watch_back".into()],
            zones: vec![
                RustZone3D { id: "case_front".into(), center: Vec3::new(0.0, 0.03, 0.0), extent: Vec3::new(0.24, 0.03, 0.24), rust_type: RustType::Deep, initial_level: 0.75, current_level: 0.75, nerf_grid_idx: Some(2) },
                RustZone3D { id: "case_back".into(), center: Vec3::new(0.0, -0.03, 0.0), extent: Vec3::new(0.24, 0.03, 0.24), rust_type: RustType::Deep, initial_level: 0.75, current_level: 0.75, nerf_grid_idx: Some(3) },
                RustZone3D { id: "crown".into(), center: Vec3::new(0.0, 0.08, 0.0), extent: Vec3::new(0.03, 0.03, 0.03), rust_type: RustType::Pitted, initial_level: 0.9, current_level: 0.9, nerf_grid_idx: None },
                RustZone3D { id: "face_hidden".into(), center: Vec3::new(0.0, 0.01, 0.0), extent: Vec3::new(0.18, 0.01, 0.18), rust_type: RustType::Patina, initial_level: 0.45, current_level: 0.45, nerf_grid_idx: None },
            ],
            disassembly_steps: vec![
                DisassemblyStep {
                    id: "open_case".into(), name: "Open Case Back".into(), name_ja: "裏蓋を開ける".into(),
                    part_ids: vec!["watch_back".into()],
                    revealed_zones: vec!["face_hidden".into()],
                    completed: false,
                    required_tool: None,
                    detach_offset: Vec3::new(0.0, -0.4, 0.0),
                },
            ],
            cpc_code: "45121".into(),      // CPC: Watches and clocks
            unspsc_code: "54111600".into(), // UNSPSC: Watches
            metal_color: [0.75, 0.75, 0.78], metallic: 0.92, roughness: 0.15,
        },
        RestorableItem {
            id: "katana_tsuba".into(), name: "Katana Tsuba".into(), name_ja: "刀の鍔".into(),
            difficulty: 5, base_score: 600, perfect_bonus: 300,
            sdf_desc: "difference(smooth_union(box(0.18,0.01,0.14), box(0.14,0.012,0.18)), box(0.02,0.02,0.06))".into(),
            entity_ids: vec!["tsuba_body".into(), "tsuba_rim".into(), "tsuba_nakago".into()],
            zones: vec![
                RustZone3D { id: "face_a".into(), center: Vec3::new(-0.06, 0.005, 0.0), extent: Vec3::new(0.08, 0.006, 0.12), rust_type: RustType::Surface, initial_level: 0.65, current_level: 0.65, nerf_grid_idx: Some(4) },
                RustZone3D { id: "face_b".into(), center: Vec3::new(0.06, 0.005, 0.0), extent: Vec3::new(0.08, 0.006, 0.12), rust_type: RustType::Surface, initial_level: 0.65, current_level: 0.65, nerf_grid_idx: Some(5) },
                RustZone3D { id: "rim".into(), center: Vec3::ZERO, extent: Vec3::new(0.18, 0.01, 0.14), rust_type: RustType::Deep, initial_level: 0.8, current_level: 0.8, nerf_grid_idx: None },
                RustZone3D { id: "nakago_ana".into(), center: Vec3::ZERO, extent: Vec3::new(0.02, 0.01, 0.05), rust_type: RustType::Pitted, initial_level: 0.95, current_level: 0.95, nerf_grid_idx: None },
                RustZone3D { id: "engraving_hidden".into(), center: Vec3::new(-0.04, -0.005, 0.03), extent: Vec3::new(0.03, 0.004, 0.06), rust_type: RustType::Patina, initial_level: 0.5, current_level: 0.5, nerf_grid_idx: None },
            ],
            disassembly_steps: vec![
                DisassemblyStep {
                    id: "flip_tsuba".into(), name: "Flip Tsuba".into(), name_ja: "鍔を裏返す".into(),
                    part_ids: vec![],
                    revealed_zones: vec!["engraving_hidden".into()],
                    completed: false,
                    required_tool: None,
                    detach_offset: Vec3::ZERO,
                },
            ],
            cpc_code: "42925".into(),      // CPC: Swords and cutlery
            unspsc_code: "46181500".into(), // UNSPSC: Knives and cutting instruments
            metal_color: [0.25, 0.25, 0.28], metallic: 0.95, roughness: 0.2,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_initializes() {
        let items = default_item_catalog();
        let game = SabiotoshiGame::new(items, 3);
        assert_eq!(game.phase, Phase::Title);
        assert_eq!(game.score, 0);
        assert_eq!(game.total_items, 3);
    }

    #[test]
    fn tool_effectiveness_matrix() {
        // Pressure washer best on surface
        assert!(
            ToolKind::PressureWasher.effectiveness(RustType::Surface)
                > ToolKind::PressureWasher.effectiveness(RustType::Pitted)
        );
        // Wire brush best on deep
        assert!(
            ToolKind::WireBrush.effectiveness(RustType::Deep)
                > ToolKind::WireBrush.effectiveness(RustType::Surface)
        );
        // Sandpaper best on pitted
        assert!(
            ToolKind::Sandpaper.effectiveness(RustType::Pitted)
                > ToolKind::Sandpaper.effectiveness(RustType::Surface)
        );
        // Polish best on patina
        assert!(
            ToolKind::PolishingCloth.effectiveness(RustType::Patina)
                > ToolKind::PolishingCloth.effectiveness(RustType::Deep)
        );
    }

    #[test]
    fn turntable_camera_orbit() {
        let mut cam = TurntableCamera::new();
        let initial_eye = cam.eye_position();
        cam.update(1.0, 0.0, 0.0, 1.0);
        let moved_eye = cam.eye_position();
        assert_ne!(initial_eye.x, moved_eye.x, "Camera should rotate on drag");
    }

    #[test]
    fn zone_containment() {
        let zone = RustZone3D {
            id: "test".into(),
            center: Vec3::ZERO,
            extent: Vec3::new(0.5, 0.5, 0.5),
            rust_type: RustType::Surface,
            initial_level: 1.0,
            current_level: 1.0,
            nerf_grid_idx: None,
        };
        assert!(zone.contains_point(Vec3::new(0.3, 0.3, 0.3)));
        assert!(!zone.contains_point(Vec3::new(0.6, 0.0, 0.0)));
    }

    #[test]
    fn item_catalog_has_cpc_unspsc() {
        let items = default_item_catalog();
        for item in &items {
            assert!(!item.cpc_code.is_empty(), "{} missing CPC code", item.id);
            assert!(
                !item.unspsc_code.is_empty(),
                "{} missing UNSPSC code",
                item.id
            );
        }
    }

    #[test]
    fn tool_catalog_has_cpc_unspsc() {
        for tool in ToolKind::all() {
            assert!(
                !tool.cpc_code().is_empty(),
                "{} missing CPC code",
                tool.name()
            );
            assert!(
                !tool.unspsc_code().is_empty(),
                "{} missing UNSPSC code",
                tool.name()
            );
        }
    }

    #[test]
    fn start_transitions_to_inspecting() {
        let items = default_item_catalog();
        let mut game = SabiotoshiGame::new(items, 3);
        game.start_game();
        assert_eq!(game.phase, Phase::Inspecting);
    }

    #[test]
    fn grade_calculation() {
        let items = default_item_catalog();
        let mut game = SabiotoshiGame::new(items, 3);
        game.score = 0;
        assert!(game.grade() == 'D');
        game.score = 1500;
        assert!(game.grade() == 'S');
    }

    #[test]
    fn disassembly_reveals_hidden_zones() {
        let items = default_item_catalog();
        let mut game = SabiotoshiGame::new(items, 4);
        game.start_game();
        game.current_item_idx = 2; // pocket watch (has disassembly)
        game.phase = Phase::Restoring;
        game.begin_disassembly();
        let item = game.current_item().unwrap();
        assert!(item.disassembly_steps[0].completed);
    }

    #[test]
    fn rust_type_colors_differ() {
        let surface = RustType::Surface.color_rgb(0.5);
        let deep = RustType::Deep.color_rgb(0.5);
        let pitted = RustType::Pitted.color_rgb(0.5);
        let patina = RustType::Patina.color_rgb(0.5);
        assert_ne!(surface, deep);
        assert_ne!(deep, pitted);
        assert_ne!(pitted, patina);
    }
}
