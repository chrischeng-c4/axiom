// SPEC-MANAGED: libs/service-http/tech-design/semantic/source/libs-service-http-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `service-http` — shared HTTP-service scaffolding for the ecosystem's
//! k8s-native services.
//!
//! lumen, keep, relay, and loom each hand-roll the same service shell today: the
//! standard probe/admin endpoints (`/healthz` `/readyz` `/metrics`
//! `/openapi.json` `/docs`), env-driven `tracing` init, a SIGTERM-aware
//! graceful-drain shutdown, the h2c serve loop (HTTP/1.1 + HTTP/2 cleartext
//! on one port), and the `{"error", "message"}` HTTP error envelope
//! ([`error`]) each service renders for its error responses. This crate is
//! the one place that shape lives — the 6th service-kit lib, after `h2c`
//! (transport), `cli-std` (the `llm`/`upgrade`/`issue` CLI convention),
//! `raft-core` + `raft-host` (replication), and `operator` (the k8s
//! reconcile scaffold). It operationalizes the CONTRIBUTING "standard
//! endpoints" convention: every service exposes the same probe surface,
//! with the same auth-exempt / no-body-limit treatment.
//!
//! ## Composition
//!
//! It composes, it does not replace: [`transport::serve`] delegates to
//! [`h2c::serve`]; [`probes::standard_probe_routes`] returns an `axum::Router`
//! a service `.merge`s its own (auth'd, body-limited) data plane onto.
//!
//! ## What a service wires
//!
//! ```ignore
//! use std::sync::Arc;
//! use std::time::Duration;
//! use service_http::{
//!     HttpConfig, LogFormat, MetricsProvider, ReadinessHook,
//!     init_tracing, serve, shutdown_with_drain, standard_probe_routes, trace_layer,
//! };
//!
//! # async fn run(cfg: HttpConfig, readiness: Arc<R>, data_plane: axum::Router) -> anyhow::Result<()>
//! # where R: ReadinessHook + 'static {
//! init_tracing(&cfg)?;
//!
//! let app = standard_probe_routes(readiness.clone(), None, my_service::openapi)
//!     .merge(data_plane)
//!     .layer(trace_layer());
//!
//! let listener = tokio::net::TcpListener::bind(cfg.bind_addr()).await?;
//! let grace = Duration::from_secs(cfg.grace_secs);
//! serve(
//!     listener,
//!     app,
//!     shutdown_with_drain(move || readiness.start_drain(), grace),
//! )
//! .await;
//! # Ok(()) }
//! # fn openapi() -> utoipa::openapi::OpenApi { unimplemented!() }
//! ```
//!
//! ## Scope
//!
//! Auth and backup are deliberately out of scope (separate follow-ups); a
//! service keeps owning those on its data plane. OTLP trace export is a stubbed
//! `// TODO(otlp)` in [`logging`] — the dep tree is deferred.
//!
//! [`error::ErrorEnvelope`]'s derived `utoipa::ToSchema` is named
//! `ErrorEnvelope` in a service's generated OpenAPI document. A service that
//! already published a different schema name for this shape (e.g. lumen's
//! established `ApiError`) keeps its own local doc-only struct of the same
//! `{error, message}` shape for that name/description rather than aliasing
//! this one — `#[derive(ToSchema)]` fixes both the schema name and its
//! `description` (sourced from the struct's doc comment) at the type's own
//! definition site, so neither a `pub type` alias nor a `#[schema(as =
//! ...)]` override on this shared struct can reproduce a *different*
//! consuming service's pre-existing name and doc-comment-derived
//! description without baking that service's spec-path text into this
//! generic crate. [`error::ApiErr`] (the runtime status/kind/message
//! wrapper — it carries no `ToSchema`) has no such constraint and is meant
//! to be adopted directly.

pub mod config;
pub mod error;
pub mod logging;
pub mod metrics;
pub mod probes;
pub mod readiness;
pub mod signal;
pub mod transport;

pub use config::{HttpConfig, LogFormat};
pub use error::{ApiErr, ErrorEnvelope};
pub use logging::init_tracing;
pub use metrics::MetricsProvider;
pub use probes::standard_probe_routes;
pub use readiness::ReadinessHook;
pub use signal::{shutdown_with_drain, wait_shutdown_signal};
pub use transport::{serve, trace_layer};
// CODEGEN-END
