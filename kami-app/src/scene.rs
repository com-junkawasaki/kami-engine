//! `Scene` trait — entity builder.
//!
//! A scene converts a specification (JSON-LD, YAML, procedural seed) into
//! ECS entities in the `hecs::World`. Scenes are one-shot: `build` is
//! called during `KamiApp::with_scene(...)`. Runtime scene mutation
//! (adding/removing entities) happens via tick hooks or custom commands.

use hecs::World;

pub trait Scene {
    fn build(self, world: &mut World);
}

/// Empty scene — useful for tests and minimal demos.
pub struct EmptyScene;
impl Scene for EmptyScene {
    fn build(self, _world: &mut World) {}
}
