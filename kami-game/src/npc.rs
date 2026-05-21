//! NPC AI: behavior tree + LLM dialogue stub + brainrot behaviors.

use crate::common::SimpleRng;
use glam::Vec3;

/// NPC behavior state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Behavior {
    Idle,
    Patrol { waypoint_index: usize },
    Chase { target_entity: u32 },
    Talk { partner_entity: u32 },
}

// ---------------------------------------------------------------------------
// Brainrot NPC Behaviors
// ---------------------------------------------------------------------------

/// Phase of the Skibidi pop-up cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkibidiPhase {
    Rise,
    Hold,
    Drop,
    Wait,
}

/// Position/rotation delta returned by brainrot behavior ticks.
#[derive(Debug, Clone, Default)]
pub struct BrainrotUpdate {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub scale: f32,
    /// If set, teleport to this position (ignore dx/dy/dz).
    pub teleport: Option<Vec3>,
    /// Puddle spawn position (Grimace).
    pub spawn_puddle: Option<Vec3>,
    /// Damage cube spawn (Ohio Boss).
    pub spawn_damage_cubes: bool,
    /// Item steal trigger (Fanum).
    pub steal_item: bool,
    /// Charm gesture active (Rizz).
    pub charm_active: bool,
}

/// Skibidi: rises up, holds, drops, waits. Head yaw oscillation while up.
#[derive(Debug, Clone)]
pub struct SkibidiBehavior {
    pub phase: SkibidiPhase,
    pub timer: f32,
}

impl SkibidiBehavior {
    pub fn new() -> Self {
        Self {
            phase: SkibidiPhase::Rise,
            timer: 0.0,
        }
    }

    fn phase_duration(&self) -> f32 {
        match self.phase {
            SkibidiPhase::Rise => 1.0,
            SkibidiPhase::Hold => 0.5,
            SkibidiPhase::Drop => 0.5,
            SkibidiPhase::Wait => 2.0,
        }
    }

    fn next_phase(&self) -> SkibidiPhase {
        match self.phase {
            SkibidiPhase::Rise => SkibidiPhase::Hold,
            SkibidiPhase::Hold => SkibidiPhase::Drop,
            SkibidiPhase::Drop => SkibidiPhase::Wait,
            SkibidiPhase::Wait => SkibidiPhase::Rise,
        }
    }

    pub fn tick(&mut self, dt: f32) -> BrainrotUpdate {
        self.timer += dt;
        let dur = self.phase_duration();
        if self.timer >= dur {
            self.timer -= dur;
            self.phase = self.next_phase();
        }
        let t = self.timer / dur;
        let mut u = BrainrotUpdate::default();
        u.scale = 1.0;
        match self.phase {
            SkibidiPhase::Rise => {
                // y += 2.0 over 1s
                u.dy = 2.0 * dt;
                u.yaw = (self.timer * 20.0).sin() * 1.5;
            }
            SkibidiPhase::Hold => {
                u.yaw = (self.timer * 20.0).sin() * 1.5;
            }
            SkibidiPhase::Drop => {
                // y -= 2.0 over 0.5s => rate = 4.0/s
                u.dy = -4.0 * dt;
            }
            SkibidiPhase::Wait => {
                let _ = t; // idle
            }
        }
        u
    }
}

/// Grimace: slow pursuit + puddle spawning + wobble scale.
#[derive(Debug, Clone)]
pub struct GrimaceBehavior {
    pub puddle_timer: f32,
    pub wobble_phase: f32,
}

impl GrimaceBehavior {
    pub fn new() -> Self {
        Self {
            puddle_timer: 0.0,
            wobble_phase: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32, my_pos: Vec3, target_pos: Vec3) -> BrainrotUpdate {
        let speed = 0.3;
        let dir = (target_pos - my_pos).normalize_or_zero();
        let vel = dir * speed * dt;

        self.puddle_timer += dt;
        let spawn = if self.puddle_timer >= 5.0 {
            self.puddle_timer -= 5.0;
            Some(my_pos)
        } else {
            None
        };

        self.wobble_phase += dt * 2.0 * std::f32::consts::TAU;
        let scale = 1.0 + 0.05 * self.wobble_phase.sin();

        BrainrotUpdate {
            dx: vel.x,
            dy: vel.y,
            dz: vel.z,
            scale,
            spawn_puddle: spawn,
            ..Default::default()
        }
    }
}

/// Sigma: stands still on the Sigma Throne. Nods when player within 5m.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigmaState {
    Idle,
    Nodding,
}

