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
pub mod dispatch;
pub mod grpc_mux;
pub mod httpmock;
pub mod openapi;
pub mod pubsub;
pub mod scheduler;
pub mod storage;
pub mod tasks;
pub mod workflows;

/// Generated googleapis gRPC types for the Cloud Tasks / Cloud Scheduler
/// emulator front-ends. The vendored proto tree (`proto/google/...`) carries the
/// shared `google.api` / `google.iam.v1` / `google.rpc` / `google.type` deps, so
/// the packages must be mounted in this exact nesting (prost cross-package
/// references climb `super::` to the `google` module). `google.protobuf.*` maps
/// to `::prost_types`.
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod googleapis {
    pub mod google {
        pub mod api { tonic::include_proto!("google.api"); }
        pub mod rpc { tonic::include_proto!("google.rpc"); }
        // `type` is a keyword, so tonic-build wrote `google.r#type.rs`; include it
        // by its real filename (`include_proto!` would look for `google.type.rs`).
        pub mod r#type { include!(concat!(env!("OUT_DIR"), "/google.r#type.rs")); }
        pub mod iam {
            pub mod v1 { tonic::include_proto!("google.iam.v1"); }
        }
        pub mod cloud {
            pub mod tasks {
                pub mod v2 { tonic::include_proto!("google.cloud.tasks.v2"); }
            }
            pub mod scheduler {
                pub mod v1 { tonic::include_proto!("google.cloud.scheduler.v1"); }
            }
        }
    }
}

use anyhow::Result;

/// Which built-in emulator to serve.
pub enum Kind {
    Pubsub,
    FirebaseAuth,
    CloudTasks,
    CloudScheduler,
    CloudWorkflows,
    CloudStorage,
    /// The HTTP mock proxy needs a CA-pem path to write and a cassette dir.
    HttpMock {
        ca_path: String,
        cassette_dir: String,
    },
    /// The OpenAPI mock serves responses from a spec document.
    Openapi {
        spec: String,
    },
}

/// Serve the selected emulator on `host_port` (e.g. `127.0.0.1:8085`) until the
/// process is killed.
pub async fn serve(kind: Kind, host_port: &str) -> Result<()> {
    match kind {
        Kind::FirebaseAuth => auth::serve(host_port).await,
        Kind::Pubsub => pubsub::serve(host_port).await,
        Kind::CloudTasks => tasks::serve(host_port).await,
        Kind::CloudScheduler => scheduler::serve(host_port).await,
        Kind::CloudWorkflows => workflows::serve(host_port).await,
        Kind::CloudStorage => storage::serve(host_port).await,
        Kind::HttpMock {
            ca_path,
            cassette_dir,
        } => httpmock::serve(host_port, &ca_path, &cassette_dir).await,
        Kind::Openapi { spec } => openapi::serve(host_port, &spec).await,
    }
}
// CODEGEN-END
