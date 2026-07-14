// HANDWRITE-BEGIN gap="sift-otlp-normalizer" tracker="1658" reason="Decode signal endpoint payloads, dispatch wire normalization, and encode media-type-matched partial-success responses."
pub mod wire;

use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Utc};
use prost::Message;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use crate::{
    AttributeValue, InstrumentationScope, MetricExemplar, MetricPoint, MetricTemporality,
    OperationalEventV2, SignalKind,
};

use self::wire::{any_value, metric, number_data_point, AnyValue};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OtlpSignal {
    Logs,
    Traces,
    Metrics,
    Profiles,
}

impl OtlpSignal {
    pub fn rejected_json_field(self) -> &'static str {
        match self {
            Self::Logs => "rejectedLogRecords",
            Self::Traces => "rejectedSpans",
            Self::Metrics => "rejectedDataPoints",
            Self::Profiles => "rejectedProfiles",
        }
    }

    fn signal_kind(self) -> SignalKind {
        match self {
            Self::Logs => SignalKind::Log,
            Self::Traces => SignalKind::Span,
            Self::Metrics => SignalKind::Metric,
            Self::Profiles => SignalKind::Profile,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OtlpMediaType {
    Json,
    Protobuf,
}

impl OtlpMediaType {
    pub fn parse(value: Option<&str>) -> Result<Self> {
        let value = value.unwrap_or("application/json");
        let media = value.split(';').next().unwrap_or(value).trim();
        match media {
            "application/json" => Ok(Self::Json),
            "application/x-protobuf" | "application/protobuf" => Ok(Self::Protobuf),
            other => bail!("unsupported OTLP content-type `{other}`"),
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Protobuf => "application/x-protobuf",
        }
    }
}

#[derive(Clone, Debug)]
pub struct OtlpItemError {
    pub event_id: Option<String>,
    pub message: String,
}

pub struct DecodedOtlp {
    pub items: Vec<std::result::Result<OperationalEventV2, OtlpItemError>>,
}

impl DecodedOtlp {
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

pub struct EncodedOtlpResponse {
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

pub fn decode(
    signal: OtlpSignal,
    media: OtlpMediaType,
    body: &[u8],
    project: &str,
) -> Result<DecodedOtlp> {
    let items = match (signal, media) {
        (OtlpSignal::Logs, OtlpMediaType::Json) => decode_logs_json(body, project)?,
        (OtlpSignal::Logs, OtlpMediaType::Protobuf) => decode_logs_proto(body, project)?,
        (OtlpSignal::Traces, OtlpMediaType::Json) => decode_traces_json(body, project)?,
        (OtlpSignal::Traces, OtlpMediaType::Protobuf) => decode_traces_proto(body, project)?,
        (OtlpSignal::Metrics, OtlpMediaType::Json) => decode_metrics_json(body, project)?,
        (OtlpSignal::Metrics, OtlpMediaType::Protobuf) => decode_metrics_proto(body, project)?,
        (OtlpSignal::Profiles, OtlpMediaType::Json) => decode_profiles_json(body, project)?,
        (OtlpSignal::Profiles, OtlpMediaType::Protobuf) => decode_profiles_proto(body, project)?,
    };
    if items.is_empty() {
        bail!("OTLP request contains no signal items");
    }
    Ok(DecodedOtlp { items })
}

pub fn encode_response(
    signal: OtlpSignal,
    media: OtlpMediaType,
    rejected: usize,
    messages: &[String],
) -> Result<EncodedOtlpResponse> {
    let error_message = messages.join("; ");
    let body = match media {
        OtlpMediaType::Json => {
            let value = if rejected == 0 && error_message.is_empty() {
                json!({})
            } else {
                let mut partial = Map::new();
                partial.insert(signal.rejected_json_field().into(), json!(rejected));
                partial.insert("errorMessage".into(), json!(error_message));
                json!({"partialSuccess": partial})
            };
            serde_json::to_vec(&value)?
        }
        OtlpMediaType::Protobuf => wire::ExportResponse {
            partial_success: (rejected != 0 || !error_message.is_empty()).then_some(
                wire::PartialSuccess {
                    rejected_items: rejected as i64,
                    error_message,
                },
            ),
        }
        .encode_to_vec(),
    };
    Ok(EncodedOtlpResponse {
        content_type: media.content_type(),
        body,
    })
}

fn decode_logs_json(
    body: &[u8],
    project: &str,
) -> Result<Vec<std::result::Result<OperationalEventV2, OtlpItemError>>> {
    let root: Value = serde_json::from_slice(body).context("decode OTLP logs JSON")?;
    let mut output = Vec::new();
    for resource_logs in array(&root, "resourceLogs", "resource_logs") {
        let resource = json_resource(resource_logs.get("resource"));
        for scope_logs in array(resource_logs, "scopeLogs", "scope_logs") {
            let scope = json_scope(
                scope_logs.get("scope"),
                string(scope_logs, "schemaUrl", "schema_url"),
            );
            for record in array(scope_logs, "logRecords", "log_records") {
                output.push(json_log_event(record, project, &resource, scope.clone()));
            }
        }
    }
    Ok(output)
}

fn json_log_event(
    record: &Value,
    project: &str,
    resource: &BTreeMap<String, String>,
    scope: Option<InstrumentationScope>,
) -> std::result::Result<OperationalEventV2, OtlpItemError> {
    let body = record
        .get("body")
        .filter(|value| !value.is_null())
        .and_then(json_any_value)
        .filter(|value| !value.is_null());
    if body.is_none() {
        return Err(item_error(None, "log record body is required"));
    }
    let occurred_nanos = nanos(record, "timeUnixNano", "time_unix_nano");
    let observed_nanos = nanos(record, "observedTimeUnixNano", "observed_time_unix_nano");
    let trace_id = id_string(record.get("traceId").or_else(|| record.get("trace_id")));
    let span_id = id_string(record.get("spanId").or_else(|| record.get("span_id")));
    let identity = format!(
        "{}:{}:{}:{}",
        trace_id.as_deref().unwrap_or(""),
        span_id.as_deref().unwrap_or(""),
        occurred_nanos,
        serde_json::to_string(record).unwrap_or_default()
    );
    let mut event = base_event(
        OtlpSignal::Logs,
        project,
        resource,
        stable_id("log", project, &identity),
        occurred_nanos,
        observed_nanos,
        record.clone(),
    );
    event.instrumentation_scope = scope;
    event.attributes = json_attributes(record.get("attributes"));
    event.trace_id = trace_id;
    event.span_id = span_id;
    event.severity = string(record, "severityText", "severity_text").map(str::to_string);
    if let Some(body) = record.get("body").and_then(json_attribute_value) {
        event.attributes.insert("otel.log.body".into(), body);
    }
    Ok(event)
}

fn decode_logs_proto(
    body: &[u8],
    project: &str,
) -> Result<Vec<std::result::Result<OperationalEventV2, OtlpItemError>>> {
    let request =
        wire::ExportLogsServiceRequest::decode(body).context("decode OTLP logs protobuf")?;
    let mut output = Vec::new();
    for resource_logs in request.resource_logs {
        let resource = proto_resource(resource_logs.resource.as_ref());
        for scope_logs in resource_logs.scope_logs {
            let scope = proto_scope(scope_logs.scope.as_ref(), &scope_logs.schema_url);
            for record in scope_logs.log_records {
                if record.body.is_none() {
                    output.push(Err(item_error(None, "log record body is required")));
                    continue;
                }
                let identity = format!(
                    "{}:{}:{}:{}",
                    hex::encode(&record.trace_id),
                    hex::encode(&record.span_id),
                    record.time_unix_nano,
                    hex::encode(record.encode_to_vec())
                );
                let mut event = base_event(
                    OtlpSignal::Logs,
                    project,
                    &resource,
                    stable_id("log", project, &identity),
                    record.time_unix_nano,
                    record.observed_time_unix_nano,
                    json!({
                        "body": record.body.as_ref().map(proto_any_json),
                        "severityNumber": record.severity_number,
                        "severityText": record.severity_text,
                        "flags": record.flags,
                        "droppedAttributesCount": record.dropped_attributes_count,
                    }),
                );
                event.instrumentation_scope = scope.clone();
                event.attributes = proto_attributes(&record.attributes);
                event.trace_id = valid_proto_id(&record.trace_id, 16);
                event.span_id = valid_proto_id(&record.span_id, 8);
                event.severity = (!record.severity_text.is_empty()).then_some(record.severity_text);
                output.push(Ok(event));
            }
        }
    }
    Ok(output)
}

fn decode_traces_json(
    body: &[u8],
    project: &str,
) -> Result<Vec<std::result::Result<OperationalEventV2, OtlpItemError>>> {
    let root: Value = serde_json::from_slice(body).context("decode OTLP traces JSON")?;
    let mut output = Vec::new();
    for resource_spans in array(&root, "resourceSpans", "resource_spans") {
        let resource = json_resource(resource_spans.get("resource"));
        for scope_spans in array(resource_spans, "scopeSpans", "scope_spans") {
            let scope = json_scope(
                scope_spans.get("scope"),
                string(scope_spans, "schemaUrl", "schema_url"),
            );
            for span in array(scope_spans, "spans", "spans") {
                let trace_id = id_string(span.get("traceId").or_else(|| span.get("trace_id")));
                let span_id = id_string(span.get("spanId").or_else(|| span.get("span_id")));
                let name = string(span, "name", "name").unwrap_or("");
                if trace_id.is_none() || span_id.is_none() || name.is_empty() {
                    output.push(Err(item_error(
                        span_id,
                        "span requires traceId, spanId, and name",
                    )));
                    continue;
                }
                let start = nanos(span, "startTimeUnixNano", "start_time_unix_nano");
                let end = nanos(span, "endTimeUnixNano", "end_time_unix_nano");
                let mut event = base_event(
                    OtlpSignal::Traces,
                    project,
                    &resource,
                    stable_id(
                        "span",
                        project,
                        &format!(
                            "{}:{}",
                            trace_id.as_deref().unwrap_or(""),
                            span_id.as_deref().unwrap_or(name)
                        ),
                    ),
                    start,
                    end,
                    span.clone(),
                );
                event.instrumentation_scope = scope.clone();
                event.attributes = json_attributes(span.get("attributes"));
                event.trace_id = trace_id;
                event.span_id = span_id;
                output.push(Ok(event));
            }
        }
    }
    Ok(output)
}

fn decode_traces_proto(
    body: &[u8],
    project: &str,
) -> Result<Vec<std::result::Result<OperationalEventV2, OtlpItemError>>> {
    let request =
        wire::ExportTraceServiceRequest::decode(body).context("decode OTLP traces protobuf")?;
    let mut output = Vec::new();
    for resource_spans in request.resource_spans {
        let resource = proto_resource(resource_spans.resource.as_ref());
        for scope_spans in resource_spans.scope_spans {
            let scope = proto_scope(scope_spans.scope.as_ref(), &scope_spans.schema_url);
            for span in scope_spans.spans {
                let trace_id = valid_proto_id(&span.trace_id, 16);
                let span_id = valid_proto_id(&span.span_id, 8);
                if trace_id.is_none() || span_id.is_none() || span.name.is_empty() {
                    output.push(Err(item_error(
                        span_id,
                        "span requires valid trace_id, span_id, and name",
                    )));
                    continue;
                }
                let mut event = base_event(
                    OtlpSignal::Traces,
                    project,
                    &resource,
                    stable_id(
                        "span",
                        project,
                        &format!(
                            "{}:{}",
                            trace_id.as_deref().unwrap_or(""),
                            span_id.as_deref().unwrap_or(&span.name)
                        ),
                    ),
                    span.start_time_unix_nano,
                    span.end_time_unix_nano,
                    json!({
                        "name": span.name,
                        "kind": span.kind,
                        "parentSpanId": hex::encode(span.parent_span_id),
                        "events": span.events.iter().map(|event| json!({"name": event.name, "timeUnixNano": event.time_unix_nano})).collect::<Vec<_>>(),
                        "links": span.links.iter().map(|link| json!({"traceId": hex::encode(&link.trace_id), "spanId": hex::encode(&link.span_id)})).collect::<Vec<_>>(),
                        "status": span.status.as_ref().map(|status| json!({"code": status.code, "message": status.message})),
                    }),
                );
                event.instrumentation_scope = scope.clone();
                event.attributes = proto_attributes(&span.attributes);
                event.trace_id = trace_id;
                event.span_id = span_id;
                output.push(Ok(event));
            }
        }
    }
    Ok(output)
}

fn decode_metrics_json(
    body: &[u8],
    project: &str,
) -> Result<Vec<std::result::Result<OperationalEventV2, OtlpItemError>>> {
    let root: Value = serde_json::from_slice(body).context("decode OTLP metrics JSON")?;
    let mut output = Vec::new();
    for resource_metrics in array(&root, "resourceMetrics", "resource_metrics") {
        let resource = json_resource(resource_metrics.get("resource"));
        for scope_metrics in array(resource_metrics, "scopeMetrics", "scope_metrics") {
            let scope = json_scope(
                scope_metrics.get("scope"),
                string(scope_metrics, "schemaUrl", "schema_url"),
            );
            for metric in array(scope_metrics, "metrics", "metrics") {
                let name = string(metric, "name", "name").unwrap_or("");
                let unit = string(metric, "unit", "unit").map(str::to_string);
                let (data, temporality) = if let Some(gauge) = metric.get("gauge") {
                    (gauge, MetricTemporality::Gauge)
                } else if let Some(sum) = metric.get("sum") {
                    (sum, json_temporality(sum))
                } else if let Some(histogram) = metric.get("histogram") {
                    (histogram, json_temporality(histogram))
                } else {
                    output.push(Err(item_error(
                        None,
                        format!("metric `{name}` has unsupported data"),
                    )));
                    continue;
                };
                for point in array(data, "dataPoints", "data_points") {
                    let value =
                        json_number(point).or_else(|| point.get("sum").and_then(Value::as_f64));
                    let Some(value) = value else {
                        output.push(Err(item_error(
                            None,
                            format!("metric `{name}` point has no numeric value"),
                        )));
                        continue;
                    };
                    let time = nanos(point, "timeUnixNano", "time_unix_nano");
                    let identity = format!(
                        "{name}:{time}:{}",
                        serde_json::to_string(point).unwrap_or_default()
                    );
                    let mut event = base_event(
                        OtlpSignal::Metrics,
                        project,
                        &resource,
                        stable_id("metric", project, &identity),
                        time,
                        time,
                        json!({"metric": metric, "point": point}),
                    );
                    event.instrumentation_scope = scope.clone();
                    event.attributes = json_attributes(point.get("attributes"));
                    event.metric = Some(MetricPoint {
                        name: name.to_string(),
                        value,
                        unit: unit.clone(),
                        temporality,
                        exemplars: json_exemplars(point.get("exemplars")),
                    });
                    output.push(Ok(event));
                }
            }
        }
    }
    Ok(output)
}

fn decode_metrics_proto(
    body: &[u8],
    project: &str,
) -> Result<Vec<std::result::Result<OperationalEventV2, OtlpItemError>>> {
    let request =
        wire::ExportMetricsServiceRequest::decode(body).context("decode OTLP metrics protobuf")?;
    let mut output = Vec::new();
    for resource_metrics in request.resource_metrics {
        let resource = proto_resource(resource_metrics.resource.as_ref());
        for scope_metrics in resource_metrics.scope_metrics {
            let scope = proto_scope(scope_metrics.scope.as_ref(), &scope_metrics.schema_url);
            for metric in scope_metrics.metrics {
                let name = metric.name.clone();
                let unit = (!metric.unit.is_empty()).then_some(metric.unit.clone());
                let Some(data) = metric.data else {
                    output.push(Err(item_error(
                        None,
                        format!("metric `{name}` has no data"),
                    )));
                    continue;
                };
                let (points, temporality): (Vec<wire::NumberDataPoint>, MetricTemporality) =
                    match data {
                        metric::Data::Gauge(gauge) => (gauge.data_points, MetricTemporality::Gauge),
                        metric::Data::Sum(sum) => (
                            sum.data_points,
                            proto_temporality(sum.aggregation_temporality),
                        ),
                        metric::Data::Histogram(histogram) => {
                            for point in histogram.data_points {
                                let value = point.sum.unwrap_or(point.count as f64);
                                output.push(Ok(proto_metric_event(
                                project,
                                &resource,
                                scope.clone(),
                                &name,
                                unit.clone(),
                                value,
                                proto_temporality(histogram.aggregation_temporality),
                                point.time_unix_nano,
                                proto_attributes(&point.attributes),
                                proto_exemplars(&point.exemplars),
                                json!({"count": point.count, "sum": point.sum, "bucketCounts": point.bucket_counts, "explicitBounds": point.explicit_bounds}),
                            )));
                            }
                            continue;
                        }
                    };
                for point in points {
                    let Some(value) = point.value.and_then(proto_number) else {
                        output.push(Err(item_error(
                            None,
                            format!("metric `{name}` point has no value"),
                        )));
                        continue;
                    };
                    output.push(Ok(proto_metric_event(
                        project,
                        &resource,
                        scope.clone(),
                        &name,
                        unit.clone(),
                        value,
                        temporality,
                        point.time_unix_nano,
                        proto_attributes(&point.attributes),
                        proto_exemplars(&point.exemplars),
                        json!({"startTimeUnixNano": point.start_time_unix_nano, "flags": point.flags}),
                    )));
                }
            }
        }
    }
    Ok(output)
}

#[allow(clippy::too_many_arguments)]
fn proto_metric_event(
    project: &str,
    resource: &BTreeMap<String, String>,
    scope: Option<InstrumentationScope>,
    name: &str,
    unit: Option<String>,
    value: f64,
    temporality: MetricTemporality,
    time: u64,
    attributes: BTreeMap<String, AttributeValue>,
    exemplars: Vec<MetricExemplar>,
    payload: Value,
) -> OperationalEventV2 {
    let identity = format!(
        "{name}:{time}:{value}:{}:{}",
        serde_json::to_string(&attributes).unwrap_or_default(),
        serde_json::to_string(&payload).unwrap_or_default()
    );
    let mut event = base_event(
        OtlpSignal::Metrics,
        project,
        resource,
        stable_id("metric", project, &identity),
        time,
        time,
        payload,
    );
    event.instrumentation_scope = scope;
    event.attributes = attributes;
    event.metric = Some(MetricPoint {
        name: name.to_string(),
        value,
        unit,
        temporality,
        exemplars,
    });
    event
}

fn decode_profiles_json(
    body: &[u8],
    project: &str,
) -> Result<Vec<std::result::Result<OperationalEventV2, OtlpItemError>>> {
    let root: Value = serde_json::from_slice(body).context("decode OTLP profiles JSON")?;
    let mut output = Vec::new();
    for resource_profiles in array(&root, "resourceProfiles", "resource_profiles") {
        let resource = json_resource(resource_profiles.get("resource"));
        for scope_profiles in array(resource_profiles, "scopeProfiles", "scope_profiles") {
            let scope = json_scope(
                scope_profiles.get("scope"),
                string(scope_profiles, "schemaUrl", "schema_url"),
            );
            let profiles = {
                let direct = array(scope_profiles, "profiles", "profiles");
                if direct.is_empty() {
                    array(scope_profiles, "profileContainers", "profile_containers")
                } else {
                    direct
                }
            };
            for profile in profiles {
                let start = nanos(profile, "startTimeUnixNano", "start_time_unix_nano");
                let profile_id = id_string(
                    profile
                        .get("profileId")
                        .or_else(|| profile.get("profile_id")),
                )
                .unwrap_or_else(|| {
                    stable_id(
                        "profile-source",
                        project,
                        &serde_json::to_string(profile).unwrap_or_default(),
                    )
                });
                let mut event = base_event(
                    OtlpSignal::Profiles,
                    project,
                    &resource,
                    stable_id("profile", project, &profile_id),
                    start,
                    start,
                    profile.clone(),
                );
                event.instrumentation_scope = scope.clone();
                output.push(Ok(event));
            }
        }
    }
    Ok(output)
}

fn decode_profiles_proto(
    body: &[u8],
    project: &str,
) -> Result<Vec<std::result::Result<OperationalEventV2, OtlpItemError>>> {
    let request = wire::ExportProfilesServiceRequest::decode(body)
        .context("decode OTLP profiles protobuf envelope")?;
    if request.resource_profiles.is_empty() {
        bail!("OTLP profiles protobuf contains no resource_profiles");
    }
    let now = Utc::now().to_rfc3339();
    Ok(request
        .resource_profiles
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let mut event = OperationalEventV2::for_project(
                project,
                "default",
                stable_id(
                    "profile-protobuf",
                    project,
                    &format!("{}:{index}", hex::encode(body)),
                ),
                SignalKind::Profile,
                json!({
                    "encoding": "otlp-protobuf",
                    "resourceIndex": index,
                    "requestBytesBase64": BASE64.encode(body),
                    "decodeStatus": "opaque-until-profile-store-v1",
                }),
            );
            event.occurred_at = now.clone();
            event.observed_at = now.clone();
            event
                .resource
                .insert("service.name".into(), "unknown".into());
            Ok(event)
        })
        .collect())
}

fn base_event(
    signal: OtlpSignal,
    project_hint: &str,
    resource: &BTreeMap<String, String>,
    event_id: String,
    occurred_nanos: u64,
    observed_nanos: u64,
    payload: Value,
) -> OperationalEventV2 {
    let project = resource
        .get("gcp.project_id")
        .or_else(|| resource.get("cloud.account.id"))
        .or_else(|| resource.get("project.id"))
        .map(String::as_str)
        .unwrap_or(project_hint);
    let environment = resource
        .get("deployment.environment.name")
        .or_else(|| resource.get("environment"))
        .map(String::as_str)
        .unwrap_or("default");
    let occurred_at = nanos_to_rfc3339(occurred_nanos);
    let observed_at = nanos_to_rfc3339(if observed_nanos == 0 {
        occurred_nanos
    } else {
        observed_nanos
    });
    let mut event = OperationalEventV2::for_project(
        project,
        environment,
        event_id,
        signal.signal_kind(),
        payload,
    );
    event.occurred_at = occurred_at;
    event.observed_at = observed_at;
    event.resource = resource.clone();
    if event.resource.is_empty() {
        event
            .resource
            .insert("service.name".into(), "unknown".into());
    }
    event
}

fn stable_id(kind: &str, project: &str, identity: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(kind.as_bytes());
    hash.update([0]);
    hash.update(project.as_bytes());
    hash.update([0]);
    hash.update(identity.as_bytes());
    format!("otlp-{kind}-{}", hex::encode(&hash.finalize()[..16]))
}

fn nanos_to_rfc3339(value: u64) -> String {
    if value == 0 {
        return Utc::now().to_rfc3339();
    }
    DateTime::<Utc>::from_timestamp(
        (value / 1_000_000_000) as i64,
        (value % 1_000_000_000) as u32,
    )
    .unwrap_or_else(Utc::now)
    .to_rfc3339()
}

fn array<'a>(value: &'a Value, camel: &str, snake: &str) -> &'a [Value] {
    value
        .get(camel)
        .or_else(|| value.get(snake))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn string<'a>(value: &'a Value, camel: &str, snake: &str) -> Option<&'a str> {
    value
        .get(camel)
        .or_else(|| value.get(snake))
        .and_then(Value::as_str)
}

