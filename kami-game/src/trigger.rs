//! Trigger system: portal traversal + item pickup via sensor collisions.

use crate::physics::{PhysicsCollider, TriggerKind, TriggerZone};
use hecs::World;
use rapier3d::prelude::ColliderHandle;

/// Events emitted by the trigger system.
#[derive(Debug, Clone)]
pub enum TriggerEvent {
    Portal {
        entity_collider: ColliderHandle,
        island_id: String,
    },
    ItemPickup {
        entity_collider: ColliderHandle,
        item_id: String,
    },
    Damage {
        entity_collider: ColliderHandle,
        amount: u32,
    },
}

/// Process sensor intersections and emit trigger events.
pub fn process_triggers(
    world: &World,
    intersection_pairs: &[(ColliderHandle, ColliderHandle)],
) -> Vec<TriggerEvent> {
    let mut events = Vec::new();

    // Build lookup: collider_handle → TriggerZone
    let triggers: Vec<(ColliderHandle, TriggerKind, String)> = world
        .query::<&TriggerZone>()
        .iter()
        .map(|(_, tz)| (tz.collider_handle, tz.kind.clone(), tz.data.clone()))
        .collect();

    for &(c1, c2) in intersection_pairs {
        for (trigger_handle, kind, data) in &triggers {
            let (entity_collider, _is_trigger) = if c1 == *trigger_handle {
                (c2, true)
            } else if c2 == *trigger_handle {
                (c1, true)
            } else {
                continue;
            };

            match kind {
                TriggerKind::Portal => {
                    events.push(TriggerEvent::Portal {
                        entity_collider,
                        island_id: data.clone(),
                    });
                }
                TriggerKind::ItemPickup => {
                    events.push(TriggerEvent::ItemPickup {
                        entity_collider,
                        item_id: data.clone(),
                    });
                }
                TriggerKind::DamageZone => {
                    events.push(TriggerEvent::Damage {
                        entity_collider,
                        amount: data.parse().unwrap_or(10),
                    });
                }
            }
        }
    }

    events
}
