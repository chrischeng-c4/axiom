// SPEC-MANAGED: libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `HttpConfig` — the env-driven runtime knobs every k8s-native service shares.
//!
//! Each service binary already parses these (under its own `SERVICE_*`
//! prefix via clap `env =`); this is the resolved, prefix-agnostic struct the
//! shared scaffolding (`logging::init_tracing`, `signal::shutdown_with_drain`,
//! the body-limit on the data-plane router) reads, so the common shape lives in
//! one place instead of being threaded through four hand-rolled `serve()` fns.

/// Log output format for the fmt layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#source
pub enum LogFormat {
    /// Human/agent-readable multi-line output (local dev default).
    Pretty,
    /// One JSON object per line (structured log shipping in-cluster).
    Json,
}

/// Stable resource identity attached to optional shared trace export.
#[derive(Clone, Debug, PartialEq, Eq)]
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#source
pub struct ServiceIdentity {
    name: String,
    version: String,
}

/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#source
impl ServiceIdentity {
    /// Create an identity for the `service.name` and `service.version`
    /// resource attributes. Empty fields are rejected before startup.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> anyhow::Result<Self> {
        let name = name.into();
        let version = version.into();
        if name.trim().is_empty() {
            anyhow::bail!("service tracing identity name must not be empty");
        }
        if version.trim().is_empty() {
            anyhow::bail!("service tracing identity version must not be empty");
        }
        Ok(Self { name, version })
    }

    /// Stable service name supplied by the owning application.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Stable service version supplied by the owning application.
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Resolved HTTP-service configuration.
///
/// Built by a service binary from its flags/env and handed to the shared
/// scaffolding. Construct with [`HttpConfig::new`].
#[derive(Clone, Debug)]
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#source
pub struct HttpConfig {
    /// Bind host. k8s passes `0.0.0.0`; local dev defaults to `127.0.0.1`.
    pub host: String,
    /// Bind port.
    pub port: u16,
    /// Base log level (`trace|debug|info|warn|error`). `RUST_LOG` still wins.
    pub log_level: String,
    /// Log output format.
    pub log_format: LogFormat,
    /// Graceful-drain window (seconds) held after SIGTERM before the listener
    /// closes, so k8s stops routing while `/readyz` reports 503.
    pub grace_secs: u64,
    /// Max request body size (bytes) for the data plane. The probe routes carry
    /// no body limit regardless.
    pub body_limit_bytes: usize,
    /// OTLP gRPC endpoint for trace export, e.g. `http://otel-collector:4317`.
    /// Opt-in: when `None`, no OTLP wiring is attempted. See
    /// [`crate::logging::init_tracing`] for the current stub status.
    pub otlp_endpoint: Option<String>,
}

/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#source
impl HttpConfig {
    /// Construct a config from already-resolved values. Every field is explicit
    /// so a service binary maps its own flags/env in one place.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        host: impl Into<String>,
        port: u16,
        log_level: impl Into<String>,
        log_format: LogFormat,
        grace_secs: u64,
        body_limit_bytes: usize,
        otlp_endpoint: Option<String>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            log_level: log_level.into(),
            log_format,
            grace_secs,
            body_limit_bytes,
            otlp_endpoint,
        }
    }

    /// `host:port` bind string for `TcpListener::bind`.
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_populates_every_field() {
        let cfg = HttpConfig::new(
            "0.0.0.0",
            7373,
            "debug",
            LogFormat::Json,
            45,
            8 * 1024 * 1024,
            Some("http://otel:4317".to_string()),
        );
        assert_eq!(cfg.host, "0.0.0.0");
        assert_eq!(cfg.port, 7373);
        assert_eq!(cfg.log_level, "debug");
        assert_eq!(cfg.log_format, LogFormat::Json);
        assert_eq!(cfg.grace_secs, 45);
        assert_eq!(cfg.body_limit_bytes, 8 * 1024 * 1024);
        assert_eq!(cfg.otlp_endpoint.as_deref(), Some("http://otel:4317"));
        assert_eq!(cfg.bind_addr(), "0.0.0.0:7373");
    }

    #[test]
    fn service_identity_rejects_blank_fields() {
        assert!(ServiceIdentity::new("", "0.1.0").is_err());
        assert!(ServiceIdentity::new("service", " ").is_err());
        assert_eq!(
            ServiceIdentity::new("service", "0.1.0").unwrap(),
            ServiceIdentity {
                name: "service".to_string(),
                version: "0.1.0".to_string(),
            }
        );
    }
}
// CODEGEN-END
// SPEC-MANAGED: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#logic
// CODEGEN-BEGIN
pub fn configure() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Decision: Is OTLP requested with a valid absolute HTTP(S) endpoint and compiled exporter support?
    if todo!("decision: Is OTLP requested with a valid absolute HTTP(S) endpoint and compiled exporter support?") /* branch */ {
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-logging
        // TODO: Implement process step: Install one RUST_LOG-first pretty or JSON subscriber
        todo!("process: Install one RUST_LOG-first pretty or JSON subscriber");
    } else if todo!("decision branch: {}", "branch") { /* branch */
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-exporter
        // TODO: Implement process step: Attach stable service.name and service.version resources and W3C propagator
        todo!("process: Attach stable service.name and service.version resources and W3C propagator");
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_adapter
        // TODO: Implement process step: service-http extracts request headers and serves provider bytes without owning protocol-neutral state
        todo!("process: service-http extracts request headers and serves provider bytes without owning protocol-neutral state");
        todo!("terminal: Existing service-http names remain additive compatibility re-exports");
    } else { /* branch */
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-fallback
        // TODO: Implement process step: Install logging-only subscriber and emit a redacted fallback reason
        todo!("process: Install logging-only subscriber and emit a redacted fallback reason");
    }
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-provider
    // TODO: Implement process step: MetricsProvider returns canonical Prometheus exposition bytes
    todo!("process: MetricsProvider returns canonical Prometheus exposition bytes");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-connection
    // TODO: Implement process step: LifecycleMetrics implements ConnectionMetrics using metrics-prometheus counters
    todo!("process: LifecycleMetrics implements ConnectionMetrics using metrics-prometheus counters");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_adapter
    // TODO: Implement process step: service-http extracts request headers and serves provider bytes without owning protocol-neutral state
    todo!("process: service-http extracts request headers and serves provider bytes without owning protocol-neutral state");
    todo!("terminal: Existing service-http names remain additive compatibility re-exports");
    todo!("terminal: Raw TCP and future protocol runtimes consume service-observability directly");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_config
    // TODO: Implement process step: service-http HttpConfig projects only its observability fields into ObservabilityConfig
    todo!("process: service-http HttpConfig projects only its observability fields into ObservabilityConfig");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_adapter
    // TODO: Implement process step: service-http extracts request headers and serves provider bytes without owning protocol-neutral state
    todo!("process: service-http extracts request headers and serves provider bytes without owning protocol-neutral state");
    todo!("terminal: Existing service-http names remain additive compatibility re-exports");
    // Terminal: compatible -> Existing service-http names remain additive compatibility re-exports
    // Terminal: non_http -> Raw TCP and future protocol runtimes consume service-observability directly
}
// CODEGEN-END