fn nanos(value: &Value, camel: &str, snake: &str) -> u64 {
    value
        .get(camel)
        .or_else(|| value.get(snake))
        .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
        .unwrap_or(0)
}

fn id_string(value: Option<&Value>) -> Option<String> {
    let source = value?.as_str()?;
    if source.is_empty() {
        return None;
    }
    if source.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Some(source.to_ascii_lowercase());
    }
    BASE64
        .decode(source)
        .ok()
        .map(hex::encode)
        .or_else(|| Some(source.to_string()))
}

fn json_resource(resource: Option<&Value>) -> BTreeMap<String, String> {
    let mut output = BTreeMap::new();
    if let Some(resource) = resource {
        for (key, value) in json_key_values(resource.get("attributes")) {
            if let Some(value) = scalar_string(&value) {
                output.insert(key, value);
            }
        }
    }
    if output.is_empty() {
        output.insert("service.name".into(), "unknown".into());
    }
    output
}

fn json_scope(scope: Option<&Value>, schema_url: Option<&str>) -> Option<InstrumentationScope> {
    let scope = scope?;
    Some(InstrumentationScope {
        name: string(scope, "name", "name")
            .unwrap_or("unknown")
            .to_string(),
        version: string(scope, "version", "version").map(str::to_string),
        attributes: json_attributes(scope.get("attributes")),
        schema_url: schema_url
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    })
}

