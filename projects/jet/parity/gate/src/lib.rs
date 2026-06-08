// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
// CODEGEN-BEGIN
//! Jet parity gate — library entrypoints.
//!
//! See `projects/jet/parity/data/docs/gating-manifest.md` for the v1
//! contract and exit-code semantics.
//!
//! Tracking: <https://github.com/chrischeng-c4/cclab/issues/2144>

pub mod cli;
pub mod gate;
pub mod init;
pub mod manifest;
pub mod result;
pub mod waivers;

pub use gate::{run_gate, GateReport};
pub use init::{run_init, InitReport};
pub use manifest::{AdapterSelector, Channel, GateError, GatingManifest, Tolerance};
pub use result::{ChannelResult, DiffKind, Status};
pub use waivers::{Waiver, Waivers};
// CODEGEN-END
