// HANDWRITE-BEGIN gap="missing-generator:logic:d8653acc" tracker="1873" reason="POST official OTLP logs JSON with bounded retries and advance checkpoints only after full success."
use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::{bail, Context, Result};
use reqwest::{StatusCode, Url};
use serde_json::{json, Value};

use crate::{AttributeValue, OperationalEventV2};

#[derive(Clone)]
pub struct CollectorClient {
    http: reqwest::Client,
    ingest_url: Url,
    project: String,
    token: Option<String>,
    projected_token: Option<Arc<service_auth::k8s::ProjectedTokenFile>>,
}

impl CollectorClient {
    pub fn new(
        endpoint: &str,
        project: &str,
        token: Option<String>,
        request_timeout: Duration,
    ) -> Result<Self> {
        if project.trim().is_empty() {
            bail!("collector project must not be empty");
        }
        let mut endpoint = Url::parse(endpoint).context("collector endpoint must be a URL")?;
        if !matches!(endpoint.scheme(), "http" | "https") || endpoint.host_str().is_none() {
            bail!("collector endpoint must use http or https and include a host");
        }
        endpoint.set_path("/v1/logs");
        endpoint.set_query(None);
        endpoint.set_fragment(None);
        let http = reqwest::Client::builder()
            .timeout(request_timeout)
            .build()
            .context("build collector HTTP client")?;
        Ok(Self {
            http,
            ingest_url: endpoint,
            project: project.to_string(),
            token,
            projected_token: None,
        })
    }

    pub fn with_projected_token_file(mut self, path: PathBuf, audience: String) -> Self {
        self.projected_token = Some(Arc::new(service_auth::k8s::ProjectedTokenFile::new(
            path, audience,
        )));
        self
    }

    async fn send_once(
        &self,
        events: &[OperationalEventV2],
    ) -> std::result::Result<service_collector::DeliveryReceipt, service_collector::DeliveryFailure>
    {
        if events.is_empty() {
            return Ok(service_collector::DeliveryReceipt::default());
        }
        let request = otlp_logs_request(events).map_err(|error| {
            service_collector::DeliveryFailure::permanent(format!(
                "build OTLP logs request: {error}"
            ))
        })?;
        let projected_token = self
            .projected_token
            .as_ref()
            .map(|source| source.read())
            .transpose()
            .map_err(|error| {
                service_collector::DeliveryFailure::permanent(format!(
                    "read rotating Sift ServiceAccount credential: {error}"
                ))
            })?;
        let mut builder = self
            .http
            .post(self.ingest_url.clone())
            .header("x-sift-project", &self.project)
            .json(&request);
        if let Some(token) = projected_token.as_ref() {
            builder = builder.bearer_auth(token.expose());
        } else if let Some(token) = self.token.as_deref() {
            builder = builder.bearer_auth(token);
        }
        let response = builder.send().await.map_err(|error| {
            service_collector::DeliveryFailure::retryable(format!(
                "Sift ingest transport error: {error}"
            ))
        })?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let message = format!("Sift ingest HTTP {status}: {}", truncate(&body, 512));
            return Err(if retryable_status(status) {
                service_collector::DeliveryFailure::retryable(message)
            } else {
                service_collector::DeliveryFailure::permanent(format!(
                    "{message}; verify SIFT_URL, project authorization, and SIFT_TOKEN"
                ))
            });
        }
        let response: Value = response.json().await.map_err(|error| {
            service_collector::DeliveryFailure::permanent(format!(
                "decode Sift OTLP logs response: {error}"
            ))
        })?;
        let rejected = response
            .pointer("/partialSuccess/rejectedLogRecords")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if rejected != 0 {
            let message = response
                .pointer("/partialSuccess/errorMessage")
                .and_then(Value::as_str)
                .unwrap_or("Sift returned OTLP partial success");
            return Err(service_collector::DeliveryFailure::permanent(format!(
                "Sift rejected {rejected} collector log record(s): {}; checkpoint unchanged",
                truncate(message, 512)
            )));
        }
        Ok(service_collector::DeliveryReceipt {
            accepted: events.len() as u64,
            duplicates: 0,
        })
    }
}

#[async_trait::async_trait]
impl service_collector::BatchSink<OperationalEventV2> for CollectorClient {
    async fn send(
        &self,
        events: &[OperationalEventV2],
    ) -> std::result::Result<service_collector::DeliveryReceipt, service_collector::DeliveryFailure>
    {
        self.send_once(events).await
    }
}

