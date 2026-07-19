// HANDWRITE-BEGIN gap="sift-bounded-event-batch" tracker="1658" reason="Decode bounded event batches and report ordered accepted, duplicate, or rejected per-item outcomes."
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::{IncomingEvent, OperationalEventV2};

use super::gcp;

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct EventWriteRequest {
    pub events: Vec<Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BatchOutcome {
    Accepted,
    Duplicate,
    Rejected,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct IngestErrorDetail {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct BatchItemResult {
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    pub outcome: BatchOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<IngestErrorDetail>,
}

impl BatchItemResult {
    pub fn accepted(index: usize, event_id: String, cursor: u64, duplicate: bool) -> Self {
        Self {
            index,
            event_id: Some(event_id),
            outcome: if duplicate {
                BatchOutcome::Duplicate
            } else {
                BatchOutcome::Accepted
            },
            cursor: Some(cursor),
            error: None,
        }
    }

    pub fn rejected(
        index: usize,
        event_id: Option<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self {
            index,
            event_id,
            outcome: BatchOutcome::Rejected,
            cursor: None,
            error: Some(IngestErrorDetail {
                code: code.into(),
                message: message.into(),
                retryable,
            }),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, ToSchema)]
pub struct EventWriteResponse {
    pub results: Vec<BatchItemResult>,
    pub accepted: usize,
    pub duplicates: usize,
    pub rejected: usize,
}

impl EventWriteResponse {
    pub fn from_results(results: Vec<BatchItemResult>) -> Self {
        let mut response = Self {
            results,
            ..Self::default()
        };
        for result in &response.results {
            match result.outcome {
                BatchOutcome::Accepted => response.accepted += 1,
                BatchOutcome::Duplicate => response.duplicates += 1,
                BatchOutcome::Rejected => response.rejected += 1,
            }
        }
        response
    }
}

pub fn event_id_hint(value: &Value) -> Option<String> {
    value
        .get("event_id")
        .or_else(|| value.get("eventId"))
        .or_else(|| value.get("insertId"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

pub fn decode_item(value: Value, project_hint: &str) -> anyhow::Result<OperationalEventV2> {
    if gcp::looks_like_structured_log(&value) {
        return gcp::normalize_structured_log(value, project_hint);
    }
    let incoming: IncomingEvent = serde_json::from_value(value)?;
    Ok(incoming.into_inner())
}
// HANDWRITE-END
