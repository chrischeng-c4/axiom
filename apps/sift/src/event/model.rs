// HANDWRITE-BEGIN gap="sift-operational-event-v2-model" tracker="1657" reason="Define the phase-one operational-event model and validated wire decode."
use std::collections::BTreeMap;
use std::collections::HashSet;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

pub const EVENT_SCHEMA_VERSION: u16 = 2;
pub const EVENT_SCHEMA_URL: &str = "https://cclab.dev/sift/schemas/operational-event/v2";

/// Phase-one signal kinds. Other SRE data is added only through a later
/// versioned public contract.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToSchema,
    ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    Log,
    Span,
    Metric,
}

impl SignalKind {
    pub const ALL: [Self; 3] = [Self::Log, Self::Metric, Self::Span];
}

impl std::fmt::Display for SignalKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Log => "log",
            Self::Span => "span",
            Self::Metric => "metric",
        })
    }
}

/// JSON representation of OpenTelemetry AnyValue semantics. Bytes are a
/// base64 string so JSON/protobuf transports preserve the distinction from a
/// normal string and from an array of integers.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum AttributeValue {
    String(String),
    Bool(bool),
    Int(i64),
    Double(f64),
    Bytes(String),
    Array(Vec<AttributeValue>),
    Map(BTreeMap<String, AttributeValue>),
}

