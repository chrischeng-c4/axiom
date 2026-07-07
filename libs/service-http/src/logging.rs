//! Tracing init: one shared `tracing-subscriber` registry built from
//! [`HttpConfig`].
//!
//! `RUST_LOG` wins; otherwise the filter falls back to `cfg.log_level`. The fmt
//! layer is `pretty` or `json` per `cfg.log_format`. This is the prefix-agnostic
//! version of the `init_tracing` each service binary hand-rolls today (lumen's
//! `init_tracing`, keep's inline `fmt().with_env_filter(...)`).

use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::config::{HttpConfig, LogFormat};

/// Install the global tracing subscriber from `cfg`.
///
/// Filter precedence: `RUST_LOG` (via `EnvFilter::try_from_default_env`) →
/// otherwise `cfg.log_level`. The fmt layer is JSON or pretty per
/// `cfg.log_format`.
///
/// Idempotency: installs the **global default** subscriber, so call this once
/// at startup. A second call returns an error (the global is already set).
///
/// OTLP: `cfg.otlp_endpoint` is honored as a stub only — see the
/// `// TODO(otlp)` below. Pulling the opentelemetry/otlp dep tree is a separate
/// follow-up; a service that needs trace export today keeps its own `otel`
/// feature-gated init (as lumen does) and uses this for the plain-log path.
pub fn init_tracing(cfg: &HttpConfig) -> anyhow::Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(cfg.log_level.clone()));

    let fmt_layer = match cfg.log_format {
        LogFormat::Pretty => tracing_subscriber::fmt::layer().boxed(),
        LogFormat::Json => tracing_subscriber::fmt::layer().json().boxed(),
    };

    // TODO(otlp): when `cfg.otlp_endpoint` is Some, attach an
    // `tracing-opentelemetry` layer exporting batch spans over OTLP/gRPC to the
    // endpoint. Deferred to keep the opentelemetry/otlp/tonic dep tree out of
    // this lib (it roughly doubles the build); a service that needs export today
    // keeps its own feature-gated init. We still note the request so a misconfig
    // is visible rather than silently ignored.
    if let Some(endpoint) = cfg.otlp_endpoint.as_deref() {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| anyhow::anyhow!("install tracing subscriber: {e}"))?;
        tracing::warn!(
            otlp_endpoint = endpoint,
            "otlp_endpoint set but service-http does not yet export traces \
             (TODO(otlp)); emitting plain logs only"
        );
        return Ok(());
    }

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| anyhow::anyhow!("install tracing subscriber: {e}"))?;
    Ok(())
}