fn json_key_values(value: Option<&Value>) -> Vec<(String, Value)> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let key = entry.get("key")?.as_str()?.to_string();
            let value = entry.get("value").and_then(json_any_value)?;
            Some((key, value))
        })
        .collect()
}

fn json_attributes(value: Option<&Value>) -> BTreeMap<String, AttributeValue> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            Some((
                entry.get("key")?.as_str()?.to_string(),
                json_attribute_value(entry.get("value")?)?,
            ))
        })
        .collect()
}

fn json_attribute_value(value: &Value) -> Option<AttributeValue> {
    if let Some(value) = value.get("stringValue").and_then(Value::as_str) {
        return Some(AttributeValue::String(value.to_string()));
    }
    if let Some(value) = value.get("boolValue").and_then(Value::as_bool) {
        return Some(AttributeValue::Bool(value));
    }
    if let Some(value) = value.get("intValue") {
        return value
            .as_i64()
            .or_else(|| value.as_str()?.parse().ok())
            .map(AttributeValue::Int);
    }
    if let Some(value) = value.get("doubleValue") {
        return value
            .as_f64()
            .or_else(|| value.as_str()?.parse().ok())
            .map(AttributeValue::Double);
    }
    if let Some(value) = value.get("bytesValue").and_then(Value::as_str) {
        return Some(AttributeValue::Bytes(value.to_string()));
    }
    if let Some(value) = value.get("arrayValue") {
        return Some(AttributeValue::Array(
            array(value, "values", "values")
                .iter()
                .filter_map(json_attribute_value)
                .collect(),
        ));
    }
    if let Some(value) = value.get("kvlistValue") {
        return Some(AttributeValue::Map(
            value
                .get("values")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|entry| {
                    Some((
                        entry.get("key")?.as_str()?.to_string(),
                        json_attribute_value(entry.get("value")?)?,
                    ))
                })
                .collect(),
        ));
    }
    (!value.is_null()).then(|| json_to_attribute(value.clone()))
}

