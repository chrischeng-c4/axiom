// HANDWRITE-BEGIN gap="missing-generator:logic:549646b3" tracker="pending-tracker" reason="Own logging, OTLP resolution, subscriber installation, and W3C extraction."
//! Structured logging and optional OTLP tracing without protocol ownership.

use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::{LogFormat, ObservabilityConfig, ServiceIdentity};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TracingMode {
    LoggingOnly,
    Otel {
        endpoint: String,
        identity: ServiceIdentity,
    },
    OtelUnavailable {
        endpoint: String,
        reason: OtelFallback,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OtelFallback {
    FeatureDisabled,
    InvalidEndpoint,
}

pub fn tracing_mode(config: &ObservabilityConfig, identity: &ServiceIdentity) -> TracingMode {
    let Some(endpoint) = config.otlp_endpoint.as_deref() else {
        return TracingMode::LoggingOnly;
    };
    if !valid_otlp_endpoint(endpoint) {
        return TracingMode::OtelUnavailable {
            endpoint: endpoint.to_string(),
            reason: OtelFallback::InvalidEndpoint,
        };
    }

    #[cfg(feature = "otlp")]
    {
        TracingMode::Otel {
            endpoint: endpoint.to_string(),
            identity: identity.clone(),
        }
    }
    #[cfg(not(feature = "otlp"))]
    {
        let _ = identity;
        TracingMode::OtelUnavailable {
            endpoint: endpoint.to_string(),
            reason: OtelFallback::FeatureDisabled,
        }
    }
}

fn valid_otlp_endpoint(endpoint: &str) -> bool {
    let Ok(uri) = endpoint.parse::<http::Uri>() else {
        return false;
    };
    matches!(uri.scheme_str(), Some("http") | Some("https")) && uri.authority().is_some()
}

pub fn init_tracing(config: &ObservabilityConfig) -> anyhow::Result<()> {
    let identity = ServiceIdentity::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))?;
    init_tracing_with_identity(config, &identity)
}

pub fn init_tracing_with_identity(
    config: &ObservabilityConfig,
    identity: &ServiceIdentity,
) -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.log_level.clone()));
    let fmt_layer = match config.log_format {
        LogFormat::Pretty => tracing_subscriber::fmt::layer().boxed(),
        LogFormat::Json => tracing_subscriber::fmt::layer().json().boxed(),
    };

    match tracing_mode(config, identity) {
        TracingMode::LoggingOnly => tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|error| anyhow::anyhow!("install tracing subscriber: {error}")),
        TracingMode::OtelUnavailable { reason, .. } => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .try_init()
                .map_err(|error| anyhow::anyhow!("install tracing subscriber: {error}"))?;
            tracing::warn!(
                ?reason,
                "OTLP tracing unavailable; emitting structured logs only"
            );
            Ok(())
        }
        #[cfg(feature = "otlp")]
        TracingMode::Otel { endpoint, identity } => match build_otel_tracer(&endpoint, &identity) {
            Ok(tracer) => {
                install_trace_context_propagator();
                tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt_layer)
                    .with(tracing_opentelemetry::layer().with_tracer(tracer))
                    .try_init()
                    .map_err(|error| anyhow::anyhow!("install tracing subscriber: {error}"))
            }
            Err(_error) => {
                tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt_layer)
                    .try_init()
                    .map_err(|error| anyhow::anyhow!("install tracing subscriber: {error}"))?;
                tracing::warn!(
                    reason = "exporter-construction",
                    "OTLP tracer construction failed; emitting structured logs only"
                );
                Ok(())
            }
        },
        #[cfg(not(feature = "otlp"))]
        TracingMode::Otel { .. } => unreachable!("OTLP mode requires the otlp feature"),
    }
}

#[cfg(feature = "otlp")]
fn build_otel_tracer(
    endpoint: &str,
    identity: &ServiceIdentity,
) -> Result<opentelemetry_sdk::trace::Tracer, Box<dyn std::error::Error>> {
    use opentelemetry_otlp::WithExportConfig;
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint.to_string());
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
            opentelemetry_sdk::Resource::new(vec![
                opentelemetry::KeyValue::new("service.name", identity.name().to_string()),
                opentelemetry::KeyValue::new("service.version", identity.version().to_string()),
            ]),
        ))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map_err(Into::into)
}

#[cfg(feature = "otlp")]
fn install_trace_context_propagator() {
    use std::sync::Once;
    static INSTALLED: Once = Once::new();
    INSTALLED.call_once(|| {
        opentelemetry::global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );
    });
}

#[cfg(feature = "otlp")]
pub fn extract_trace_context(headers: &http::HeaderMap) -> opentelemetry::Context {
    use opentelemetry::propagation::Extractor;
    install_trace_context_propagator();

    struct HeaderExtractor<'a>(&'a http::HeaderMap);
    impl Extractor for HeaderExtractor<'_> {
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).and_then(|value| value.to_str().ok())
        }

        fn keys(&self) -> Vec<&str> {
            self.0.keys().map(http::HeaderName::as_str).collect()
        }
    }

    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(headers))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(endpoint: Option<&str>) -> ObservabilityConfig {
        ObservabilityConfig::new("info", LogFormat::Json, endpoint.map(str::to_string))
    }

    #[test]
    fn logging_only_is_default() {
        let identity = ServiceIdentity::new("test", "0.1.0").unwrap();
        assert_eq!(
            tracing_mode(&config(None), &identity),
            TracingMode::LoggingOnly
        );
    }

    #[test]
    fn malformed_endpoint_falls_back() {
        let identity = ServiceIdentity::new("test", "0.1.0").unwrap();
        assert_eq!(
            tracing_mode(&config(Some("not-an-endpoint")), &identity),
            TracingMode::OtelUnavailable {
                endpoint: "not-an-endpoint".to_string(),
                reason: OtelFallback::InvalidEndpoint,
            }
        );
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn valid_endpoint_selects_otel_when_exporter_is_compiled() {
        let identity = ServiceIdentity::new("test", "0.1.0").unwrap();
        assert_eq!(
            tracing_mode(&config(Some("http://otel-collector:4317")), &identity),
            TracingMode::Otel {
                endpoint: "http://otel-collector:4317".to_string(),
                identity,
            }
        );
    }

    #[cfg(not(feature = "otlp"))]
    #[test]
    fn valid_endpoint_reports_disabled_exporter() {
        let identity = ServiceIdentity::new("test", "0.1.0").unwrap();
        assert_eq!(
            tracing_mode(&config(Some("http://otel-collector:4317")), &identity),
            TracingMode::OtelUnavailable {
                endpoint: "http://otel-collector:4317".to_string(),
                reason: OtelFallback::FeatureDisabled,
            }
        );
    }
}
// HANDWRITE-END
