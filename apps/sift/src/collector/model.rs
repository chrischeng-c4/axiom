// HANDWRITE-BEGIN gap="missing-generator:logic:4b472642" tracker="1873" reason="Decode and strictly validate ServiceLogEventV1, convert bounded primitive attributes, preserve payload, validate correlation, and derive stable ids."
use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use service_observability::{
    StructuredServiceLogV1, MAX_ATTRIBUTES, MAX_ATTRIBUTE_KEY_BYTES, MAX_ATTRIBUTE_VALUE_BYTES,
    MAX_EVENT_BYTES, MAX_REQUEST_ID_BYTES, SERVICE_LOG_SCHEMA_V1,
};
use sha2::{Digest, Sha256};

use super::source::RecordEnrichment;
use crate::{AttributeValue, InstrumentationScope, OperationalEventV2, SignalKind};

// <HANDWRITE gap="missing-generator:logic" tracker="1675" reason="Apply bounded source enrichment and shared Cloud Logging coexistence identity after decoding.">
pub fn decode_service_log(
    raw_line: &[u8],
    source_id: &str,
    offset: u64,
    project: &str,
    environment: &str,
) -> Result<OperationalEventV2> {
    decode_service_log_enriched(
        raw_line,
        source_id,
        offset,
        project,
        environment,
        &RecordEnrichment::default(),
    )
}

pub(crate) fn decode_service_log_enriched(
    raw_line: &[u8],
    source_id: &str,
    offset: u64,
    project: &str,
    environment: &str,
    enrichment: &RecordEnrichment,
) -> Result<OperationalEventV2> {
    let json = trim_line_ending(raw_line);
    let log: StructuredServiceLogV1 =
        serde_json::from_slice(json).context("decode axiom service log JSON")?;
    validate(&log)?;

    let event_id = deterministic_event_id(source_id, offset, raw_line);
    let payload = serde_json::to_value(&log)?;
    let mut event =
        OperationalEventV2::for_project(project, environment, event_id, SignalKind::Log, payload);
    event.occurred_at = log.timestamp.clone();
    event.observed_at = Utc::now().to_rfc3339();
    event
        .resource
        .insert("service.name".to_string(), log.service.name.clone());
    event
        .resource
        .insert("service.version".to_string(), log.service.version.clone());
    event
        .resource
        .insert("collector.source_id".to_string(), source_id.to_string());
    event.instrumentation_scope = Some(InstrumentationScope {
        name: log.service.name.clone(),
        version: Some(log.service.version.clone()),
        attributes: BTreeMap::new(),
        schema_url: Some(SERVICE_LOG_SCHEMA_V1.to_string()),
    });
    for (key, value) in &log.attributes {
        if let Some(value) = attribute_value(value)? {
            event.attributes.insert(key.clone(), value);
        }
    }
    event.attributes.insert(
        "event.name".to_string(),
        AttributeValue::String(log.event.clone()),
    );
    event.attributes.insert(
        "collector.source_id".to_string(),
        AttributeValue::String(source_id.to_string()),
    );
    event.attributes.insert(
        "collector.offset".to_string(),
        i64::try_from(offset)
            .map(AttributeValue::Int)
            .unwrap_or_else(|_| AttributeValue::String(offset.to_string())),
    );
    if let Some(parent_span_id) = log.parent_span_id.as_ref() {
        event.attributes.insert(
            "parent_span_id".to_string(),
            AttributeValue::String(parent_span_id.clone()),
        );
    }
    if let Some(trace_flags) = log.trace_flags.as_ref() {
        event.attributes.insert(
            "trace.flags".to_string(),
            AttributeValue::String(trace_flags.clone()),
        );
    }
    event.trace_id = log.trace_id;
    event.span_id = log.span_id;
    event.request_id = log.request_id;
    event.severity = Some(log.severity);
    event.resource.extend(enrichment.resource.clone());
    event.attributes.extend(enrichment.attributes.clone());
    if enrichment.cloud_logging_coexistence {
        let resource_type = event
            .resource
            .get("gcp.resource.type")
            .context("CRI enrichment requires gcp.resource.type")?;
        event.event_id = crate::ingest::gcp::stable_id(
            project,
            resource_type,
            &event.occurred_at,
            &event.resource,
            &event.payload,
        );
    }
    event.validate()?;
    Ok(event)
}
// </HANDWRITE>

