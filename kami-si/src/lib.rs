/// KAMI Signal Integrity — transmission line analysis, eye diagram generation,
/// crosstalk analysis, and S-parameter extraction.

pub mod transmission_line;
pub mod eye_diagram;
pub mod crosstalk;
pub mod s_param;

pub use transmission_line::{TLineParams, TLineType};
pub use eye_diagram::{EyeMetrics, EyeDiagramData};
pub use crosstalk::{CrosstalkResult, CouplingType};
pub use s_param::{SParameter, SParamMetrics};