fn json_any_value(value: &Value) -> Option<Value> {
    if !value.is_object() {
        return Some(value.clone());
    }
    for key in ["stringValue", "boolValue", "intValue", "doubleValue"] {
        if let Some(value) = value.get(key) {
            return Some(value.clone());
        }
    }
    if let Some(bytes) = value.get("bytesValue").and_then(Value::as_str) {
        return Some(json!({"bytesBase64": bytes}));
    }
    if let Some(array_value) = value.get("arrayValue") {
        return Some(Value::Array(
            array(array_value, "values", "values")
                .iter()
                .filter_map(json_any_value)
                .collect(),
        ));
    }
    if let Some(map) = value.get("kvlistValue") {
        return Some(Value::Object(
            json_key_values(map.get("values")).into_iter().collect(),
        ));
    }
    Some(value.clone())
}

fn json_to_attribute(value: Value) -> AttributeValue {
    match value {
        Value::String(value) => AttributeValue::String(value),
        Value::Bool(value) => AttributeValue::Bool(value),
        Value::Number(value) => value
            .as_i64()
            .map(AttributeValue::Int)
            .unwrap_or_else(|| AttributeValue::Double(value.as_f64().unwrap_or_default())),
        Value::Array(values) => {
            AttributeValue::Array(values.into_iter().map(json_to_attribute).collect())
        }
        Value::Object(values) => AttributeValue::Map(
            values
                .into_iter()
                .map(|(key, value)| (key, json_to_attribute(value)))
                .collect(),
        ),
        Value::Null => AttributeValue::String("null".into()),
    }
}

