---
id: projects-vat-src-emulator-mod-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/emulator/mod.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/emulator/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/src/emulator/mod.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
    /// The HTTP mock proxy needs a CA-pem path to write and a cassette dir, plus
    /// an optional seed of `(host, local base URL)` host-routing rules.
    HttpMock {
        ca_path: String,
        cassette_dir: String,
        routes: Vec<(String, String)>,
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
            routes,
        } => httpmock::serve(host_port, &ca_path, &cassette_dir, &routes).await,
        Kind::Openapi { spec } => openapi::serve(host_port, &spec).await,
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/emulator/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/emulator/mod.rs` captured during #39 vat standardization.
```
