use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use regex::Regex;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    AttributeValue, EventEnvelope, MetricExemplar, MetricPoint, MetricTemporality, SignalKind,
};

pub use metrics_remote_write::{proto as remote, PROMETHEUS_STALE_NAN_BITS};

#[derive(Clone, Debug)]
pub struct DecodedWrite {
    pub events: Vec<EventEnvelope>,
    pub project: String,
}

pub fn decode_remote_write(body: &[u8], admitted_project: Option<&str>) -> Result<DecodedWrite> {
    let write = metrics_remote_write::decode_write_request(body)
        .context("validate Prometheus WriteRequest")?;
    metrics_remote_write::RemoteWriteConsumer::consume(
        &SiftRemoteWriteConsumer { admitted_project },
        write,
    )
}

struct SiftRemoteWriteConsumer<'a> {
    admitted_project: Option<&'a str>,
}

impl metrics_remote_write::RemoteWriteConsumer for SiftRemoteWriteConsumer<'_> {
    type Output = DecodedWrite;
    type Error = anyhow::Error;

    fn consume(&self, write: metrics_remote_write::ValidatedWrite) -> Result<Self::Output> {
        let admitted_project = self.admitted_project;
        let request = write.into_inner();
        let metadata = request
            .metadata
            .into_iter()
            .map(|item| (item.metric_family_name, item.unit))
            .collect::<BTreeMap<_, _>>();
        let mut events = Vec::new();
        let mut request_project: Option<String> = admitted_project.map(str::to_owned);
        for series in request.timeseries {
            let labels = series
                .labels
                .into_iter()
                .map(|label| (label.name, label.value))
                .collect::<BTreeMap<_, _>>();
            let metric_name = labels
                .get("__name__")
                .filter(|name| !name.is_empty())
                .context("remote write series requires __name__ label")?;
            let project = labels
                .get("project")
                .or_else(|| labels.get("sift.project"))
                .map(String::as_str)
                .or(admitted_project)
                .context("remote write series requires project label or x-sift-project header")?;
            if admitted_project.is_some_and(|admitted| admitted != project) {
                bail!("series project `{project}` does not match admitted project");
            }
            if request_project
                .as_deref()
                .is_some_and(|known| known != project)
            {
                bail!("one remote write request cannot mix projects");
            }
            request_project = Some(project.to_owned());
            let environment = labels
                .get("environment")
                .or_else(|| labels.get("deployment.environment.name"))
                .map(String::as_str)
                .unwrap_or("default");
            for sample in series.samples {
                let stale = sample.value.to_bits() == PROMETHEUS_STALE_NAN_BITS;
                let occurred_at = Utc
                    .timestamp_millis_opt(sample.timestamp)
                    .single()
                    .context("remote write timestamp is outside the supported range")?
                    .to_rfc3339();
                let identity = serde_json::to_vec(&serde_json::json!({
                    "project": project,
                    "labels": labels,
                    "timestamp": sample.timestamp,
                    "value_bits": sample.value.to_bits()
                }))?;
                let event_id = format!("prom-rw-{}", hex::encode(&Sha256::digest(identity)[..16]));
                let mut event = EventEnvelope::for_project(
                    project,
                    environment,
                    event_id,
                    SignalKind::Metric,
                    serde_json::json!({"prometheus_remote_write": "1.0"}),
                );
                event.occurred_at = occurred_at.clone();
                event.observed_at = occurred_at;
                event
                    .resource
                    .insert("telemetry.sdk.name".into(), "prometheus".into());
                for (name, value) in &labels {
                    if matches!(
                        name.as_str(),
                        "__name__"
                            | "project"
                            | "sift.project"
                            | "environment"
                            | "deployment.environment.name"
                    ) {
                        continue;
                    }
                    if name.starts_with("service.")
                        || name.starts_with("cloud.")
                        || name.starts_with("k8s.")
                    {
                        event.resource.insert(name.clone(), value.clone());
                    } else {
                        event
                            .attributes
                            .insert(name.clone(), AttributeValue::String(value.clone()));
                    }
                }
                let exemplars = series
                    .exemplars
                    .iter()
                    .filter(|exemplar| exemplar.timestamp == sample.timestamp)
                    .filter_map(to_metric_exemplar)
                    .collect();
                event.metric = Some(MetricPoint {
                    name: metric_name.clone(),
                    value: if stale { 0.0 } else { sample.value },
                    stale,
                    unit: metadata
                        .get(metric_name)
                        .filter(|unit| !unit.is_empty())
                        .cloned(),
                    temporality: if metric_name.ends_with("_total") {
                        MetricTemporality::Cumulative
                    } else {
                        MetricTemporality::Gauge
                    },
                    exemplars,
                });
                events.push(event);
            }
        }
        Ok(DecodedWrite {
            events,
            project: request_project.context("remote write request has no project")?,
        })
    }
}

