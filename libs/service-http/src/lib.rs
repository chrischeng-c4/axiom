// CODEGEN-BEGIN
//! `service-http` — shared HTTP-service scaffolding for the ecosystem's
//! k8s-native services.
//!
//! lumen, keep, relay, and loom compose the same HTTP policy shell: the
//! standard probe/admin endpoints (`/healthz` `/readyz` `/metrics`
//! `/openapi.json` `/docs`), observability compatibility adapters, lifecycle
//! readiness/shutdown re-exports, runtime delegation, per-request
//! `Server-Timing` attribution ([`server_timing`]), the shared
//! request-body byte cap ([`body_limit`]), and the
//! `{"error", "message"}` HTTP error envelope
//! ([`error`]) each service renders for its error responses. This crate is
//! the one place that HTTP shape lives. Protocol-neutral logging, tracing,
//! metric-provider, and lifecycle metric ownership belongs to
//! `service-observability`. This crate operationalizes the CONTRIBUTING "standard
//! endpoints" convention: every service exposes the same probe surface,
//! with the same auth-exempt / no-body-limit treatment.
//!
//! ## Composition
//!
//! It composes, it does not replace: [`transport::serve`] delegates listener
//! ownership to `server-http`; [`probes::standard_probe_routes`] returns an `axum::Router`
//! a service `.merge`s its own (auth'd, body-limited) data plane onto —
//! [`body_limit::body_limit_layer`] is the body-limiting piece of that data
//! plane (see its module docs for placement and the recommended default).
//!
//! ## What a service wires
//!
//! ```ignore
//! use std::sync::Arc;
//! use std::time::Duration;
//! use service_http::{
//!     HttpConfig, init_tracing, lifecycle_probe_routes, serve_with_lifecycle,
//!     LifecycleShutdownTrigger, run_signal_bridge, server_timing_middleware,
//!     trace_layer,
//! };
//!
//! # async fn run(cfg: HttpConfig, data_plane: axum::Router) -> anyhow::Result<()>
//! # {
//! init_tracing(&cfg)?;
//! let lifecycle = server_lifecycle::LifecycleController::new();
//! lifecycle.transition(server_lifecycle::LifecyclePhase::Serving, "ready", "startup complete")?;
//! let app = lifecycle_probe_routes(lifecycle.clone(), None, my_service::openapi)
//!     .merge(data_plane)
//!     .layer(trace_layer())
//!     .layer(axum::middleware::from_fn(server_timing_middleware));
//!
//! let listener = tokio::net::TcpListener::bind(cfg.bind_addr()).await?;
//! let grace = Duration::from_secs(cfg.grace_secs);
//! let trigger = LifecycleShutdownTrigger::new(lifecycle.clone(), grace, std::time::Duration::ZERO)?;
//! let signal_task = tokio::spawn(run_signal_bridge(trigger.clone(), async {
//!     tokio::signal::ctrl_c().await.expect("signal");
//! }));
//! let report = serve_with_lifecycle(
//!     listener,
//!     app,
//!     server_http::HttpServerOptions::default(),
//!     lifecycle.clone(),
//! );
//! let (_http_report, _shutdown_report) = tokio::join!(report, signal_task);
//! # Ok(()) }
//! # fn openapi() -> utoipa::openapi::OpenApi { unimplemented!() }
//! ```
//!
//! ## Scope
//!
//! Auth and backup are deliberately out of scope (separate follow-ups); a
//! service keeps owning those on its data plane. OTLP trace export is optional
//! behind the `otlp` feature; a service supplies its own stable identity.
//! [`server_timing_middleware`] always renders the `app;dur=` baseline and
//! defaults every response to [`ServerTimingDisclosure::TotalOnly`] — this
//! crate cannot see a request's auth outcome (see [`server_timing`] for why)
//! so it does not attempt to gate the phase breakdown on it; a service opts
//! a response into [`ServerTimingDisclosure::Full`] itself.
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

pub mod admission;
pub mod body_limit;
pub mod config;
pub mod error;
pub mod logging;
pub mod metrics;
pub mod probes;
pub mod readiness;
pub mod server_timing;
pub mod signal;
pub mod transport;

pub use admission::{
    admission_middleware, AdmissionConfig, AdmissionConfigError, AdmissionController,
    AdmissionDecision, AdmissionEvent, AdmissionInput, AdmissionMiddleware, AdmissionObserver,
    AdmissionOutcome, AdmissionPolicy, AdmissionPolicyError, NoopAdmissionObserver,
};
pub use body_limit::{body_limit_layer, BodyLimitLayer, BodyLimitService};
pub use config::{HttpConfig, LogFormat, ServiceIdentity};
pub use error::{ApiErr, ErrorEnvelope};
#[cfg(feature = "otlp")]
pub use logging::extract_trace_context;
pub use logging::{
    init_tracing, init_tracing_with_identity, tracing_mode, OtelFallback, TracingMode,
};
pub use metrics::MetricsProvider;
pub use probes::{
    lifecycle_probe_routes, lifecycle_probe_routes_canonical_json, standard_probe_routes,
    standard_probe_routes_canonical_json,
};
pub use readiness::ReadinessHook;
/// Re-exported so a service can build a [`serve_tls`] configuration source
/// without depending on `server-http` directly (#3113 R1).
pub use server_http::{config_source, ServerConfigSource};
pub use server_timing::{server_timing_middleware, ServerTimingDisclosure, ServerTimingExt};
pub use service_observability::LifecycleMetrics;
pub use signal::{
    run_signal_bridge, shutdown_on_signal, shutdown_with_drain, wait_shutdown_signal,
    LifecycleShutdownTrigger,
};
pub use transport::{serve, serve_tls, serve_with_lifecycle, trace_layer, PropagatingMakeSpan};
// CODEGEN-END
