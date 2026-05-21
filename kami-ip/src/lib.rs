/// KAMI IP Management — IP-XACT component catalog, bus protocol generation,
/// NoC topology synthesis, and CDC analysis.

pub mod ip_xact;
pub mod bus_protocol;
pub mod noc;
pub mod cdc;

pub use ip_xact::{IpXactComponent, BusInterface, BusType, IpPort, IpParam, IpCatalog};
pub use bus_protocol::{AxiConfig, ApbConfig};
pub use noc::{NocTopology, NocRouter, NocPort, NocConfig, NocDesign};
pub use cdc::{CdcCrossing, CdcReport, CdcViolation, CrossingType, CdcViolationKind, SynchronizerType};