#[derive(Debug, Clone)]
pub struct SigmaBehavior {
    pub state: SigmaState,
    pub nod_timer: f32,
}

impl SigmaBehavior {
    pub fn new() -> Self {
        Self {
            state: SigmaState::Idle,
            nod_timer: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32, _my_pos: Vec3, nearest_player_dist: f32) -> BrainrotUpdate {
        let mut u = BrainrotUpdate {
            scale: 1.0,
            ..Default::default()
        };
        match self.state {
            SigmaState::Idle => {
                if nearest_player_dist < 5.0 {
                    self.state = SigmaState::Nodding;
                    self.nod_timer = 0.0;
                }
            }
            SigmaState::Nodding => {
                self.nod_timer += dt;
                let nod_duration = 0.5;
                if self.nod_timer < nod_duration * 0.5 {
                    // pitch down 15 degrees
                    u.pitch = -15.0_f32.to_radians() * 2.0 * dt / nod_duration;
                } else if self.nod_timer < nod_duration {
                    // pitch back up
                    u.pitch = 15.0_f32.to_radians() * 2.0 * dt / nod_duration;
                } else {
                    self.state = SigmaState::Idle;
                    self.nod_timer = 0.0;
                }
            }
        }
        u
    }
}

/// Ohio Boss: teleports every 3s, slow rotation, spawns damage cubes near player.
#[derive(Debug, Clone)]
pub struct OhioBossBehavior {
    pub teleport_timer: f32,
    pub rng: SimpleRng,
}

impl OhioBossBehavior {
    pub fn new(seed: u32) -> Self {
        Self {
            teleport_timer: 0.0,
            rng: SimpleRng::new(seed),
        }
    }

    pub fn tick(&mut self, dt: f32, my_pos: Vec3, nearest_player_dist: f32) -> BrainrotUpdate {
        self.teleport_timer += dt;
        let mut u = BrainrotUpdate {
            scale: 1.0,
            ..Default::default()
        };
        u.yaw = 2.0 * dt;

        if self.teleport_timer >= 3.0 {
            self.teleport_timer -= 3.0;
            let angle = self.rng.next_f32() * std::f32::consts::TAU;
            let radius = self.rng.next_f32() * 20.0;
            let tx = my_pos.x + angle.cos() * radius;
            let tz = my_pos.z + angle.sin() * radius;
            u.teleport = Some(Vec3::new(tx, my_pos.y, tz));
        }

        if nearest_player_dist < 10.0 {
            u.spawn_damage_cubes = true;
        }

        u
    }
}

/// Fanum: patrols food stalls, steals nearby player items with cooldown.
#[derive(Debug, Clone)]
pub struct FanumBehavior {
    pub waypoint_index: usize,
    pub waypoints: Vec<Vec3>,
    pub steal_cooldown: f32,
}

impl FanumBehavior {
    pub fn new(waypoints: Vec<Vec3>) -> Self {
        Self {
            waypoint_index: 0,
            waypoints,
            steal_cooldown: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32, my_pos: Vec3, nearby_item: bool) -> BrainrotUpdate {
        self.steal_cooldown = (self.steal_cooldown - dt).max(0.0);
        let mut u = BrainrotUpdate {
            scale: 1.0,
            ..Default::default()
        };

        // Steal if item nearby and cooldown expired.
        if nearby_item && self.steal_cooldown <= 0.0 {
            u.steal_item = true;
            self.steal_cooldown = 3.0;
        }

        // Patrol between food stalls at speed 1.0.
        if !self.waypoints.is_empty() {
            let target = self.waypoints[self.waypoint_index];
            let dir = target - my_pos;
            if dir.length() < 0.5 {
                self.waypoint_index = (self.waypoint_index + 1) % self.waypoints.len();
            }
            let vel = dir.normalize_or_zero() * 1.0 * dt;
            u.dx = vel.x;
            u.dy = vel.y;
            u.dz = vel.z;
        }

        u
    }
}

/// Rizz: approach player, charm, walk away, repeat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RizzPhase {
    Approach,
    Charm,
    WalkAway,
}

#[derive(Debug, Clone)]
pub struct RizzBehavior {
    pub phase: RizzPhase,
    pub timer: f32,
    pub walkaway_target: Vec3,
    pub rng: SimpleRng,
}

impl RizzBehavior {
    pub fn new(seed: u32) -> Self {
        Self {
            phase: RizzPhase::Approach,
            timer: 0.0,
            walkaway_target: Vec3::ZERO,
            rng: SimpleRng::new(seed),
        }
    }

