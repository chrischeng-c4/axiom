// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
// CODEGEN-BEGIN
//! jet-parity-corpus — parser, hasher, verifier, and CLI for the MUI parity
//! fixture corpus described by `.aw/tech-design/projects/jet/specs/jet-parity-fixture-corpus.md`.

pub mod cli;
pub mod hash;
pub mod manifest;
pub mod verify;

pub use cli::{run, FixturesCli, FixturesCmd};
pub use hash::hash_jsx_file;
pub use manifest::{
    parse_manifest, CorpusError, FixtureEntry, FixtureManifest, ObservationChannel,
};
pub use verify::{verify, DriftedFixture, FixtureStatus, VerifyEntry, VerifyReport};
// CODEGEN-END
