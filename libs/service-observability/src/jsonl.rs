// HANDWRITE-BEGIN gap="missing-generator:logic:d8834b62" tracker="1868" reason="Define ServiceLogEventV1, stable schema constants, correlation validation, sensitive-key exclusion, bounded attributes, and the tracing-subscriber JSONL formatter."
//! Versioned collector-compatible structured stdout.
//!
//! Every line carries [`SERVICE_LOG_SCHEMA_V1`] (`axiom.service.log.v1`) as its
//! `schema` field, so a collector keys off that constant rather than off the shape
//! it happens to observe. Changing a field's meaning therefore means a new
//! constant, not an edited one.
//!
//! Three attribute policies are decided here, and each one chooses between
//! dropping and truncating:
//!
//! * **Bounds truncate.** An oversized key or value is cut to
//!   [`MAX_ATTRIBUTE_KEY_BYTES`] / [`MAX_ATTRIBUTE_VALUE_BYTES`] on a UTF-8
//!   boundary rather than rejected, so one long field cannot cost the line. Past
//!   [`MAX_ATTRIBUTES`] the remainder is dropped instead, and because the input is
//!   a `BTreeMap` the survivors are the alphabetically first — deterministic, so a
//!   caller near the limit loses the same attributes on every line rather than a
//!   different arbitrary subset each time.
//! * **Reserved keys are dropped, not renamed.** A caller attribute colliding with
//!   a schema field (`severity`, `trace_id`, `request_id`, …) is discarded, so
//!   nothing a caller passes can overwrite or forge the framing the collector
//!   trusts.
//! * **Sensitive keys are dropped, not masked.** `authorization`,
//!   `proxy_authorization`, `cookie`, `set_cookie`, `baggage` and `tracestate` are
//!   matched case-insensitively with `-` normalized to `_`, and also as a `.`, `/`
//!   or `_` suffix so a namespaced `http.request.authorization` is caught too.
//!   `baggage` and `tracestate` are in that list because they are propagation
//!   headers that carry caller-supplied payload, not just correlation ids — they
//!   are treated as credential-bearing rather than as trace metadata. Because the
//!   key is removed rather than replaced with a placeholder, a test asserts absence
//!   rather than asserting a mask, and no downstream index ever sees the field name.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::{FormatEvent, JsonFields, Writer};
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::{FmtContext, FormattedFields};
use tracing_subscriber::registry::LookupSpan;

use crate::config::{LogFormat, ServiceIdentity};

pub const SERVICE_LOG_SCHEMA_V1: &str = "axiom.service.log.v1";
pub const MAX_ATTRIBUTES: usize = 64;
pub const MAX_ATTRIBUTE_KEY_BYTES: usize = 128;
pub const MAX_ATTRIBUTE_VALUE_BYTES: usize = 4096;
pub const MAX_EVENT_BYTES: usize = 128;
pub const MAX_REQUEST_ID_BYTES: usize = 128;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServiceLogIdentityV1 {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServiceLogEventV1 {
    pub schema: String,
    pub timestamp: String,
    pub severity: String,
    pub service: ServiceLogIdentityV1,
    pub event: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_flags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
}

pub fn collector_compatible(format: LogFormat) -> bool {
    matches!(format, LogFormat::Json)
}

/// Return the checked-in JSON Schema as a parsed value.
///
/// Repository ownership comments are removed before parsing; callers always
/// receive the machine-readable schema object rather than JSON-with-comments.
pub fn service_log_schema_v1() -> Value {
    let source = include_str!("../contracts/axiom.service.log.v1.schema.json");
    let json = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("// HANDWRITE-"))
        .collect::<Vec<_>>()
        .join("\n");
    serde_json::from_str(&json).expect("axiom.service.log.v1 schema must be valid JSON")
}

#[derive(Clone, Debug)]
pub struct ServiceJsonFormatter {
    identity: ServiceIdentity,
}

impl ServiceJsonFormatter {
    pub fn new(identity: ServiceIdentity) -> Self {
        Self { identity }
    }
}

impl<S> FormatEvent<S, JsonFields> for ServiceJsonFormatter
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, JsonFields>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut span_fields = BTreeMap::new();
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                let extensions = span.extensions();
                let Some(fields) = extensions.get::<FormattedFields<JsonFields>>() else {
                    continue;
                };
                let Ok(values) = serde_json::from_str::<BTreeMap<String, Value>>(&fields.fields)
                else {
                    continue;
                };
                span_fields.extend(values);
            }
        }

        let mut visitor = JsonFieldVisitor::default();
        event.record(&mut visitor);
        let event_fields = visitor.values;
        let metadata = event.metadata();

        let explicit_event = preferred_string(&event_fields, &span_fields, "event")
            .filter(|value| !value.is_empty());
        let event_name = truncate_utf8(
            explicit_event.as_deref().unwrap_or(metadata.name()),
            MAX_EVENT_BYTES,
        );
        let message = truncate_utf8(
            field_string(event_fields.get("message"))
                .as_deref()
                .unwrap_or(&event_name),
            MAX_ATTRIBUTE_VALUE_BYTES,
        );

        let trace_id = preferred_hex(&event_fields, &span_fields, "trace_id", 32, true);
        let span_id = preferred_hex(&event_fields, &span_fields, "span_id", 16, true);
        let parent_span_id = preferred_hex(&event_fields, &span_fields, "parent_span_id", 16, true);
        let trace_flags = preferred_hex(&event_fields, &span_fields, "trace_flags", 2, false);
        let request_id = preferred_request_id(&event_fields, &span_fields);

        let mut merged = span_fields;
        merged.extend(event_fields);
        merged
            .entry("target".to_string())
            .or_insert_with(|| Value::String(metadata.target().to_string()));
        let attributes = bounded_attributes(merged);

        let mut timestamp = String::new();
        let timer = tracing_subscriber::fmt::time::SystemTime;
        timer.format_time(&mut Writer::new(&mut timestamp))?;

        let record = ServiceLogEventV1 {
            schema: SERVICE_LOG_SCHEMA_V1.to_string(),
            timestamp,
            severity: metadata.level().as_str().to_string(),
            service: ServiceLogIdentityV1 {
                name: truncate_utf8(self.identity.name(), MAX_EVENT_BYTES),
                version: truncate_utf8(self.identity.version(), MAX_EVENT_BYTES),
            },
            event: event_name,
            message,
            trace_id,
            span_id,
            parent_span_id,
            trace_flags,
            request_id,
            attributes,
        };
        let line = serde_json::to_string(&record).map_err(|_| fmt::Error)?;
        writeln!(writer, "{line}")
    }
}