fn to_metric_exemplar(exemplar: &remote::Exemplar) -> Option<MetricExemplar> {
    let labels = exemplar
        .labels
        .iter()
        .map(|label| (label.name.as_str(), label.value.as_str()))
        .collect::<BTreeMap<_, _>>();
    Some(MetricExemplar {
        value: exemplar.value,
        trace_id: labels.get("trace_id")?.to_string(),
        span_id: labels.get("span_id")?.to_string(),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromFunction {
    Raw,
    Sum,
    Avg,
    Min,
    Max,
    Count,
    Rate,
}

#[derive(Clone, Debug)]
pub struct ParsedPromQuery {
    pub metric: String,
    pub labels: BTreeMap<String, String>,
    pub function: PromFunction,
}

pub fn parse_promql(input: &str) -> Result<ParsedPromQuery> {
    let input = input.trim();
    if input.is_empty() {
        bail!("query must not be empty");
    }
    let function = Regex::new(r"^(sum|avg|min|max|count|rate)\((.*)\)$")?;
    let (function, selector) = match function.captures(input) {
        Some(captures) => {
            let function = match &captures[1] {
                "sum" => PromFunction::Sum,
                "avg" => PromFunction::Avg,
                "min" => PromFunction::Min,
                "max" => PromFunction::Max,
                "count" => PromFunction::Count,
                "rate" => PromFunction::Rate,
                _ => unreachable!(),
            };
            (function, captures[2].trim().to_string())
        }
        None => (PromFunction::Raw, input.to_string()),
    };
    let selector_pattern = Regex::new(r#"^([a-zA-Z_:][a-zA-Z0-9_:]*)(?:\{(.*)\})?$"#)?;
    let captures = selector_pattern
        .captures(&selector)
        .context("unsupported PromQL; expected a metric selector or sum/avg/min/max/count/rate")?;
    let metric = captures[1].to_string();
    let mut labels = BTreeMap::new();
    if let Some(matchers) = captures.get(2).map(|value| value.as_str()) {
        let matcher = Regex::new(r#"^\s*([a-zA-Z_][a-zA-Z0-9_.]*)\s*=\s*\"([^\"]*)\"\s*$"#)?;
        for part in split_matchers(matchers)? {
            let captures = matcher
                .captures(part)
                .with_context(|| format!("unsupported label matcher `{part}`"))?;
            labels.insert(captures[1].to_string(), captures[2].to_string());
        }
    }
    Ok(ParsedPromQuery {
        metric,
        labels,
        function,
    })
}

fn split_matchers(input: &str) -> Result<Vec<&str>> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }
    if input.contains('\\') {
        bail!("escaped PromQL label values are not supported in phase one");
    }
    Ok(input.split(',').collect())
}

#[derive(Debug, Deserialize)]
pub struct InstantQueryParams {
    pub project: String,
    #[serde(default)]
    pub environment: Option<String>,
    pub query: String,
    #[serde(default)]
    pub time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RangeQueryParams {
    pub project: String,
    #[serde(default)]
    pub environment: Option<String>,
    pub query: String,
    pub start: String,
    pub end: String,
    pub step: String,
}

pub fn parse_prom_time_nanos(value: &str) -> Result<i64> {
    if let Ok(value) = DateTime::parse_from_rfc3339(value) {
        return value
            .timestamp_nanos_opt()
            .context("Prometheus timestamp is outside the supported range");
    }
    parse_decimal_seconds_nanos(value)
        .with_context(|| format!("invalid Prometheus timestamp `{value}`"))
}

pub fn parse_prom_duration_nanos(value: &str) -> Result<i64> {
    parse_decimal_seconds_nanos(value)
        .with_context(|| format!("invalid Prometheus duration `{value}`"))
}

fn parse_decimal_seconds_nanos(value: &str) -> Result<i64> {
    let (negative, unsigned) = match value.as_bytes().first() {
        Some(b'-') => (true, &value[1..]),
        Some(b'+') => (false, &value[1..]),
        _ => (false, value),
    };
    if unsigned.is_empty() {
        bail!("decimal seconds must contain digits");
    }
    let mut exponent_parts = unsigned.split(['e', 'E']);
    let mantissa = exponent_parts.next().unwrap_or_default();
    let exponent = exponent_parts
        .next()
        .map(str::parse::<i32>)
        .transpose()
        .context("decimal exponent is invalid")?
        .unwrap_or(0);
    if exponent_parts.next().is_some() {
        bail!("decimal seconds contain more than one exponent");
    }
    let mut mantissa_parts = mantissa.split('.');
    let integer = mantissa_parts.next().unwrap_or_default();
    let fraction = mantissa_parts.next().unwrap_or_default();
    if mantissa_parts.next().is_some()
        || (integer.is_empty() && fraction.is_empty())
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        bail!("decimal seconds have an invalid mantissa");
    }
    let digits = format!("{integer}{fraction}");
    let coefficient = digits
        .parse::<i128>()
        .context("decimal seconds exceed the supported precision")?;
    if coefficient == 0 {
        return Ok(0);
    }
    let fraction_digits =
        i128::try_from(fraction.len()).context("decimal seconds exceed the supported precision")?;
    let power = i128::from(exponent) - fraction_digits + 9;
    let magnitude = if power >= 0 {
        let power = u32::try_from(power).context("decimal seconds are outside nanosecond range")?;
        coefficient
            .checked_mul(
                10_i128
                    .checked_pow(power)
                    .context("decimal seconds are outside nanosecond range")?,
            )
            .context("decimal seconds are outside nanosecond range")?
    } else {
        let divisor_power =
            u32::try_from(-power).context("decimal seconds are outside nanosecond range")?;
        let Some(divisor) = 10_i128.checked_pow(divisor_power) else {
            return Ok(0);
        };
        let quotient = coefficient / divisor;
        let remainder = coefficient % divisor;
        quotient + i128::from(remainder >= (divisor + 1) / 2)
    };
    let nanos = if negative {
        magnitude
            .checked_neg()
            .context("decimal seconds are outside nanosecond range")?
    } else {
        magnitude
    };
    i64::try_from(nanos).context("decimal seconds are outside nanosecond range")
}

pub fn nanos_rfc3339(nanos: i64) -> Result<String> {
    let seconds = nanos.div_euclid(1_000_000_000);
    let subsecond = nanos.rem_euclid(1_000_000_000) as u32;
    Ok(DateTime::<Utc>::from_timestamp(seconds, subsecond)
        .context("Prometheus timestamp is outside the supported range")?
        .to_rfc3339_opts(SecondsFormat::Nanos, true))
}

#[cfg(test)]
mod tests {
    use super::{nanos_rfc3339, parse_prom_time_nanos};

    #[test]
    fn prometheus_times_keep_exact_nanoseconds() {
        assert_eq!(
            parse_prom_time_nanos("1783987200.000999600").unwrap(),
            1_783_987_200_000_999_600
        );
        assert_eq!(
            parse_prom_time_nanos("2026-07-14T00:00:00.000999600Z").unwrap(),
            1_783_987_200_000_999_600
        );
        assert_eq!(
            nanos_rfc3339(1_783_987_200_000_999_600).unwrap(),
            "2026-07-14T00:00:00.000999600Z"
        );
    }
}