    pub fn tick(&mut self, dt: f32, my_pos: Vec3, nearest_player_pos: Vec3) -> BrainrotUpdate {
        let mut u = BrainrotUpdate {
            scale: 1.0,
            ..Default::default()
        };
        let dist = my_pos.distance(nearest_player_pos);

        match self.phase {
            RizzPhase::Approach => {
                if dist <= 3.0 {
                    self.phase = RizzPhase::Charm;
                    self.timer = 0.0;
                } else {
                    let dir = (nearest_player_pos - my_pos).normalize_or_zero();
                    let vel = dir * 0.5 * dt;
                    u.dx = vel.x;
                    u.dy = vel.y;
                    u.dz = vel.z;
                }
            }
            RizzPhase::Charm => {
                self.timer += dt;
                u.pitch = -10.0_f32.to_radians();
                u.charm_active = true;
                if self.timer >= 2.0 {
                    self.phase = RizzPhase::WalkAway;
                    self.timer = 0.0;
                    let angle = self.rng.next_f32() * std::f32::consts::TAU;
                    let r = self.rng.next_f32() * 10.0;
                    self.walkaway_target = Vec3::new(
                        my_pos.x + angle.cos() * r,
                        my_pos.y,
                        my_pos.z + angle.sin() * r,
                    );
                }
            }
            RizzPhase::WalkAway => {
                let dir = (self.walkaway_target - my_pos).normalize_or_zero();
                let vel = dir * 0.5 * dt;
                u.dx = vel.x;
                u.dy = vel.y;
                u.dz = vel.z;
                if my_pos.distance(self.walkaway_target) < 0.5 {
                    self.phase = RizzPhase::Approach;
                    self.timer = 0.0;
                }
            }
        }
        u
    }
}

/// NPC component attached to hecs entity.
#[derive(Debug, Clone)]
pub struct Npc {
    pub name: String,
    pub behavior: Behavior,
    pub waypoints: Vec<Vec3>,
    pub patrol_speed: f32,
    pub detection_radius: f32,
    pub talk_radius: f32,
    pub dialogue_cooldown: f32,
    cooldown_timer: f32,
}

impl Npc {
    pub fn new(name: &str, waypoints: Vec<Vec3>) -> Self {
        Self {
            name: name.to_string(),
            behavior: Behavior::Patrol { waypoint_index: 0 },
            waypoints,
            patrol_speed: 2.0,
            detection_radius: 8.0,
            talk_radius: 3.0,
            dialogue_cooldown: 10.0,
            cooldown_timer: 0.0,
        }
    }