fn scalar_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn json_number(point: &Value) -> Option<f64> {
    point
        .get("asDouble")
        .or_else(|| point.get("as_double"))
        .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))
        .or_else(|| {
            point
                .get("asInt")
                .or_else(|| point.get("as_int"))
                .and_then(|value| {
                    value
                        .as_i64()
                        .map(|value| value as f64)
                        .or_else(|| value.as_str()?.parse().ok())
                })
        })
}

fn json_temporality(value: &Value) -> MetricTemporality {
    match value
        .get("aggregationTemporality")
        .or_else(|| value.get("aggregation_temporality"))
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
    {
        Some(1) => MetricTemporality::Delta,
        _ => MetricTemporality::Cumulative,
    }
}

fn json_exemplars(value: Option<&Value>) -> Vec<MetricExemplar> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|exemplar| {
            Some(MetricExemplar {
                value: json_number(exemplar)?,
                trace_id: id_string(exemplar.get("traceId").or_else(|| exemplar.get("trace_id")))?,
                span_id: id_string(exemplar.get("spanId").or_else(|| exemplar.get("span_id")))?,
            })
        })
        .collect()
}

fn proto_resource(resource: Option<&wire::Resource>) -> BTreeMap<String, String> {
    let mut output: BTreeMap<String, String> = resource
        .map(|resource| {
            resource
                .attributes
                .iter()
                .filter_map(|entry| {
                    let value = entry.value.as_ref().map(proto_any_json)?;
                    Some((entry.key.clone(), scalar_string(&value)?))
                })
                .collect()
        })
        .unwrap_or_default();
    if output.is_empty() {
        output.insert("service.name".into(), "unknown".into());
    }
    output
}