fn validate(log: &StructuredServiceLogV1) -> Result<()> {
    if log.schema != SERVICE_LOG_SCHEMA_V1 {
        bail!(
            "unsupported service log schema {}; expected {}",
            log.schema,
            SERVICE_LOG_SCHEMA_V1
        );
    }
    DateTime::parse_from_rfc3339(&log.timestamp)
        .context("service log timestamp must be RFC3339")?;
    if !matches!(
        log.severity.as_str(),
        "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR"
    ) {
        bail!("unsupported service log severity {}", log.severity);
    }
    bounded_nonempty("service.name", &log.service.name, MAX_EVENT_BYTES)?;
    bounded_nonempty("service.version", &log.service.version, MAX_EVENT_BYTES)?;
    bounded_nonempty("event", &log.event, MAX_EVENT_BYTES)?;
    if log.message.len() > MAX_ATTRIBUTE_VALUE_BYTES {
        bail!("service log message exceeds {MAX_ATTRIBUTE_VALUE_BYTES} bytes");
    }
    if log.attributes.len() > MAX_ATTRIBUTES {
        bail!("service log attributes exceed {MAX_ATTRIBUTES} entries");
    }
    for (key, value) in &log.attributes {
        bounded_nonempty("attribute key", key, MAX_ATTRIBUTE_KEY_BYTES)?;
        validate_attribute(value)
            .with_context(|| format!("invalid service log attribute {key}"))?;
    }
    validate_optional_hex("trace_id", log.trace_id.as_deref(), 32, true)?;
    validate_optional_hex("span_id", log.span_id.as_deref(), 16, true)?;
    validate_optional_hex("parent_span_id", log.parent_span_id.as_deref(), 16, true)?;
    validate_optional_hex("trace_flags", log.trace_flags.as_deref(), 2, false)?;
    if let Some(request_id) = log.request_id.as_deref() {
        if request_id.is_empty()
            || request_id.len() > MAX_REQUEST_ID_BYTES
            || request_id.chars().any(char::is_control)
        {
            bail!("invalid service log request_id");
        }
    }
    Ok(())
}

fn bounded_nonempty(name: &str, value: &str, max: usize) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{name} must not be empty");
    }
    if value.len() > max {
        bail!("{name} exceeds {max} bytes");
    }
    Ok(())
}

fn validate_attribute(value: &Value) -> Result<()> {
    match value {
        Value::String(value) if value.len() > MAX_ATTRIBUTE_VALUE_BYTES => {
            bail!("string exceeds {MAX_ATTRIBUTE_VALUE_BYTES} bytes")
        }
        Value::String(_) | Value::Bool(_) | Value::Null => Ok(()),
        Value::Number(value) if value.as_f64().is_some_and(f64::is_finite) => Ok(()),
        Value::Number(_) => bail!("number must be finite"),
        Value::Array(_) | Value::Object(_) => {
            bail!("collector attributes must be primitive JSON values")
        }
    }
}

fn attribute_value(value: &Value) -> Result<Option<AttributeValue>> {
    Ok(match value {
        Value::Null => None,
        Value::String(value) => Some(AttributeValue::String(value.clone())),
        Value::Bool(value) => Some(AttributeValue::Bool(*value)),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                Some(AttributeValue::Int(value))
            } else {
                let value = value.as_f64().context("JSON number is not representable")?;
                if !value.is_finite() {
                    bail!("JSON number must be finite");
                }
                Some(AttributeValue::Double(value))
            }
        }
        Value::Array(_) | Value::Object(_) => {
            bail!("collector attributes must be primitive JSON values")
        }
    })
}

