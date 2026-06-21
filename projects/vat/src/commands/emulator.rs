// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-commands.md#schema
// CODEGEN-BEGIN
//! `vat emulator` — run one of vat's built-in Rust emulators.
//!
//! Internal: vat spawns *itself* as the service process for a built-in emulator
//! preset (`preset = "pubsub"` / `"firebase-auth"`), so this verb is hidden.
//! It builds a tokio runtime and serves until the process is killed (vat's
//! `stop_services` SIGKILLs it at teardown, like any service).

use std::process::ExitCode;

use anyhow::Result;

use crate::cli::EmulatorKind;

/// Run the selected built-in emulator bound to `host_port`.
/// @spec projects/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#cli
#[cfg(feature = "emulator")]
pub fn exec(
    kind: EmulatorKind,
    host_port: String,
    ca_path: Option<String>,
    cassette_dir: Option<String>,
    spec: Option<String>,
) -> Result<ExitCode> {
    let kind = match kind {
        EmulatorKind::Pubsub => crate::emulator::Kind::Pubsub,
        EmulatorKind::FirebaseAuth => crate::emulator::Kind::FirebaseAuth,
        EmulatorKind::CloudTasks => crate::emulator::Kind::CloudTasks,
        EmulatorKind::CloudScheduler => crate::emulator::Kind::CloudScheduler,
        EmulatorKind::CloudWorkflows => crate::emulator::Kind::CloudWorkflows,
        EmulatorKind::CloudStorage => crate::emulator::Kind::CloudStorage,
        EmulatorKind::HttpMock => crate::emulator::Kind::HttpMock {
            ca_path: ca_path.unwrap_or_else(|| "vat-http-mock-ca.pem".to_string()),
            cassette_dir: cassette_dir.unwrap_or_else(|| "vat-http-mock-cassettes".to_string()),
        },
        EmulatorKind::Openapi => crate::emulator::Kind::Openapi {
            spec: spec.unwrap_or_else(|| "openapi.yaml".to_string()),
        },
    };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(crate::emulator::serve(kind, &host_port))?;
    Ok(ExitCode::SUCCESS)
}

/// Lean build (no `emulator` feature): the verb is present but inert.
/// @spec projects/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#cli
#[cfg(not(feature = "emulator"))]
pub fn exec(
    _kind: EmulatorKind,
    _host_port: String,
    _ca_path: Option<String>,
    _cassette_dir: Option<String>,
    _spec: Option<String>,
) -> Result<ExitCode> {
    anyhow::bail!(
        "this vat was built without the `emulator` feature; rebuild with default features to use `vat emulator`"
    );
}
// CODEGEN-END
