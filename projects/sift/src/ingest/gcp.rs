// HANDWRITE-BEGIN gap="sift-gcp-structured-normalizer" tracker="1658" reason="Normalize representative Cloud Logging structured JSON and GKE monitored resources into OperationalEventV2."
use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::{AttributeValue, OperationalEventV2, SignalKind};

pub fn looks_like_structured_log(value: &Value) -> bool {
    value.get("jsonPayload").is_some() && value.get("resource").and_then(Value::as_object).is_some()
}

pub fn normalize_structured_log(value: Value, project_hint: &str) -> Result<OperationalEventV2> {
    let object = value
        .as_object()
        .context("GCP structured log entry must be a JSON object")?;
    let json_payload = object
        .get("jsonPayload")
        .filter(|value| value.is_object())
        .context("GCP ingest is structured-only and requires object jsonPayload")?;
    let resource_object = object
        .get("resource")
        .and_then(Value::as_object)
        .context("GCP structured log requires resource")?;
    let resource_type = resource_object
        .get("type")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .context("GCP monitored resource type is required")?;
    let labels = resource_object
        .get("labels")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let project = labels
        .get("project_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(project_hint);
    if project.trim().is_empty() {
        bail!("GCP structured log requires project_id or x-sift-project");
    }
    let occurred_at = parse_timestamp(object.get("timestamp"), "timestamp")?;
    let observed = match object.get("receiveTimestamp") {
        Some(value) => parse_timestamp(Some(value), "receiveTimestamp")?,
        None => occurred_at.clone(),
    };
    let event_id = object
        .get("insertId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| stable_id(project, resource_type, &occurred_at, json_payload));
    let environment = labels
        .get("environment")
        .or_else(|| labels.get("namespace_name"))
        .and_then(Value::as_str)
        .unwrap_or("default");
    let mut event = OperationalEventV2::for_project(
        project,
        environment,
        event_id,
        SignalKind::Log,
        json!({"jsonPayload": json_payload}),
    );
    event.occurred_at = occurred_at;
    event.observed_at = observed;
    event.resource = normalize_resource(resource_type, &labels);
    event.severity = object
        .get("severity")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_uppercase());
    event.trace_id = object
        .get("trace")
        .and_then(Value::as_str)
        .and_then(trace_id);
    event.span_id = object
        .get("spanId")
        .or_else(|| object.get("span_id"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    event.request_id = request_id(object, json_payload);
    event.attributes = normalize_attributes(object);
    event.validate()?;
    Ok(event)
}

fn parse_timestamp(value: Option<&Value>, name: &str) -> Result<String> {
    let Some(value) = value else {
        return Ok(Utc::now().to_rfc3339());
    };
    if let Some(value) = value.as_str() {
        DateTime::parse_from_rfc3339(value)
            .with_context(|| format!("GCP {name} must be RFC3339"))?;
        return Ok(value.to_string());
    }
    let seconds = value
        .get("seconds")
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
        .context("GCP timestamp seconds must be an integer")?;
    let nanos = value.get("nanos").and_then(Value::as_u64).unwrap_or(0) as u32;
    DateTime::<Utc>::from_timestamp(seconds, nanos)
        .context("GCP timestamp is outside the supported range")
        .map(|value| value.to_rfc3339())
}

fn normalize_resource(
    resource_type: &str,
    labels: &serde_json::Map<String, Value>,
) -> BTreeMap<String, String> {
    let mut output = BTreeMap::from([("gcp.resource.type".to_string(), resource_type.to_string())]);
    for (key, value) in labels {
        let Some(value) = scalar(value) else {
            continue;
        };
        output.insert(format!("gcp.resource.label.{key}"), value.clone());
        let normalized = match key.as_str() {
            "project_id" => Some("gcp.project_id"),
            "cluster_name" => Some("k8s.cluster.name"),
            "namespace_name" => Some("k8s.namespace.name"),
            "pod_name" => Some("k8s.pod.name"),
            "container_name" => Some("k8s.container.name"),
            "location" => Some("cloud.region"),
            "zone" => Some("cloud.availability_zone"),
            _ => None,
        };
        if let Some(normalized) = normalized {
            output.insert(normalized.to_string(), value);
        }
    }
    output
}

fn normalize_attributes(
    object: &serde_json::Map<String, Value>,
) -> BTreeMap<String, AttributeValue> {
    let mut attributes = BTreeMap::new();
    if let Some(labels) = object.get("labels").and_then(Value::as_object) {
        for (key, value) in labels {
            if let Some(value) = scalar(value) {
                attributes.insert(
                    format!("gcp.logging.label.{key}"),
                    AttributeValue::String(value),
                );
            }
        }
    }
    for (source, target) in [
        ("logName", "gcp.log.name"),
        ("traceSampled", "gcp.trace.sampled"),
    ] {
        if let Some(value) = object.get(source) {
            let attribute = match value {
                Value::String(value) => AttributeValue::String(value.clone()),
                Value::Bool(value) => AttributeValue::Bool(*value),
                _ => continue,
            };
            attributes.insert(target.to_string(), attribute);
        }
    }
    if let Some(http) = object.get("httpRequest").and_then(Value::as_object) {
        for (key, value) in http {
            if let Some(value) = scalar(value) {
                attributes.insert(
                    format!("gcp.http_request.{key}"),
                    AttributeValue::String(value),
                );
            }
        }
    }
    attributes
}

fn request_id(object: &serde_json::Map<String, Value>, json_payload: &Value) -> Option<String> {
    object
        .get("httpRequest")
        .and_then(|value| value.get("requestId").or_else(|| value.get("request_id")))
        .or_else(|| {
            object
                .get("labels")
                .and_then(|value| value.get("logging.googleapis.com/request_id"))
        })
        .or_else(|| {
            json_payload
                .get("request_id")
                .or_else(|| json_payload.get("requestId"))
        })
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn trace_id(value: &str) -> Option<String> {
    let value = value.rsplit('/').next().unwrap_or(value);
    (!value.is_empty()).then(|| value.to_string())
}

fn stable_id(project: &str, resource_type: &str, timestamp: &str, payload: &Value) -> String {
    let mut digest = Sha256::new();
    digest.update(project.as_bytes());
    digest.update([0]);
    digest.update(resource_type.as_bytes());
    digest.update([0]);
    digest.update(timestamp.as_bytes());
    digest.update([0]);
    digest.update(serde_json::to_vec(payload).unwrap_or_default());
    format!("gcp-log-{}", hex::encode(&digest.finalize()[..16]))
}

fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}
// HANDWRITE-END
