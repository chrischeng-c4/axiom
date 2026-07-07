// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-commands-emulator-rs.md#rust-source-unit
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
    route: Vec<String>,
    no_forward: bool,
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
            routes: parse_routes(&route),
            forward: !no_forward,
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

/// Parse repeatable `--route host=base` flags into `(host, base URL)` pairs.
/// Entries without a `=` (or with an empty side) are skipped.
#[cfg(feature = "emulator")]
fn parse_routes(routes: &[String]) -> Vec<(String, String)> {
    routes
        .iter()
        .filter_map(|r| {
            let (host, base) = r.split_once('=')?;
            let (host, base) = (host.trim(), base.trim());
            (!host.is_empty() && !base.is_empty()).then(|| (host.to_string(), base.to_string()))
        })
        .collect()
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
    _route: Vec<String>,
    _no_forward: bool,
) -> Result<ExitCode> {
    anyhow::bail!(
        "this vat was built without the `emulator` feature; rebuild with default features to use `vat emulator`"
    );
}

#[cfg(all(test, feature = "emulator"))]
mod tests {
    use super::parse_routes;

    #[test]
    fn parse_routes_splits_host_eq_base_and_skips_malformed() {
        let routes = parse_routes(&[
            "cloudtasks.googleapis.com=http://127.0.0.1:8085".to_string(),
            " example.test = http://127.0.0.1:9000 ".to_string(), // trimmed
            "no-equals".to_string(),                              // skipped
            "=http://x".to_string(),                              // empty host → skipped
            "host=".to_string(),                                  // empty base → skipped
        ]);
        assert_eq!(
            routes,
            vec![
                (
                    "cloudtasks.googleapis.com".to_string(),
                    "http://127.0.0.1:8085".to_string()
                ),
                (
                    "example.test".to_string(),
                    "http://127.0.0.1:9000".to_string()
                ),
            ]
        );
    }
}
// CODEGEN-END
