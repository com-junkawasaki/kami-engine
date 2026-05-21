//! kami-scene-graph: Scene DAG with parent-child transform hierarchy.
//!
//! Integrates with hecs ECS. Entities have Parent/Children components.
//! World transforms computed by traversing the DAG top-down.

use glam::{Mat4, Quat, Vec3};
use hecs::World;

/// Local transform component.
pub struct LocalTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for LocalTransform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl LocalTransform {
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

/// Computed world transform (updated by propagation).
pub struct WorldTransform(pub Mat4);

/// Parent entity reference.
pub struct Parent(pub hecs::Entity);

/// Children entity list.
pub struct Children(pub Vec<hecs::Entity>);

/// Root marker (no parent).
pub struct Root;

/// Propagate transforms down the scene graph.
/// Call once per frame after updating LocalTransforms.
pub fn propagate_transforms(world: &mut World) {
    // Phase 1: Collect roots (entities with Root + LocalTransform)
    let roots: Vec<hecs::Entity> = world
        .query::<(&Root, &LocalTransform)>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    // Phase 2: BFS from each root
    for root in roots {
        propagate_recursive(world, root, Mat4::IDENTITY);
    }
}

fn propagate_recursive(world: &mut World, entity: hecs::Entity, parent_world: Mat4) {
    let local = world
        .get::<&LocalTransform>(entity)
        .ok()
        .map(|t| t.matrix());
    let world_mat = match local {
        Some(m) => parent_world * m,
        None => parent_world,
    };

    // Set or insert WorldTransform
    if world.get::<&WorldTransform>(entity).is_ok() {
        if let Ok(mut wt) = world.get::<&mut WorldTransform>(entity) {
            wt.0 = world_mat;
        }
    } else {
        let _ = world.insert_one(entity, WorldTransform(world_mat));
    }

    // Recurse into children
    let children: Vec<hecs::Entity> = world
        .get::<&Children>(entity)
        .ok()
        .map(|c| c.0.clone())
        .unwrap_or_default();

    for child in children {
        propagate_recursive(world, child, world_mat);
    }
}

/// Attach child to parent (sets Parent component + updates Children list).
pub fn attach(world: &mut World, parent: hecs::Entity, child: hecs::Entity) {
    let _ = world.insert_one(child, Parent(parent));
    if let Ok(mut children) = world.get::<&mut Children>(parent) {
        children.0.push(child);
    } else {
        let _ = world.insert_one(parent, Children(vec![child]));
    }
}

/// Detach child from parent.
pub fn detach(world: &mut World, child: hecs::Entity) {
    if let Ok(parent_ref) = world.get::<&Parent>(child) {
        let parent = parent_ref.0;
        drop(parent_ref);
        if let Ok(mut children) = world.get::<&mut Children>(parent) {
            children.0.retain(|&e| e != child);
        }
    }
    let _ = world.remove_one::<Parent>(child);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_graph() {
        let mut world = World::new();
        let root = world.spawn((
            Root,
            LocalTransform {
                position: Vec3::new(10.0, 0.0, 0.0),
                ..Default::default()
            },
        ));
        let child = world.spawn((LocalTransform {
            position: Vec3::new(5.0, 0.0, 0.0),
            ..Default::default()
        },));
        attach(&mut world, root, child);
        propagate_transforms(&mut world);

        let wt = world.get::<&WorldTransform>(child).unwrap();
        // Child world pos = root(10,0,0) + child(5,0,0) = (15,0,0)
        let pos = wt.0.col(3);
        assert!((pos.x - 15.0).abs() < 0.01);
    }
}
