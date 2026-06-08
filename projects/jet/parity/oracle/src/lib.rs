// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src.md#schema
// CODEGEN-BEGIN

//! `jet-parity-oracle` — headless DOM reference runner.
//!
//! Implements the oracle described in
//! `.aw/tech-design/projects/jet/specs/parity-dom-reference-runner.md`.
//!
//! Browser-driving code paths are abstracted behind the [`BrowserSession`]
//! trait so unit tests can run without a live Chromium. The default
//! production implementation [`runner::JetBrowserSession`] launches Chromium
//! directly and captures DOM oracle artifacts through CDP.

pub mod artifacts;
pub mod channels;
pub mod manifest;
pub mod runner;

pub use artifacts::{ArtifactBundle, ArtifactWriter};
pub use channels::{
    a11y::A11yChannel, focus::FocusChannel, ime::ImeChannel, pixel::PixelChannel,
    pointer::PointerChannel, Channel, ChannelArtifact, ChannelCtx, ChannelError, DeterministicPrng,
};
pub use manifest::{FixtureManifest, ManifestError};
pub use runner::{
    BrowserKind, BrowserSession, JetBrowserSession, MatrixEntry, PageHost,
    PlaywrightBrowserSession, Runner, RunnerConfig, RunnerError, StubBrowserSession,
};

use std::path::Path;

/// @spec parity-dom-reference-runner.md#Logic
///
/// High-level entry point invoked by the jet test harness and the
/// `parity-oracle` CLI binary. Drives the fixed five-channel sequence
/// (pixel → a11y → focus → pointer → ime) defined in §Logic and writes
/// the resulting [`ArtifactBundle`] to disk.
pub async fn run_fixture(
    config: &RunnerConfig,
    fixture_path: &Path,
    matrix_entry: MatrixEntry,
) -> Result<ArtifactBundle, RunnerError> {
    let mut runner = Runner::new(config.clone(), matrix_entry);
    runner.run(fixture_path).await
}
// CODEGEN-END