fn otlp_logs_request(events: &[OperationalEventV2]) -> Result<Value> {
    let resource_logs = events
        .iter()
        .map(|event| {
            let mut resource = event.resource.clone();
            resource.insert(
                "deployment.environment.name".into(),
                event.environment.clone(),
            );
            let resource_attributes = resource
                .into_iter()
                .map(|(key, value)| json!({"key": key, "value": {"stringValue": value}}))
                .collect::<Vec<_>>();

            let mut attributes = event.attributes.clone();
            attributes.insert(
                "sift.event_id".into(),
                AttributeValue::String(event.event_id.clone()),
            );
            if let Some(request_id) = &event.request_id {
                attributes.insert(
                    "sift.request_id".into(),
                    AttributeValue::String(request_id.clone()),
                );
            }
            if let Some(session_id) = &event.session_id {
                attributes.insert(
                    "sift.session_id".into(),
                    AttributeValue::String(session_id.clone()),
                );
            }
            let attributes = attributes
                .into_iter()
                .map(|(key, value)| json!({"key": key, "value": otlp_any_value(&value)}))
                .collect::<Vec<_>>();
            let body = event
                .payload
                .get("message")
                .cloned()
                .unwrap_or_else(|| event.payload.clone());
            let mut record = json!({
                "timeUnixNano": rfc3339_nanos(&event.occurred_at)?,
                "observedTimeUnixNano": rfc3339_nanos(&event.observed_at)?,
                "severityText": event.severity.clone().unwrap_or_default(),
                "body": json_any_value(&body),
                "attributes": attributes,
            });
            if let Some(trace_id) = &event.trace_id {
                record["traceId"] = Value::String(trace_id.clone());
            }
            if let Some(span_id) = &event.span_id {
                record["spanId"] = Value::String(span_id.clone());
            }
            let scope = event
                .instrumentation_scope
                .as_ref()
                .map(|scope| {
                    json!({
                        "name": scope.name,
                        "version": scope.version,
                        "attributes": scope.attributes.iter().map(|(key, value)| {
                            json!({"key": key, "value": otlp_any_value(value)})
                        }).collect::<Vec<_>>()
                    })
                })
                .unwrap_or_else(|| json!({"name": "sift-agent"}));
            Ok(json!({
                "resource": {"attributes": resource_attributes},
                "scopeLogs": [{"scope": scope, "logRecords": [record]}]
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(json!({"resourceLogs": resource_logs}))
}

fn rfc3339_nanos(value: &str) -> Result<String> {
    let timestamp = chrono::DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("collector timestamp `{value}` is not RFC3339"))?;
    let nanos = timestamp
        .timestamp_nanos_opt()
        .context("collector timestamp is outside the OTLP nanosecond range")?;
    Ok(nanos.to_string())
}

fn otlp_any_value(value: &AttributeValue) -> Value {
    match value {
        AttributeValue::String(value) => json!({"stringValue": value}),
        AttributeValue::Bool(value) => json!({"boolValue": value}),
        AttributeValue::Int(value) => json!({"intValue": value.to_string()}),
        AttributeValue::Double(value) => json!({"doubleValue": value}),
        AttributeValue::Bytes(value) => json!({"bytesValue": value}),
        AttributeValue::Array(values) => json!({
            "arrayValue": {"values": values.iter().map(otlp_any_value).collect::<Vec<_>>()}
        }),
        AttributeValue::Map(values) => json!({
            "kvlistValue": {"values": values.iter().map(|(key, value)| {
                json!({"key": key, "value": otlp_any_value(value)})
            }).collect::<Vec<_>>()}
        }),
    }
}

fn json_any_value(value: &Value) -> Value {
    match value {
        Value::String(value) => json!({"stringValue": value}),
        Value::Bool(value) => json!({"boolValue": value}),
        Value::Number(value) if value.as_i64().is_some() => {
            json!({"intValue": value.as_i64().unwrap().to_string()})
        }
        Value::Number(value) => json!({"doubleValue": value.as_f64().unwrap_or_default()}),
        Value::Array(values) => json!({
            "arrayValue": {"values": values.iter().map(json_any_value).collect::<Vec<_>>()}
        }),
        Value::Object(values) => json!({
            "kvlistValue": {"values": values.iter().map(|(key, value)| {
                json!({"key": key, "value": json_any_value(value)})
            }).collect::<Vec<_>>()}
        }),
        Value::Null => json!({"stringValue": ""}),
    }
}

fn retryable_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn truncate(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_classification_is_explicit() {
        assert!(retryable_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(retryable_status(StatusCode::SERVICE_UNAVAILABLE));
        assert!(!retryable_status(StatusCode::BAD_REQUEST));
        assert!(!retryable_status(StatusCode::UNAUTHORIZED));
    }

    #[test]
    fn client_rejects_non_http_or_missing_project_configuration() {
        assert!(
            CollectorClient::new("file:///tmp/sift", "project", None, Duration::from_secs(1))
                .is_err()
        );
        assert!(
            CollectorClient::new("http://127.0.0.1:7380", "", None, Duration::from_secs(1))
                .is_err()
        );
    }
}
// HANDWRITE-END