impl AttributeValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    fn validate(&self) -> Result<()> {
        match self {
            Self::Double(value) if !value.is_finite() => {
                bail!("attribute double values must be finite")
            }
            Self::Array(values) => {
                for value in values {
                    value.validate()?;
                }
            }
            Self::Map(values) => {
                for (key, value) in values {
                    if key.trim().is_empty() {
                        bail!("attribute map keys must not be empty");
                    }
                    value.validate()?;
                }
            }
            Self::String(_) | Self::Bool(_) | Self::Int(_) | Self::Double(_) | Self::Bytes(_) => {}
        }
        Ok(())
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct InstrumentationScope {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, AttributeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_url: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MetricTemporality {
    Delta,
    Cumulative,
    Gauge,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct MetricExemplar {
    pub value: f64,
    pub trace_id: String,
    pub span_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct MetricPoint {
    pub name: String,
    pub value: f64,
    /// Prometheus Remote Write uses a reserved NaN bit pattern to mark a
    /// series stale. Sift stores that state explicitly so durable JSON never
    /// contains a non-finite number.
    #[serde(default, skip_serializing_if = "is_false")]
    pub stale: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub temporality: MetricTemporality,
    #[serde(default)]
    pub exemplars: Vec<MetricExemplar>,
}

/// Durable content-addressed reference for a payload externalized before the
/// canonical raw event is acknowledged.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct ContentBlobRef {
    pub hash: String,
    pub size: u64,
    pub encoding: String,
}

/// The only event shape serialized by new journal writers.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct OperationalEventV2 {
    pub schema_version: u16,
    pub schema_url: String,
    pub event_id: String,
    pub project: String,
    pub environment: String,
    pub occurred_at: String,
    pub observed_at: String,
    pub signal: SignalKind,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub resource: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instrumentation_scope: Option<InstrumentationScope>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, AttributeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric: Option<MetricPoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blob_refs: Vec<ContentBlobRef>,
    #[schema(value_type = Object)]
    pub payload: Value,
}

impl OperationalEventV2 {
    /// Compatibility constructor for bootstrap callers. Ingest producers
    /// should use [`for_project`](Self::for_project) with explicit tenancy.
    pub fn new(event_id: impl Into<String>, signal: SignalKind, payload: Value) -> Self {
        Self::for_project("default", "default", event_id, signal, payload)
    }

    pub fn for_project(
        project: impl Into<String>,
        environment: impl Into<String>,
        event_id: impl Into<String>,
        signal: SignalKind,
        payload: Value,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            schema_version: EVENT_SCHEMA_VERSION,
            schema_url: EVENT_SCHEMA_URL.to_string(),
            event_id: event_id.into(),
            project: project.into(),
            environment: environment.into(),
            occurred_at: now.clone(),
            observed_at: now,
            signal,
            resource: BTreeMap::new(),
            instrumentation_scope: None,
            attributes: BTreeMap::new(),
            trace_id: None,
            span_id: None,
            request_id: None,
            session_id: None,
            severity: None,
            metric: None,
            blob_refs: Vec::new(),
            payload,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != EVENT_SCHEMA_VERSION {
            bail!(
                "unsupported schema_version {}; expected {}",
                self.schema_version,
                EVENT_SCHEMA_VERSION
            );
        }
        for (name, value) in [
            ("schema_url", self.schema_url.as_str()),
            ("event_id", self.event_id.as_str()),
            ("project", self.project.as_str()),
            ("environment", self.environment.as_str()),
            ("occurred_at", self.occurred_at.as_str()),
            ("observed_at", self.observed_at.as_str()),
        ] {
            if value.trim().is_empty() {
                bail!("{name} must not be empty");
            }
        }
        DateTime::parse_from_rfc3339(&self.occurred_at).context("occurred_at must be RFC3339")?;
        DateTime::parse_from_rfc3339(&self.observed_at).context("observed_at must be RFC3339")?;
        if self.resource.is_empty() {
            bail!("resource must contain at least one stable identity field");
        }
        if self.payload.is_null() {
            bail!("payload must not be null");
        }
        if let Some(scope) = &self.instrumentation_scope {
            if scope.name.trim().is_empty() {
                bail!("instrumentation_scope.name must not be empty");
            }
            for value in scope.attributes.values() {
                value.validate()?;
            }
        }
        for (key, value) in &self.attributes {
            if key.trim().is_empty() {
                bail!("attribute keys must not be empty");
            }
            value.validate()?;
        }
        match (self.signal, &self.metric) {
            (SignalKind::Metric, Some(metric)) => validate_metric(metric)?,
            (SignalKind::Metric, None) => bail!("metric signals require a direct metric point"),
            (_, Some(_)) => bail!("only metric signals may contain a metric point"),
            (_, None) => {}
        }
        let mut blob_hashes = HashSet::new();
        for blob in &self.blob_refs {
            let digest = blob
                .hash
                .strip_prefix("sha256:")
                .context("blob hash must use the sha256:<hex> form")?;
            if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                bail!("blob hash must contain a 64-character SHA-256 digest");
            }
            if blob.size == 0 {
                bail!("blob size must be greater than zero");
            }
            if blob.encoding.trim().is_empty() {
                bail!("blob encoding must not be empty");
            }
            if !blob_hashes.insert(blob.hash.as_str()) {
                bail!("blob references must not contain duplicate hashes");
            }
        }
        Ok(())
    }
}

fn validate_metric(metric: &MetricPoint) -> Result<()> {
    if metric.name.trim().is_empty() || !metric.value.is_finite() {
        bail!("metric name must be non-empty and value must be finite");
    }
    if metric.stale && metric.value != 0.0 {
        bail!("stale metric points must use a zero storage value");
    }
    for exemplar in &metric.exemplars {
        if !exemplar.value.is_finite()
            || exemplar.trace_id.trim().is_empty()
            || exemplar.span_id.trim().is_empty()
        {
            bail!("metric exemplars require finite value, trace_id, and span_id");
        }
    }
    Ok(())
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// Deserializes the current event shape and validates it before use.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct IncomingEvent(pub OperationalEventV2);

impl IncomingEvent {
    pub fn into_inner(self) -> OperationalEventV2 {
        self.0
    }
}

impl<'de> Deserialize<'de> for IncomingEvent {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        decode_event_value(value)
            .map(Self)
            .map_err(D::Error::custom)
    }
}

pub fn decode_event_json(bytes: &[u8]) -> Result<OperationalEventV2> {
    serde_json::from_slice::<IncomingEvent>(bytes)
        .map(IncomingEvent::into_inner)
        .map_err(|error| anyhow::anyhow!("decode operational event: {error}"))
}

fn decode_event_value(value: Value) -> Result<OperationalEventV2> {
    let version = value
        .get("schema_version")
        .and_then(Value::as_u64)
        .context("operational event requires integer schema_version")?;
    let version = u16::try_from(version).context("schema_version exceeds u16")?;
    if version != EVENT_SCHEMA_VERSION {
        bail!("unsupported schema_version {version}");
    }
    let event = serde_json::from_value::<OperationalEventV2>(value)?;
    event.validate()?;
    Ok(event)
}

// HANDWRITE-END
