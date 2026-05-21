//! Reference `VehicleAssembly` builders for the public examples and the
//! integration-style smoke tests. Kept as a `pub mod` so external tools
//! (driver.gftd.ai backend / CI rigs) can spin up a known-good vehicle
//! without reinventing the topology.

pub mod roadster;
pub mod synth_sedan;

pub use roadster::roadster_na;
pub use synth_sedan::synth_sedan;