fn validate_optional_hex(
    name: &str,
    value: Option<&str>,
    len: usize,
    reject_zero: bool,
) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    let valid = value.len() == len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && (!reject_zero || value.bytes().any(|byte| byte != b'0'));
    if !valid {
        bail!("invalid service log {name}");
    }
    Ok(())
}

fn deterministic_event_id(source_id: &str, offset: u64, raw_line: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(source_id.as_bytes());
    digest.update([0]);
    digest.update(offset.to_le_bytes());
    digest.update([0]);
    digest.update(raw_line);
    format!("stdout-{}", hex::encode(digest.finalize()))
}

fn trim_line_ending(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\n")
        .unwrap_or(line)
        .strip_suffix(b"\r")
        .unwrap_or_else(|| line.strip_suffix(b"\n").unwrap_or(line))
}

#[cfg(test)]
mod tests {
    use service_observability::ServiceLogIdentityV1;

    use super::*;

    fn fixture() -> StructuredServiceLogV1 {
        StructuredServiceLogV1 {
            schema: SERVICE_LOG_SCHEMA_V1.to_string(),
            timestamp: "2026-07-17T10:00:00Z".to_string(),
            severity: "INFO".to_string(),
            service: ServiceLogIdentityV1 {
                name: "lumen".to_string(),
                version: "0.4.21".to_string(),
            },
            event: "collection_create_or_extend".to_string(),
            message: "collection created".to_string(),
            trace_id: Some("0af7651916cd43dd8448eb211c80319c".to_string()),
            span_id: Some("b7ad6b7169203331".to_string()),
            parent_span_id: Some("00f067aa0ba902b7".to_string()),
            trace_flags: Some("01".to_string()),
            request_id: Some("request-42".to_string()),
            attributes: BTreeMap::from([
                (
                    "collection_id".to_string(),
                    Value::String("docs".to_string()),
                ),
                ("fields".to_string(), Value::Number(2.into())),
            ]),
        }
    }

    #[test]
    fn maps_shared_log_to_canonical_event_with_stable_id() {
        let raw = serde_json::to_vec(&fixture()).unwrap();
        let first = decode_service_log(&raw, "fixture", 10, "local", "test").unwrap();
        let again = decode_service_log(&raw, "fixture", 10, "local", "test").unwrap();
        let shifted = decode_service_log(&raw, "fixture", 11, "local", "test").unwrap();

        assert_eq!(first.event_id, again.event_id);
        assert_ne!(first.event_id, shifted.event_id);
        assert_eq!(first.project, "local");
        assert_eq!(first.resource["service.name"], "lumen");
        assert_eq!(
            first.trace_id.as_deref(),
            Some("0af7651916cd43dd8448eb211c80319c")
        );
        assert_eq!(
            first.attributes["parent_span_id"].as_str(),
            Some("00f067aa0ba902b7")
        );
        assert_eq!(first.payload["message"], "collection created");
    }

    #[test]
    fn rejects_wrong_schema_invalid_ids_and_nested_attributes() {
        let mut log = fixture();
        log.schema = "other.v1".to_string();
        assert!(decode_service_log(
            &serde_json::to_vec(&log).unwrap(),
            "fixture",
            0,
            "local",
            "test"
        )
        .is_err());

        let mut log = fixture();
        log.trace_id = Some("ABCDEFABCDEFABCDEFABCDEFABCDEFAB".to_string());
        assert!(decode_service_log(
            &serde_json::to_vec(&log).unwrap(),
            "fixture",
            0,
            "local",
            "test"
        )
        .is_err());

        let mut log = fixture();
        log.attributes
            .insert("nested".to_string(), serde_json::json!({"no": "objects"}));
        assert!(decode_service_log(
            &serde_json::to_vec(&log).unwrap(),
            "fixture",
            0,
            "local",
            "test"
        )
        .is_err());
    }
}
// HANDWRITE-END
