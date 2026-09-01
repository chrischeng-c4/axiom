use std::collections::BTreeMap;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::projection::{LogRecordV1, MetricSeriesResultV1, TraceResultV1};

use super::query::{MAX_QUERY_LIMIT, QUERY_AST_VERSION};
use super::{QueryExpressionV1, TimeRangeV1};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LogTailRequestV1 {
    pub version: u32,
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<QueryExpressionV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_cursor: Option<String>,
    #[serde(default = "default_tail_wait_ms")]
    pub wait_ms: u64,
    #[serde(default = "default_query_limit")]
    pub limit: usize,
}

impl LogTailRequestV1 {
    pub fn validate(&self) -> Result<()> {
        if self.version != QUERY_AST_VERSION {
            bail!(
                "unsupported query version {}; expected {QUERY_AST_VERSION}",
                self.version
            );
        }
        if self.project.trim().is_empty() {
            bail!("project must not be empty");
        }
        if self.limit == 0 || self.limit > MAX_QUERY_LIMIT {
            bail!("limit must be between 1 and {MAX_QUERY_LIMIT}");
        }
        if self.wait_ms > 30_000 {
            bail!("wait_ms must not exceed 30000");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorrelationRequestV1 {
    pub version: u32,
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default)]
    pub time_range: TimeRangeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, serde_json::Value>,
    #[serde(default = "default_query_limit")]
    pub limit: usize,
}

impl CorrelationRequestV1 {
    pub fn validate(&self) -> Result<()> {
        if self.version != QUERY_AST_VERSION {
            bail!(
                "unsupported query version {}; expected {QUERY_AST_VERSION}",
                self.version
            );
        }
        if self.project.trim().is_empty() {
            bail!("project must not be empty");
        }
        if self.limit == 0 || self.limit > MAX_QUERY_LIMIT {
            bail!("limit must be between 1 and {MAX_QUERY_LIMIT}");
        }
        if self.trace_id.as_deref().is_none_or(str::is_empty)
            && self.span_id.as_deref().is_none_or(str::is_empty)
            && self.service.as_deref().is_none_or(str::is_empty)
            && self.attributes.is_empty()
        {
            bail!("correlation requires trace_id, span_id, service, or attributes");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CorrelationResponseV1 {
    pub logs: Vec<LogRecordV1>,
    pub metrics: Vec<MetricSeriesResultV1>,
    pub traces: Vec<TraceResultV1>,
    pub watermark: u64,
    pub partial: bool,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceQueryV1 {
    pub project: String,
    #[serde(default)]
    pub environment: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ServiceSummaryV1 {
    pub name: String,
    pub environments: Vec<String>,
    pub versions: Vec<String>,
    pub signals: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ServiceListResponseV1 {
    pub services: Vec<ServiceSummaryV1>,
    pub watermark: u64,
}

fn default_tail_wait_ms() -> u64 {
    2_000
}

fn default_query_limit() -> usize {
    100
}
