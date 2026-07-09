// SPEC-MANAGED: apps/courier/tech-design/interfaces/rest/github-issues-proxy.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:c0ur1e06" tracker="pending-tracker" reason="axum h2c app over the courier GithubClient: AppState (draining flag + verifier + GithubClient), router() composing the shared probe routes with the /v1 data plane behind the shared bearer-auth middleware, mirroring apps/relay/src/server.rs's shape."
//! axum HTTP/2 (h2c) application: the GitHub-issues-proxy data plane.
//!
//! The operational surface is the shared service shell: the standard probe
//! routes (`/healthz` `/readyz` `/metrics` `/openapi.json` `/docs`) come from
//! `service_http::standard_probe_routes` merged with the `/v1` data plane;
//! error responses render the shared `{error, message}` envelope
//! ([`service_http::ApiErr`]).
//!
//! Request auth is the shared `libs/service-auth` bearer contract
//! ([`auth`]): the blanket `service_auth::auth_middleware` runs on the `/v1`
//! data plane ONLY (probes stay tokenless), injecting a
//! [`service_auth::RoleMapPrincipal`] each handler authorizes on its
//! `{owner}/{name}` via [`auth::authorize`].
//!
//! courier has no state of its own (#1's "stateless GitHub proxy" framing):
//! [`AppState`] holds only the drain flag, the bearer verifier, and the
//! [`github::GithubClient`] that forwards to `api.github.com`.

pub mod auth;
pub mod github;
pub mod openapi;
pub mod routes;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use axum::Router;
use service_auth::StaticRoleMapVerifier;
use service_http::MetricsProvider;

use crate::http::github::GithubClient;

/// Shared application state: the drain flag `/readyz` reports, the bearer
/// verifier the data-plane auth layer runs, and the GitHub client every
/// handler forwards through.
#[derive(Clone)]
pub struct AppState {
    github: Arc<GithubClient>,
    draining: Arc<AtomicBool>,
    verifier: Arc<StaticRoleMapVerifier>,
}

impl AppState {
    /// Build state from a resolved [`github::GithubClient`] and
    /// [`auth::AuthConfig`]: the data-plane auth layer runs the registry
    /// verifier when auth is required, the open verifier when off (the
    /// `COURIER_AUTH=off` default).
    pub fn new(github: GithubClient, auth: auth::AuthConfig) -> Self {
        AppState {
            github: Arc::new(github),
            draining: Arc::new(AtomicBool::new(false)),
            verifier: Arc::new(auth.verifier()),
        }
    }

    pub fn github(&self) -> &GithubClient {
        &self.github
    }

    fn verifier(&self) -> Arc<StaticRoleMapVerifier> {
        Arc::clone(&self.verifier)
    }

    /// Flip readiness to draining so `/readyz` returns 503. Called on
    /// SIGTERM via `service_http::shutdown_with_drain`.
    pub fn start_drain(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }
}

/// Readiness source for the shared probe router: `/readyz` reports 503 once
/// SIGTERM flips `start_drain`.
impl service_http::ReadinessHook for AppState {
    fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }
}

/// Build the HTTP/2 router for the GitHub-issues-proxy data plane.
///
/// @spec apps/courier/tech-design/interfaces/rest/github-issues-proxy.md#logic
pub fn router(state: AppState) -> Router {
    let verifier = state.verifier();
    let data_plane = Router::new()
        .route(
            "/v1/issues/{owner}/{name}",
            get(routes::search_issues).post(routes::create_issue),
        )
        .route(
            "/v1/issues/{owner}/{name}/{number}",
            get(routes::view_issue),
        )
        .route(
            "/v1/issues/{owner}/{name}/{number}/comments",
            post(routes::comment_issue),
        )
        // Shared bearer auth on the data plane ONLY — probes stay tokenless.
        // The blanket middleware authenticates (401 on a missing/unknown
        // token when required) and injects the RoleMapPrincipal each
        // handler authorizes on its {owner}/{name}.
        .route_layer(from_fn_with_state(
            verifier,
            service_auth::auth_middleware::<StaticRoleMapVerifier>,
        ))
        .with_state(state.clone());

    // Standard probes (`/healthz`, `/readyz`, `/metrics`, `/openapi.json`,
    // `/docs`) come from the shared service shell. courier tracks no request
    // metrics of its own today, so the metrics slot is `None`; AppState
    // still supplies readiness.
    let probe_state = Arc::new(state);
    let metrics: Option<Arc<dyn MetricsProvider>> = None;
    let probes = service_http::standard_probe_routes(probe_state, metrics, openapi::openapi);

    probes
        .merge(data_plane)
        // One INFO-level tracing span per request — spans probes + data plane.
        .layer(service_http::trace_layer())
}
// HANDWRITE-END
