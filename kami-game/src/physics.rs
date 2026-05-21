//! Rapier 3D physics integration.
//!
//! PhysicsWorld wraps Rapier pipeline. hecs entities get PhysicsBody handles.
//! step() advances simulation and writes back positions to ECS.

use glam::Vec3;
use hecs::World;
use rapier3d::prelude::*;

use kami_core::actor::components::Position;

/// Handle linking a hecs entity to a Rapier rigid body.
#[derive(Debug, Clone, Copy)]
pub struct PhysicsBody {
    pub body_handle: RigidBodyHandle,
}

/// Handle linking a hecs entity to a Rapier collider.
#[derive(Debug, Clone, Copy)]
pub struct PhysicsCollider {
    pub collider_handle: ColliderHandle,
}

/// Sensor trigger (portal, item pickup zone).
#[derive(Debug, Clone)]
pub struct TriggerZone {
    pub collider_handle: ColliderHandle,
    pub kind: TriggerKind,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerKind {
    Portal,
    ItemPickup,
    DamageZone,
}

/// Physics world: owns all Rapier state.
pub struct PhysicsWorld {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub gravity: Vector<Real>,
    pub integration_params: IntegrationParameters,
    pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            gravity: vector![0.0, -9.81, 0.0],
            integration_params: IntegrationParameters::default(),
            pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        }
    }

    /// Add a ground plane at y=0.
    pub fn add_ground(&mut self) -> ColliderHandle {
        let ground = ColliderBuilder::cuboid(100.0, 0.1, 100.0)
            .translation(vector![0.0, -0.1, 0.0])
            .build();
        self.colliders.insert(ground)
    }

    /// Add a dynamic rigid body with a box collider. Returns (body_handle, collider_handle).
    pub fn add_dynamic_box(
        &mut self,
        pos: Vec3,
        half_extents: Vec3,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let rb = RigidBodyBuilder::dynamic()
            .translation(vector![pos.x, pos.y, pos.z])
            .lock_rotations()
            .build();
        let bh = self.bodies.insert(rb);

        let col = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            .restitution(0.0)
            .friction(0.5)
            .build();
        let ch = self.colliders.insert_with_parent(col, bh, &mut self.bodies);

        (bh, ch)
    }

    /// Add a kinematic body (server-controlled NPC).
    pub fn add_kinematic_box(
        &mut self,
        pos: Vec3,
        half_extents: Vec3,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let rb = RigidBodyBuilder::kinematic_position_based()
            .translation(vector![pos.x, pos.y, pos.z])
            .build();
        let bh = self.bodies.insert(rb);

        let col = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z).build();
        let ch = self.colliders.insert_with_parent(col, bh, &mut self.bodies);

        (bh, ch)
    }

    /// Add a sensor trigger zone (no physics response, only intersection detection).
    pub fn add_sensor(&mut self, pos: Vec3, half_extents: Vec3) -> ColliderHandle {
        let col = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            .translation(vector![pos.x, pos.y, pos.z])
            .sensor(true)
            .build();
        self.colliders.insert(col)
    }

    /// Apply velocity to a rigid body (player movement).
    pub fn set_linvel(&mut self, handle: RigidBodyHandle, vel: Vec3) {
        if let Some(rb) = self.bodies.get_mut(handle) {
            rb.set_linvel(vector![vel.x, rb.linvel().y, vel.z], true);
        }
    }

    /// Apply jump impulse.
    pub fn jump(&mut self, handle: RigidBodyHandle, impulse: f32) {
        if let Some(rb) = self.bodies.get_mut(handle) {
            if rb.linvel().y.abs() < 0.1 {
                rb.apply_impulse(vector![0.0, impulse, 0.0], true);
            }
        }
    }

    /// Set kinematic body position (NPC movement).
    pub fn set_kinematic_pos(&mut self, handle: RigidBodyHandle, pos: Vec3) {
        if let Some(rb) = self.bodies.get_mut(handle) {
            rb.set_next_kinematic_translation(vector![pos.x, pos.y, pos.z]);
        }
    }

    /// Step physics simulation.
    pub fn step(&mut self) {
        self.pipeline.step(
            &self.gravity,
            &self.integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    /// Write physics positions back to hecs ECS.
    pub fn sync_to_ecs(&self, world: &mut World) {
        for (_, (pos, body)) in world.query_mut::<(&mut Position, &PhysicsBody)>() {
            if let Some(rb) = self.bodies.get(body.body_handle) {
                let t = rb.translation();
                pos.0 = [t.x, t.y, t.z];
            }
        }
    }

    /// Get position of a rigid body.
    pub fn get_position(&self, handle: RigidBodyHandle) -> Option<Vec3> {
        self.bodies.get(handle).map(|rb| {
            let t = rb.translation();
            Vec3::new(t.x, t.y, t.z)
        })
    }

    /// Get all sensor intersection pairs (for trigger detection).
    pub fn sensor_intersections(&self) -> Vec<(ColliderHandle, ColliderHandle)> {
        let mut pairs = Vec::new();
        for pair in self.narrow_phase.intersection_pairs() {
            if pair.2 {
                pairs.push((pair.0, pair.1));
            }
        }
        pairs
    }

    /// Raycast from origin in direction. Returns first hit distance + collider.
    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<(f32, ColliderHandle)> {
        let ray = Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![dir.x, dir.y, dir.z],
        );
        self.query_pipeline
            .cast_ray(
                &self.bodies,
                &self.colliders,
                &ray,
                max_dist,
                true,
                QueryFilter::default(),
            )
            .map(|(handle, dist)| (dist, handle))
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_gravity() {
        let mut pw = PhysicsWorld::new();
        pw.add_ground();
        let (bh, _) = pw.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::splat(0.5));

        // Step 60 frames
        for _ in 0..60 {
            pw.step();
        }

        let pos = pw.get_position(bh).unwrap();
        // After 1 second of gravity, should be near ground (y ≈ 0.5)
        assert!(pos.y < 5.0, "body should have fallen, y={}", pos.y);
        assert!(
            pos.y > -1.0,
            "body should not fall through ground, y={}",
            pos.y
        );
    }

    #[test]
    fn physics_sensor() {
        let mut pw = PhysicsWorld::new();
        let (bh, _) = pw.add_dynamic_box(Vec3::new(0.0, 0.5, 0.0), Vec3::splat(0.5));
        let _sensor = pw.add_sensor(Vec3::new(0.0, 0.5, 0.0), Vec3::splat(1.0));

        pw.step();
        let pairs = pw.sensor_intersections();
        assert!(!pairs.is_empty(), "sensor should detect intersection");
    }
}
