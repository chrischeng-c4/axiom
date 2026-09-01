// HANDWRITE-BEGIN gap="sift-gcp-structured-normalizer" tracker="1658" reason="Normalize representative Cloud Logging structured JSON and GKE monitored resources into OperationalEventV2."
use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use service_observability::SERVICE_LOG_SCHEMA_V1;
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
    let resource = normalize_resource(resource_type, &labels);
    let insert_id = object
        .get("insertId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let is_axiom_service_log =
        json_payload.get("schema").and_then(Value::as_str) == Some(SERVICE_LOG_SCHEMA_V1);
    let event_id = if is_axiom_service_log {
        stable_id(
            project,
            resource_type,
            &occurred_at,
            &resource,
            json_payload,
        )
    } else {
        insert_id.clone().unwrap_or_else(|| {
            stable_id(
                project,
                resource_type,
                &occurred_at,
                &resource,
                json_payload,
            )
        })
    };
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
    event.resource = resource;
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
    if let Some(insert_id) = insert_id {
        event.attributes.insert(
            "gcp.insert_id".to_string(),
            AttributeValue::String(insert_id),
        );
    }
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

// <HANDWRITE gap="missing-generator:logic" tracker="1675" reason="Share the insertId-free Cloud Logging identity with CRI collection.">
pub(crate) fn stable_id(
    project: &str,
    resource_type: &str,
    timestamp: &str,
    resource: &BTreeMap<String, String>,
    payload: &Value,
) -> String {
    let mut digest = Sha256::new();
    digest.update(project.as_bytes());
    digest.update([0]);
    digest.update(resource_type.as_bytes());
    digest.update([0]);
    digest.update(timestamp.as_bytes());
    digest.update([0]);
    for key in [
        "gcp.resource.label.project_id",
        "gcp.resource.label.location",
        "gcp.resource.label.cluster_name",
        "gcp.resource.label.namespace_name",
        "gcp.resource.label.pod_name",
        "gcp.resource.label.container_name",
    ] {
        digest.update(key.as_bytes());
        digest.update([0]);
        digest.update(resource.get(key).map(String::as_bytes).unwrap_or_default());
        digest.update([0]);
    }
    digest.update(canonical_json(payload));
    format!("gcp-log-{}", hex::encode(&digest.finalize()[..16]))
}

fn canonical_json(value: &Value) -> Vec<u8> {
    fn write(value: &Value, output: &mut Vec<u8>) {
        match value {
            Value::Object(object) => {
                output.push(b'{');
                let mut keys = object.keys().collect::<Vec<_>>();
                keys.sort_unstable();
                for (index, key) in keys.into_iter().enumerate() {
                    if index > 0 {
                        output.push(b',');
                    }
                    serde_json::to_writer(&mut *output, key)
                        .expect("JSON object key serialization");
                    output.push(b':');
                    write(&object[key], output);
                }
                output.push(b'}');
            }
            Value::Array(values) => {
                output.push(b'[');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        output.push(b',');
                    }
                    write(value, output);
                }
                output.push(b']');
            }
            _ => serde_json::to_writer(output, value).expect("JSON scalar serialization"),
        }
    }

    let mut output = Vec::new();
    write(value, &mut output);
    output
}
// </HANDWRITE>

fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod coexistence_tests {
    use super::*;

    #[test]
    fn fallback_identity_is_recursive_object_order_independent() {
        let resource = BTreeMap::from([
            (
                "gcp.resource.label.project_id".to_string(),
                "project-a".to_string(),
            ),
            (
                "gcp.resource.label.pod_name".to_string(),
                "lumen-0".to_string(),
            ),
        ]);
        let left = serde_json::from_str::<Value>(
            r#"{"schema":"axiom.service.log.v1","service":{"name":"lumen","version":"1"},"attributes":{"b":2,"a":1}}"#,
        )
        .unwrap();
        let right = serde_json::from_str::<Value>(
            r#"{"attributes":{"a":1,"b":2},"service":{"version":"1","name":"lumen"},"schema":"axiom.service.log.v1"}"#,
        )
        .unwrap();
        assert_eq!(
            stable_id(
                "project-a",
                "k8s_container",
                "2026-07-17T10:00:00Z",
                &resource,
                &left
            ),
            stable_id(
                "project-a",
                "k8s_container",
                "2026-07-17T10:00:00Z",
                &resource,
                &right
            )
        );
    }

    #[test]
    fn fallback_identity_separates_workload_resources() {
        let payload = serde_json::json!({"message": "same"});
        let left = BTreeMap::from([(
            "gcp.resource.label.pod_name".to_string(),
            "lumen-0".to_string(),
        )]);
        let right = BTreeMap::from([(
            "gcp.resource.label.pod_name".to_string(),
            "lumen-1".to_string(),
        )]);
        assert_ne!(
            stable_id(
                "project-a",
                "k8s_container",
                "2026-07-17T10:00:00Z",
                &left,
                &payload
            ),
            stable_id(
                "project-a",
                "k8s_container",
                "2026-07-17T10:00:00Z",
                &right,
                &payload
            )
        );
    }

    #[test]
    fn axiom_service_log_uses_canonical_id_and_preserves_cloud_insert_id() {
        let payload = serde_json::json!({
            "schema": SERVICE_LOG_SCHEMA_V1,
            "timestamp": "2026-07-17T10:00:00Z",
            "severity": "INFO",
            "service": {"name": "lumen", "version": "1"},
            "event": "request_complete",
            "message": "done",
            "attributes": {}
        });
        let value = serde_json::json!({
            "insertId": "cloud-generated-id",
            "timestamp": "2026-07-17T10:00:00Z",
            "jsonPayload": payload,
            "resource": {
                "type": "k8s_container",
                "labels": {
                    "project_id": "project-a",
                    "namespace_name": "prod",
                    "pod_name": "lumen-0",
                    "container_name": "lumen"
                }
            }
        });
        let event = normalize_structured_log(value, "unused").unwrap();
        assert!(event.event_id.starts_with("gcp-log-"));
        assert_ne!(event.event_id, "cloud-generated-id");
        assert_eq!(
            event.attributes["gcp.insert_id"].as_str(),
            Some("cloud-generated-id")
        );
    }
}
// HANDWRITE-END
