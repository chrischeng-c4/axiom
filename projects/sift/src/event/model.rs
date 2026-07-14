// HANDWRITE-BEGIN gap="sift-operational-event-v2-model" tracker="1657" reason="Define OperationalEventV2, typed attributes, eight signals, v1 wire shape, incoming compatibility decode, and deterministic upcast."
use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

pub const EVENT_SCHEMA_VERSION_V1: u16 = 1;
pub const EVENT_SCHEMA_VERSION: u16 = 2;
pub const EVENT_SCHEMA_URL: &str =
    "https://cclab.dev/sift/schemas/operational-event/v2";

/// Canonical signal kinds. GenAI generation/tool/RAG observations are span
/// specializations; sessions are correlation groups rather than a signal.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, ToSchema, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    Log,
    Span,
    Metric,
    Exception,
    AuditEvent,
    ChangeEvent,
    Profile,
    Evaluation,
}

impl SignalKind {
    pub const ALL: [Self; 8] = [
        Self::Log,
        Self::Span,
        Self::Metric,
        Self::Exception,
        Self::AuditEvent,
        Self::ChangeEvent,
        Self::Profile,
        Self::Evaluation,
    ];

    fn existed_in_v1(self) -> bool {
        !matches!(self, Self::Profile | Self::Evaluation)
    }
}

impl std::fmt::Display for SignalKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Log => "log",
            Self::Span => "span",
            Self::Metric => "metric",
            Self::Exception => "exception",
            Self::AuditEvent => "audit_event",
            Self::ChangeEvent => "change_event",
            Self::Profile => "profile",
            Self::Evaluation => "evaluation",
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
            Self::String(_)
            | Self::Bool(_)
            | Self::Int(_)
            | Self::Double(_)
            | Self::Bytes(_) => {}
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub temporality: MetricTemporality,
    #[serde(default)]
    pub exemplars: Vec<MetricExemplar>,
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
        DateTime::parse_from_rfc3339(&self.occurred_at)
            .context("occurred_at must be RFC3339")?;
        DateTime::parse_from_rfc3339(&self.observed_at)
            .context("observed_at must be RFC3339")?;
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
        Ok(())
    }
}

fn validate_metric(metric: &MetricPoint) -> Result<()> {
    if metric.name.trim().is_empty() || !metric.value.is_finite() {
        bail!("metric name must be non-empty and value must be finite");
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

/// Exact bootstrap wire shape retained only for journal/snapshot/raft upcast.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EventEnvelopeV1 {
    pub schema_version: u16,
    pub event_id: String,
    pub occurred_at: String,
    pub signal: SignalKind,
    #[serde(default)]
    pub resource: BTreeMap<String, String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric: Option<MetricPoint>,
    pub payload: Value,
}

impl TryFrom<EventEnvelopeV1> for OperationalEventV2 {
    type Error = anyhow::Error;

    fn try_from(event: EventEnvelopeV1) -> Result<Self> {
        if event.schema_version != EVENT_SCHEMA_VERSION_V1 {
            bail!(
                "unsupported legacy schema_version {}; expected {}",
                event.schema_version,
                EVENT_SCHEMA_VERSION_V1
            );
        }
        if !event.signal.existed_in_v1() {
            bail!("legacy schema v1 did not support signal {}", event.signal);
        }
        let project = first_resource(
            &event.resource,
            &["gcp.project_id", "project.id", "cloud.account.id"],
        )
        .unwrap_or("legacy")
        .to_string();
        let environment = first_resource(
            &event.resource,
            &["deployment.environment.name", "environment"],
        )
        .unwrap_or("unknown")
        .to_string();
        let occurred_at = event.occurred_at;
        let mut attributes = event
            .attributes
            .into_iter()
            .map(|(key, value)| (key, AttributeValue::String(value)))
            .collect::<BTreeMap<_, _>>();
        attributes.insert(
            "sift.original_schema_version".to_string(),
            AttributeValue::Int(EVENT_SCHEMA_VERSION_V1.into()),
        );
        let canonical = Self {
            schema_version: EVENT_SCHEMA_VERSION,
            schema_url: EVENT_SCHEMA_URL.to_string(),
            event_id: event.event_id,
            project,
            environment,
            occurred_at: occurred_at.clone(),
            observed_at: occurred_at,
            signal: event.signal,
            resource: event.resource,
            instrumentation_scope: None,
            attributes,
            trace_id: event.trace_id,
            span_id: event.span_id,
            request_id: None,
            session_id: None,
            severity: event.severity,
            metric: event.metric,
            payload: event.payload,
        };
        canonical.validate()?;
        Ok(canonical)
    }
}

fn first_resource<'a>(resource: &'a BTreeMap<String, String>, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| resource.get(*key).map(String::as_str))
}

/// Deserializes either the retained v1 wire shape or canonical V2 and always
/// exposes V2 to the service core.
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
        .context("decode operational event")
        .map(IncomingEvent::into_inner)
}

fn decode_event_value(value: Value) -> Result<OperationalEventV2> {
    let version = value
        .get("schema_version")
        .and_then(Value::as_u64)
        .context("operational event requires integer schema_version")?;
    match u16::try_from(version).context("schema_version exceeds u16")? {
        EVENT_SCHEMA_VERSION_V1 => {
            OperationalEventV2::try_from(serde_json::from_value::<EventEnvelopeV1>(value)?)
        }
        EVENT_SCHEMA_VERSION => {
            let event = serde_json::from_value::<OperationalEventV2>(value)?;
            event.validate()?;
            Ok(event)
        }
        version => bail!("unsupported schema_version {version}"),
    }
}

<!-- marker: sift-operational-event-v2-model path: projects/sift/src/event/model.rs reason: Define OperationalEventV2, typed attributes, eight signals, v1 wire shape, incoming compatibility decode, and deterministic upcast. -->
// HANDWRITE-END