fn proto_scope(
    scope: Option<&wire::InstrumentationScope>,
    schema_url: &str,
) -> Option<InstrumentationScope> {
    let scope = scope?;
    Some(InstrumentationScope {
        name: if scope.name.is_empty() {
            "unknown".into()
        } else {
            scope.name.clone()
        },
        version: (!scope.version.is_empty()).then(|| scope.version.clone()),
        attributes: proto_attributes(&scope.attributes),
        schema_url: (!schema_url.is_empty()).then(|| schema_url.to_string()),
    })
}

fn proto_attributes(values: &[wire::KeyValue]) -> BTreeMap<String, AttributeValue> {
    values
        .iter()
        .filter_map(|entry| {
            Some((
                entry.key.clone(),
                json_to_attribute(proto_any_json(entry.value.as_ref()?)),
            ))
        })
        .collect()
}

fn proto_any_json(value: &AnyValue) -> Value {
    match value.value.as_ref() {
        Some(any_value::Value::StringValue(value)) => json!(value),
        Some(any_value::Value::BoolValue(value)) => json!(value),
        Some(any_value::Value::IntValue(value)) => json!(value),
        Some(any_value::Value::DoubleValue(value)) => json!(value),
        Some(any_value::Value::BytesValue(value)) => json!({"bytesBase64": BASE64.encode(value)}),
        Some(any_value::Value::ArrayValue(value)) => {
            Value::Array(value.values.iter().map(proto_any_json).collect())
        }
        Some(any_value::Value::KvlistValue(value)) => Value::Object(
            value
                .values
                .iter()
                .filter_map(|entry| {
                    Some((entry.key.clone(), proto_any_json(entry.value.as_ref()?)))
                })
                .collect(),
        ),
        None => Value::Null,
    }
}

