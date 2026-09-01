use std::{collections::BTreeMap, time::Duration};

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

pub const QUERY_AST_VERSION: u32 = 1;
pub const DEFAULT_QUERY_LIMIT: usize = 100;
pub const MAX_QUERY_LIMIT: usize = 1_000;
const MAX_FILTER_DEPTH: usize = 32;
const MAX_FILTER_ARGUMENTS: usize = 64;
const MAX_REGEX_BYTES: usize = 1_024;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TimeRangeV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryModeV1 {
    #[default]
    Auto,
    Sync,
    Async,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricFunctionV1 {
    #[default]
    Raw,
    Sum,
    Avg,
    Min,
    Max,
    Count,
    Rate,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum QuerySignalV1 {
    Logs {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<QueryExpressionV1>,
    },
    Metrics {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default)]
        function: MetricFunctionV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        step_seconds: Option<u64>,
        #[serde(default)]
        group_by: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<QueryExpressionV1>,
    },
    Traces {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        service: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        operation: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min_duration_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_duration_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        #[serde(default)]
        attributes: BTreeMap<String, serde_json::Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<QueryExpressionV1>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum QueryExpressionV1 {
    And {
        args: Vec<QueryExpressionV1>,
    },
    Or {
        args: Vec<QueryExpressionV1>,
    },
    Not {
        arg: Box<QueryExpressionV1>,
    },
    Eq {
        field: String,
        value: serde_json::Value,
    },
    In {
        field: String,
        values: Vec<serde_json::Value>,
    },
    Exists {
        field: String,
    },
    Range {
        field: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gt: Option<serde_json::Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gte: Option<serde_json::Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lt: Option<serde_json::Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lte: Option<serde_json::Value>,
    },
    Text {
        field: String,
        value: String,
    },
    Regex {
        field: String,
        pattern: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QueryRequestV1 {
    pub version: u32,
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default)]
    pub time_range: TimeRangeV1,
    pub signal: QuerySignalV1,
    #[serde(default = "default_query_limit")]
    pub limit: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(default)]
    pub mode: QueryModeV1,
}

impl QueryRequestV1 {
    pub fn validate(&self) -> Result<()> {
        if self.version != QUERY_AST_VERSION {
            bail!(
                "unsupported query version {}; expected {}",
                self.version,
                QUERY_AST_VERSION
            );
        }
        if self.project.trim().is_empty() {
            bail!("project must not be empty");
        }
        if self
            .environment
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            bail!("environment must not be empty");
        }
        if self.limit == 0 || self.limit > MAX_QUERY_LIMIT {
            bail!("limit must be between 1 and {MAX_QUERY_LIMIT}");
        }
        if let (Some(start), Some(end)) = (&self.time_range.start, &self.time_range.end) {
            let start = chrono::DateTime::parse_from_rfc3339(start)
                .context("time_range.start must be RFC 3339")?;
            let end = chrono::DateTime::parse_from_rfc3339(end)
                .context("time_range.end must be RFC 3339")?;
            if start >= end {
                bail!("time_range.start must be earlier than time_range.end");
            }
        } else {
            for (name, value) in [
                ("time_range.start", self.time_range.start.as_ref()),
                ("time_range.end", self.time_range.end.as_ref()),
            ] {
                if let Some(value) = value {
                    chrono::DateTime::parse_from_rfc3339(value)
                        .with_context(|| format!("{name} must be RFC 3339"))?;
                }
            }
        }
        match &self.signal {
            QuerySignalV1::Logs { filter }
            | QuerySignalV1::Metrics { filter, .. }
            | QuerySignalV1::Traces { filter, .. } => {
                if let Some(filter) = filter {
                    filter.validate(0)?;
                }
            }
        }
        if let QuerySignalV1::Metrics {
            step_seconds,
            group_by,
            ..
        } = &self.signal
        {
            if step_seconds.is_some_and(|step| step == 0) {
                bail!("step_seconds must be greater than zero");
            }
            if group_by.len() > MAX_FILTER_ARGUMENTS {
                bail!("group_by has too many fields");
            }
            for field in group_by {
                validate_field(field)?;
            }
        }
        if let QuerySignalV1::Traces {
            min_duration_ms,
            max_duration_ms,
            attributes,
            ..
        } = &self.signal
        {
            if min_duration_ms
                .zip(*max_duration_ms)
                .is_some_and(|(min, max)| min > max)
            {
                bail!("min_duration_ms must not exceed max_duration_ms");
            }
            if attributes.len() > MAX_FILTER_ARGUMENTS {
                bail!("attributes has too many fields");
            }
        }
        Ok(())
    }
}