    /// Tick behavior tree. Returns desired movement direction + optional dialogue trigger.
    pub fn tick(&mut self, my_pos: Vec3, players: &[(u32, Vec3)], dt: f32) -> NpcAction {
        self.cooldown_timer = (self.cooldown_timer - dt).max(0.0);

        // Check for nearby player
        let nearest = players
            .iter()
            .map(|(id, p)| (*id, my_pos.distance(*p)))
            .filter(|(_, d)| *d < self.detection_radius)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        match (&self.behavior, nearest) {
            // Close enough to talk
            (_, Some((player_id, dist)))
                if dist < self.talk_radius && self.cooldown_timer <= 0.0 =>
            {
                self.behavior = Behavior::Talk {
                    partner_entity: player_id,
                };
                self.cooldown_timer = self.dialogue_cooldown;
                NpcAction::Talk {
                    npc_name: self.name.clone(),
                    partner: player_id,
                }
            }
            // Player detected → chase
            (_, Some((player_id, _dist))) => {
                self.behavior = Behavior::Chase {
                    target_entity: player_id,
                };
                let target_pos = players.iter().find(|(id, _)| *id == player_id).unwrap().1;
                let dir = (target_pos - my_pos).normalize_or_zero();
                NpcAction::Move(dir * self.patrol_speed)
            }
            // No player nearby → patrol
            (Behavior::Patrol { waypoint_index }, None) => {
                if self.waypoints.is_empty() {
                    return NpcAction::Move(Vec3::ZERO);
                }
                let target = self.waypoints[*waypoint_index];
                let dir = target - my_pos;
                if dir.length() < 0.5 {
                    let next = (*waypoint_index + 1) % self.waypoints.len();
                    self.behavior = Behavior::Patrol {
                        waypoint_index: next,
                    };
                }
                NpcAction::Move(dir.normalize_or_zero() * self.patrol_speed)
            }
            // Default: return to patrol
            (_, None) => {
                self.behavior = Behavior::Patrol { waypoint_index: 0 };
                NpcAction::Move(Vec3::ZERO)
            }
        }
    }
}

/// Action returned by NPC tick.
#[derive(Debug, Clone)]
pub enum NpcAction {
    Move(Vec3),
    Talk { npc_name: String, partner: u32 },
}

/// LLM dialogue stub. In production → murakumo.etzhayyim.com API.
pub fn generate_dialogue(npc_name: &str, _player_name: &str) -> String {
    match npc_name {
        "Guard" => {
            "Halt! Who goes there? This island is protected by the KAMI World Council.".into()
        }
        "Merchant" => {
            "Welcome, traveler! I have rare gems and artifacts from distant islands.".into()
        }
        _ => format!("{npc_name}: Greetings, adventurer!"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npc_patrol() {
        let mut npc = Npc::new(
            "Guard",
            vec![Vec3::new(5.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0)],
        );
        let action = npc.tick(Vec3::ZERO, &[], 1.0 / 60.0);
        match action {
            NpcAction::Move(v) => assert!(v.length() > 0.0),
            _ => panic!("expected Move"),
        }
    }

    #[test]
    fn npc_talk_on_proximity() {
        let mut npc = Npc::new("Merchant", vec![Vec3::ZERO]);
        let players = vec![(1, Vec3::new(1.0, 0.0, 0.0))]; // within talk_radius
        let action = npc.tick(Vec3::ZERO, &players, 1.0 / 60.0);
        match action {
            NpcAction::Talk { npc_name, partner } => {
                assert_eq!(npc_name, "Merchant");
                assert_eq!(partner, 1);
            }
            _ => panic!("expected Talk"),
        }
    }

    #[test]
    fn skibidi_cycles_through_phases() {
        let mut s = SkibidiBehavior::new();
        assert_eq!(s.phase, SkibidiPhase::Rise);

        // Use exact durations to avoid float accumulation drift.
        s.tick(1.0); // rise (1.0s)
        assert_eq!(s.phase, SkibidiPhase::Hold);

        s.tick(0.5); // hold (0.5s)
        assert_eq!(s.phase, SkibidiPhase::Drop);

        s.tick(0.5); // drop (0.5s)
        assert_eq!(s.phase, SkibidiPhase::Wait);

        s.tick(2.0); // wait (2.0s)
        assert_eq!(s.phase, SkibidiPhase::Rise);

        println!("skibidi_cycles_through_phases: dop dop yes yes");
    }

    #[test]
    fn grimace_moves_toward_target() {
        let mut g = GrimaceBehavior::new();
        let my_pos = Vec3::ZERO;
        let target = Vec3::new(10.0, 0.0, 0.0);
        let u = g.tick(1.0, my_pos, target);
        // Should move in +x direction toward target
        assert!(u.dx > 0.0, "grimace dx={} should be positive", u.dx);
        assert!(u.dz.abs() < 0.001, "grimace dz should be near zero");
        // Scale should wobble around 1.0
        assert!((0.9..=1.1).contains(&u.scale), "grimace scale={}", u.scale);
        println!("grimace_moves_toward_target: wobble scale={:.3}", u.scale);
    }

    #[test]
    fn sigma_stays_still_no_player() {
        let mut s = SigmaBehavior::new();
        let my_pos = Vec3::ZERO;
        // Player far away (100m)
        let u = s.tick(1.0, my_pos, 100.0);
        assert_eq!(s.state, SigmaState::Idle);
        assert!(u.dx.abs() < 0.001);
        assert!(u.dy.abs() < 0.001);
        assert!(u.dz.abs() < 0.001);
        assert!(u.pitch.abs() < 0.001);
        println!("sigma_stays_still: sigma stare (no player nearby)");
    }

    #[test]
    fn ohio_boss_teleports_after_3s() {
        let mut o = OhioBossBehavior::new(42);
        let pos = Vec3::new(50.0, 0.0, 50.0);

        // Tick for 2.9s — no teleport yet
        let u = o.tick(2.9, pos, 100.0);
        assert!(u.teleport.is_none(), "should not teleport before 3s");

        // Tick another 0.2s (total >= 3.0s) — teleport
        let u = o.tick(0.2, pos, 100.0);
        assert!(u.teleport.is_some(), "should teleport after 3s");
        let tp = u.teleport.unwrap();
        let dist = pos.distance(tp);
        assert!(
            dist <= 20.1,
            "teleport distance={} should be within 20m radius",
            dist
        );
        println!(
            "ohio_boss_teleports: teleported to ({:.1}, {:.1}, {:.1}), dist={:.1}m",
            tp.x, tp.y, tp.z, dist
        );
    }
}