fn valid_proto_id(bytes: &[u8], expected: usize) -> Option<String> {
    (bytes.len() == expected && bytes.iter().any(|byte| *byte != 0)).then(|| hex::encode(bytes))
}

fn proto_temporality(value: i32) -> MetricTemporality {
    if value == 1 {
        MetricTemporality::Delta
    } else {
        MetricTemporality::Cumulative
    }
}

fn proto_number(value: number_data_point::Value) -> Option<f64> {
    match value {
        number_data_point::Value::AsDouble(value) if value.is_finite() => Some(value),
        number_data_point::Value::AsInt(value) => Some(value as f64),
        _ => None,
    }
}

fn proto_exemplars(values: &[wire::Exemplar]) -> Vec<MetricExemplar> {
    values
        .iter()
        .filter_map(|value| {
            Some(MetricExemplar {
                value: value.value.clone().and_then(|value| match value {
                    wire::exemplar::Value::AsDouble(value) if value.is_finite() => Some(value),
                    wire::exemplar::Value::AsInt(value) => Some(value as f64),
                    _ => None,
                })?,
                trace_id: valid_proto_id(&value.trace_id, 16)?,
                span_id: valid_proto_id(&value.span_id, 8)?,
            })
        })
        .collect()
}

fn item_error(event_id: Option<String>, message: impl Into<String>) -> OtlpItemError {
    OtlpItemError {
        event_id,
        message: message.into(),
    }
}
// HANDWRITE-END
