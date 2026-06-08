//! OpenTelemetry tracing for distributed task tracking

#[cfg(feature = "tracing-otel")]
use opentelemetry::KeyValue;
use tracing::{info_span, Span};

/// Create a span for task execution
#[cfg(feature = "tracing-otel")]
pub fn create_task_span(task_name: &str, task_id: &str, queue: &str) -> Span {
    info_span!(
        "task.execute",
        otel.name = %format!("task.{}", task_name),
        otel.kind = "CONSUMER",
        task.name = %task_name,
        task.id = %task_id,
        task.queue = %queue,
    )
}

/// Task execution attributes for tracing
#[derive(Debug, Clone)]
pub struct TaskSpanAttributes {
    pub task_name: String,
    pub task_id: String,
    pub queue: String,
    pub retry_count: u32,
    pub correlation_id: Option<String>,
}

impl TaskSpanAttributes {
    pub fn new(task_name: impl Into<String>, task_id: impl Into<String>, queue: impl Into<String>) -> Self {
        Self {
            task_name: task_name.into(),
            task_id: task_id.into(),
            queue: queue.into(),
            retry_count: 0,
            correlation_id: None,
        }
    }

    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
}

/// Initialize OpenTelemetry tracing
///
/// This sets up OTLP export to the specified endpoint (defaults to localhost:4317).
/// The tracer is installed globally and will be used for all subsequent spans.
#[cfg(feature = "tracing-otel")]
pub fn init_tracing(service_name: &str, otlp_endpoint: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::Resource;

    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION").to_string()),
    ]);

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(otlp_endpoint.unwrap_or("http://localhost:4317"));

    // Install the tracer provider globally
    // Note: install_batch returns a Tracer, but sets the global provider internally
    let _tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_resource(resource)
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    Ok(())
}

/// Shutdown tracing gracefully
#[cfg(feature = "tracing-otel")]
pub fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}
