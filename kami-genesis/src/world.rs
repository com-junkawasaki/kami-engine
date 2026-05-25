//! `World` + `Articulation` — Isaac Sim / PhysX-style API surface.
//!
//! At R1.1 the only supported `Articulation` topology is Cartpole.
//! Future articulations (Franka, ANYmal, suki, sarutahiko) plug in here at R1.5+
//! via the kami-articulated `ArticulatedSystem` and Featherstone solver.

use crate::cartpole::{CartpoleConfig, CartpoleState};
use kami_articulated::{ArticulatedSystem, JointKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorldError {
    #[error("articulation topology not supported at R1.1: {0}. Cartpole (1 prismatic + 1 revolute) is the only supported topology.")]
    UnsupportedTopology(String),
    #[error("articulation handle {0} is invalid")]
    InvalidHandle(usize),
    #[error("articulation `{0}` already registered")]
    DuplicateName(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArticulationHandle(pub usize);

/// PhysX-style scene + Isaac Sim-style World container.
///
/// Tracks articulations in a flat `Vec` and steps them in lockstep.
/// API surface mirrors:
///   - `isaacsim.core.api.World.step(render=False)`
///   - `PxScene::simulate(elapsedTime)` + `PxScene::fetchResults()`
#[derive(Debug)]
pub struct World {
    pub gravity: f32,
    pub dt: f32,
    articulations: Vec<Articulation>,
}

impl Default for World {
    fn default() -> Self {
        World { gravity: 9.81, dt: 1.0 / 60.0, articulations: Vec::new() }
    }
}

impl World {
    pub fn new(gravity: f32, dt: f32) -> Self {
        World { gravity, dt, articulations: Vec::new() }
    }

    /// Add an articulation, returning its handle. Equivalent to:
    ///   - `isaacsim.core.api.World.scene.add(articulation)`
    ///   - `PxScene::addArticulation(...)`
    pub fn add_articulation(
        &mut self,
        sys: ArticulatedSystem,
    ) -> Result<ArticulationHandle, WorldError> {
        let name = sys.name.clone();
        if self.articulations.iter().any(|a| a.name == name) {
            return Err(WorldError::DuplicateName(name));
        }
        let art = Articulation::from_urdf(sys, self.gravity, self.dt)?;
        let handle = ArticulationHandle(self.articulations.len());
        self.articulations.push(art);
        Ok(handle)
    }

    /// Advance the simulation by one `dt`.
    /// Mirrors `PxScene::simulate(dt) + PxScene::fetchResults()` and
    /// `isaacsim.core.api.World.step()`.
    pub fn step(&mut self) {
        for art in &mut self.articulations {
            art.step();
        }
    }

    pub fn get(&self, h: ArticulationHandle) -> Result<&Articulation, WorldError> {
        self.articulations.get(h.0).ok_or(WorldError::InvalidHandle(h.0))
    }

    pub fn get_mut(
        &mut self,
        h: ArticulationHandle,
    ) -> Result<&mut Articulation, WorldError> {
        self.articulations.get_mut(h.0).ok_or(WorldError::InvalidHandle(h.0))
    }

    pub fn articulation_count(&self) -> usize {
        self.articulations.len()
    }
}

/// Articulation = USD physics ArticulationRoot / PhysX PxArticulationReducedCoordinate.
///
/// At R1.1 backed by closed-form Cartpole; future articulations dispatch on
/// detected topology (kami-articulated `ArticulatedSystem` shape).
#[derive(Debug)]
pub struct Articulation {
    pub name: String,
    pub system: ArticulatedSystem,
    topology: ArticulationTopology,
    applied_action_cartpole: f32,
}

#[derive(Debug)]
enum ArticulationTopology {
    Cartpole { state: CartpoleState, cfg: CartpoleConfig },
}

impl Articulation {
    pub fn from_urdf(
        sys: ArticulatedSystem,
        gravity: f32,
        dt: f32,
    ) -> Result<Self, WorldError> {
        let topology = detect_topology(&sys, gravity, dt)?;
        let name = sys.name.clone();
        Ok(Articulation {
            name,
            system: sys,
            topology,
            applied_action_cartpole: 0.0,
        })
    }

    pub fn step(&mut self) {
        match &mut self.topology {
            ArticulationTopology::Cartpole { state, cfg } => {
                let action = self.applied_action_cartpole;
                state.step(action, cfg);
                self.applied_action_cartpole = 0.0;
            }
        }
    }

    /// Set the force applied to the cart for the next `step()`. Mirrors
    /// `PxArticulationJointReducedCoordinate::setDriveTarget` for the slider DOF.
    pub fn set_cart_force(&mut self, force: f32) {
        self.applied_action_cartpole = force;
    }

    /// Read the current state (Cartpole only at R1.1).
    pub fn cartpole_state(&self) -> Option<CartpoleState> {
        match &self.topology {
            ArticulationTopology::Cartpole { state, .. } => Some(*state),
        }
    }

    /// Mutate state (used by `reset`).
    pub fn set_cartpole_state(&mut self, new_state: CartpoleState) {
        match &mut self.topology {
            ArticulationTopology::Cartpole { state, .. } => *state = new_state,
        }
    }
}

fn detect_topology(
    sys: &ArticulatedSystem,
    gravity: f32,
    dt: f32,
) -> Result<ArticulationTopology, WorldError> {
    // Cartpole signature: 1 prismatic joint with parent=world + 1 revolute joint.
    let has_prismatic_to_world = sys
        .joints
        .iter()
        .any(|j| j.kind == JointKind::Prismatic && j.parent == "world");
    let has_one_revolute =
        sys.joints.iter().filter(|j| j.kind == JointKind::Revolute).count() == 1;
    let total_dofs = sys
        .joints
        .iter()
        .filter(|j| matches!(j.kind, JointKind::Prismatic | JointKind::Revolute))
        .count();

    if has_prismatic_to_world && has_one_revolute && total_dofs == 2 {
        let cart = sys
            .links
            .iter()
            .find(|l| l.name == "cart")
            .ok_or_else(|| WorldError::UnsupportedTopology("missing `cart` link".into()))?;
        let pole = sys
            .links
            .iter()
            .find(|l| l.name == "pole_link")
            .ok_or_else(|| {
                WorldError::UnsupportedTopology("missing `pole_link` link".into())
            })?;
        let slider = sys
            .joints
            .iter()
            .find(|j| j.kind == JointKind::Prismatic)
            .expect("checked above");
        let cfg = CartpoleConfig {
            cart_mass: cart.inertia.mass,
            pole_mass: pole.inertia.mass,
            pole_half_length: 0.25, // hardcoded from URDF cylinder length 0.5; future R1.5 reads visual
            gravity,
            force_mag: slider.effort.max(1.0),
            dt,
        };
        Ok(ArticulationTopology::Cartpole {
            state: CartpoleState::default(),
            cfg,
        })
    } else {
        Err(WorldError::UnsupportedTopology(format!(
            "{} (prismatic_to_world={}, revolute_count={}, dofs={})",
            sys.name,
            has_prismatic_to_world,
            sys.joints.iter().filter(|j| j.kind == JointKind::Revolute).count(),
            total_dofs
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARTPOLE_URDF: &str =
        include_str!("../../../../70-tools/e7m-sim/scenes/cartpole/cartpole.urdf");

    fn cartpole_world() -> (World, ArticulationHandle) {
        let sys = kami_articulated::parse_urdf(CARTPOLE_URDF).unwrap();
        let mut world = World::default();
        let h = world.add_articulation(sys).unwrap();
        (world, h)
    }

    #[test]
    fn world_loads_cartpole_urdf() {
        let (world, _) = cartpole_world();
        assert_eq!(world.articulation_count(), 1);
    }

    #[test]
    fn world_steps_cartpole_under_gravity() {
        let (mut world, h) = cartpole_world();
        world.get_mut(h).unwrap().set_cartpole_state(CartpoleState {
            theta: 0.05,
            ..Default::default()
        });
        for _ in 0..120 {
            world.step();
        }
        let s = world.get(h).unwrap().cartpole_state().unwrap();
        assert!(s.theta.abs() > 0.05, "pole should fall under gravity");
    }

    #[test]
    fn cart_force_moves_cart() {
        let (mut world, h) = cartpole_world();
        for _ in 0..60 {
            world.get_mut(h).unwrap().set_cart_force(20.0);
            world.step();
        }
        let s = world.get(h).unwrap().cartpole_state().unwrap();
        assert!(s.x > 0.0, "force should push cart in +x direction");
    }

    #[test]
    fn unsupported_topology_rejected() {
        let xml = r#"<robot name="single_link">
          <link name="a"><inertial><mass value="1.0"/><inertia ixx="0.1" iyy="0.1" izz="0.1"/></inertial></link>
        </robot>"#;
        let sys = kami_articulated::parse_urdf(xml).unwrap();
        let mut world = World::default();
        assert!(matches!(
            world.add_articulation(sys),
            Err(WorldError::UnsupportedTopology(_))
        ));
    }
}