impl QueryExpressionV1 {
    fn validate(&self, depth: usize) -> Result<()> {
        if depth > MAX_FILTER_DEPTH {
            bail!("filter is nested too deeply");
        }
        match self {
            Self::And { args } | Self::Or { args } => {
                if args.is_empty() || args.len() > MAX_FILTER_ARGUMENTS {
                    bail!("and/or requires between 1 and {MAX_FILTER_ARGUMENTS} arguments");
                }
                for arg in args {
                    arg.validate(depth + 1)?;
                }
            }
            Self::Not { arg } => arg.validate(depth + 1)?,
            Self::Eq { field, .. } | Self::Exists { field } => validate_field(field)?,
            Self::In { field, values } => {
                validate_field(field)?;
                if values.is_empty() || values.len() > MAX_FILTER_ARGUMENTS {
                    bail!("in requires between 1 and {MAX_FILTER_ARGUMENTS} values");
                }
            }
            Self::Range {
                field,
                gt,
                gte,
                lt,
                lte,
            } => {
                validate_field(field)?;
                if [gt, gte, lt, lte].into_iter().all(Option::is_none) {
                    bail!("range requires at least one bound");
                }
                if gt.is_some() && gte.is_some() {
                    bail!("range cannot contain both gt and gte");
                }
                if lt.is_some() && lte.is_some() {
                    bail!("range cannot contain both lt and lte");
                }
            }
            Self::Text { field, value } => {
                validate_field(field)?;
                if value.trim().is_empty() {
                    bail!("text value must not be empty");
                }
            }
            Self::Regex { field, pattern } => {
                validate_field(field)?;
                if pattern.is_empty() || pattern.len() > MAX_REGEX_BYTES {
                    bail!("regex pattern must contain 1 to {MAX_REGEX_BYTES} bytes");
                }
                Regex::new(pattern).context("invalid regex pattern")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueryStatsV1 {
    pub elapsed_ms: u64,
    pub scanned: usize,
    pub returned: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueryResponseV1 {
    pub data: serde_json::Value,
    pub next_cursor: Option<String>,
    pub watermark: u64,
    pub partial: bool,
    pub warnings: Vec<String>,
    pub stats: QueryStatsV1,
    pub query_id: Option<String>,
}

impl QueryResponseV1 {
    pub fn complete(
        data: serde_json::Value,
        next_cursor: Option<String>,
        watermark: u64,
        scanned: usize,
        returned: usize,
        elapsed: Duration,
    ) -> Self {
        Self {
            data,
            next_cursor,
            watermark,
            partial: false,
            warnings: Vec::new(),
            stats: QueryStatsV1 {
                elapsed_ms: elapsed.as_millis().min(u128::from(u64::MAX)) as u64,
                scanned,
                returned,
            },
            query_id: None,
        }
    }
}

pub fn evaluate_filter(filter: &QueryExpressionV1, document: &serde_json::Value) -> Result<bool> {
    match filter {
        QueryExpressionV1::And { args } => {
            for arg in args {
                if !evaluate_filter(arg, document)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        QueryExpressionV1::Or { args } => {
            for arg in args {
                if evaluate_filter(arg, document)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        QueryExpressionV1::Not { arg } => Ok(!evaluate_filter(arg, document)?),
        QueryExpressionV1::Eq { field, value } => Ok(field_value(document, field) == Some(value)),
        QueryExpressionV1::In { field, values } => Ok(field_value(document, field)
            .is_some_and(|actual| values.iter().any(|value| value == actual))),
        QueryExpressionV1::Exists { field } => {
            Ok(field_value(document, field).is_some_and(|value| !value.is_null()))
        }
        QueryExpressionV1::Range {
            field,
            gt,
            gte,
            lt,
            lte,
        } => {
            let Some(actual) = field_value(document, field) else {
                return Ok(false);
            };
            Ok(gt
                .as_ref()
                .is_none_or(|bound| compare(actual, bound).is_some_and(|value| value > 0))
                && gte
                    .as_ref()
                    .is_none_or(|bound| compare(actual, bound).is_some_and(|value| value >= 0))
                && lt
                    .as_ref()
                    .is_none_or(|bound| compare(actual, bound).is_some_and(|value| value < 0))
                && lte
                    .as_ref()
                    .is_none_or(|bound| compare(actual, bound).is_some_and(|value| value <= 0)))
        }
        QueryExpressionV1::Text { field, value } => Ok(field_value(document, field)
            .and_then(value_text)
            .is_some_and(|actual| actual.to_lowercase().contains(&value.to_lowercase()))),
        QueryExpressionV1::Regex { field, pattern } => {
            let regex = Regex::new(pattern).context("invalid regex pattern")?;
            Ok(field_value(document, field)
                .and_then(value_text)
                .is_some_and(|actual| regex.is_match(&actual)))
        }
    }
}

fn default_query_limit() -> usize {
    DEFAULT_QUERY_LIMIT
}

fn validate_field(field: &str) -> Result<()> {
    if field.trim().is_empty() || field.len() > 256 {
        bail!("field must contain 1 to 256 bytes");
    }
    Ok(())
}

fn field_value<'a>(document: &'a serde_json::Value, field: &str) -> Option<&'a serde_json::Value> {
    if let Some(value) = document.get(field) {
        return Some(value);
    }
    let (first, remainder) = field.split_once('.')?;
    let child = document.get(first)?;
    if let Some(value) = child.get(remainder) {
        return Some(value);
    }
    remainder
        .split('.')
        .try_fold(child, |value, part| value.get(part))
}

fn value_text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn compare(left: &serde_json::Value, right: &serde_json::Value) -> Option<i8> {
    if let (Some(left), Some(right)) = (left.as_f64(), right.as_f64()) {
        return left.partial_cmp(&right).map(|ordering| match ordering {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        });
    }
    if let (Some(left), Some(right)) = (left.as_str(), right.as_str()) {
        return Some(match left.cmp(right) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        });
    }
    None
}
