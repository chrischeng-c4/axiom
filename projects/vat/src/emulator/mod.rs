// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
// CODEGEN-BEGIN
//! vat's built-in Rust local-test emulators.
//!
//! Each emulator is a pure in-process server (no Java, no gcloud, no Docker),
//! run by the hidden `vat emulator` subcommand and reached by the runner through
//! the standard `*_EMULATOR_HOST` env var. Faithful enough for local tests of
//! the common client operations; the official emulators remain available as a
//! `runtime = docker`/`native` fidelity fallback.
//!
//! @spec projects/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#logic

pub mod auth;
pub mod pubsub;

use anyhow::Result;

/// Which built-in emulator to serve.
pub enum Kind {
    Pubsub,
    FirebaseAuth,
}

/// Serve the selected emulator on `host_port` (e.g. `127.0.0.1:8085`) until the
/// process is killed.
pub async fn serve(kind: Kind, host_port: &str) -> Result<()> {
    match kind {
        Kind::FirebaseAuth => auth::serve(host_port).await,
        Kind::Pubsub => pubsub::serve(host_port).await,
    }
}
// CODEGEN-END