#[derive(Default)]
struct JsonFieldVisitor {
    values: BTreeMap<String, Value>,
}

impl JsonFieldVisitor {
    fn insert(&mut self, field: &Field, value: Value) {
        self.values.insert(field.name().to_string(), value);
    }
}

impl Visit for JsonFieldVisitor {
    fn record_f64(&mut self, field: &Field, value: f64) {
        let value = serde_json::Number::from_f64(value)
            .map(Value::Number)
            .unwrap_or(Value::Null);
        self.insert(field, value);
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.insert(field, Value::Number(value.into()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.insert(field, Value::Number(value.into()));
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        match i64::try_from(value) {
            Ok(value) => self.record_i64(field, value),
            Err(_) => self.insert(field, Value::String(value.to_string())),
        }
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        match u64::try_from(value) {
            Ok(value) => self.record_u64(field, value),
            Err(_) => self.insert(field, Value::String(value.to_string())),
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.insert(field, Value::Bool(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.insert(field, Value::String(value.to_string()));
    }

    fn record_bytes(&mut self, field: &Field, value: &[u8]) {
        let encoded = value
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        self.insert(field, Value::String(encoded));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.insert(field, Value::String(value.to_string()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.insert(field, Value::String(format!("{value:?}")));
    }
}

fn preferred_string(
    event_fields: &BTreeMap<String, Value>,
    span_fields: &BTreeMap<String, Value>,
    key: &str,
) -> Option<String> {
    field_string(event_fields.get(key)).or_else(|| field_string(span_fields.get(key)))
}

fn preferred_hex(
    event_fields: &BTreeMap<String, Value>,
    span_fields: &BTreeMap<String, Value>,
    key: &str,
    expected_len: usize,
    reject_zero: bool,
) -> Option<String> {
    [event_fields.get(key), span_fields.get(key)]
        .into_iter()
        .flatten()
        .filter_map(|value| field_string(Some(value)))
        .find(|value| valid_lower_hex(value, expected_len, reject_zero))
}

fn preferred_request_id(
    event_fields: &BTreeMap<String, Value>,
    span_fields: &BTreeMap<String, Value>,
) -> Option<String> {
    const KEYS: [&str; 3] = ["request_id", "request.id", "http.request.id"];
    for fields in [event_fields, span_fields] {
        for key in KEYS {
            if let Some(value) =
                field_string(fields.get(key)).filter(|value| valid_request_id(value))
            {
                return Some(value);
            }
        }
    }
    None
}

fn field_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) => Some(value.clone()),
        _ => None,
    }
}

fn valid_lower_hex(value: &str, expected_len: usize, reject_zero: bool) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && (!reject_zero || value.bytes().any(|byte| byte != b'0'))
}

fn valid_request_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_REQUEST_ID_BYTES && !value.chars().any(char::is_control)
}

fn bounded_attributes(values: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    let mut attributes = BTreeMap::new();
    for (key, value) in values {
        if attributes.len() == MAX_ATTRIBUTES {
            break;
        }
        if is_reserved_key(&key) || is_sensitive_key(&key) {
            continue;
        }
        let key = truncate_utf8(&key, MAX_ATTRIBUTE_KEY_BYTES);
        if key.is_empty() {
            continue;
        }
        attributes.insert(key, bounded_value(value));
    }
    attributes
}

fn bounded_value(value: Value) -> Value {
    match value {
        Value::String(value) => Value::String(truncate_utf8(&value, MAX_ATTRIBUTE_VALUE_BYTES)),
        Value::Number(_) | Value::Bool(_) | Value::Null => value,
        other => Value::String(truncate_utf8(&other.to_string(), MAX_ATTRIBUTE_VALUE_BYTES)),
    }
}

fn is_reserved_key(key: &str) -> bool {
    matches!(
        key,
        "schema"
            | "timestamp"
            | "severity"
            | "service"
            | "event"
            | "message"
            | "trace_id"
            | "span_id"
            | "parent_span_id"
            | "trace_flags"
            | "request_id"
            | "request.id"
            | "http.request.id"
            | "attributes"
    )
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace('-', "_");
    [
        "authorization",
        "proxy_authorization",
        "cookie",
        "set_cookie",
        "baggage",
        "tracestate",
    ]
    .into_iter()
    .any(|sensitive| {
        normalized == sensitive
            || normalized.ends_with(&format!(".{sensitive}"))
            || normalized.ends_with(&format!("/{sensitive}"))
            || normalized.ends_with(&format!("_{sensitive}"))
    })
}

fn truncate_utf8(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

// HANDWRITE-END
