//! loom relay-proxy (#432) — the schema-guard gateway between worker and relay.
//!
//! Its only job is to **guarantee the message format/schema** so any hand-written,
//! any-language worker conforms or gets a clear error — no per-language SDK. It is
//! thin: it validates frames, **owns the keep key schema** (the worker never builds
//! a key), and forwards. The DAG fold stays in the controller.
//!
//! Worker ⟷ proxy is a bidi stream: the proxy pushes [`TaskEnvelope`] frames down,
//! the worker sends [`UpFrame`]s back up. This module is the transport-agnostic
//! core (frames + envelope); the bidi-h2 endpoint + relay-consume wiring layer on
//! top (a later atom), so the schema-guarantee heart is verifiable in isolation.

use serde::{Deserialize, Serialize};

use crate::scheduler::TaskMessage;

/// Frames the worker sends UP the bidi stream. Strict deserialization *is* the
/// format guarantee — a malformed frame fails to parse and is rejected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UpFrame {
    /// First frame: join a work `group` with a `prefetch` (credit) window.
    Subscribe { group: String, prefetch: u32 },
    /// Task finished OK. Small results come back inline; large results are already
    /// in keep (the worker PUT them to the envelope's `result_put_url`).
    Done {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result_inline: Option<Vec<u8>>,
    },
    /// Task could not be handled — redeliver to another consumer.
    Nack { id: String },
}

impl UpFrame {
    /// Parse + validate an up-frame. Errors on any malformed/unknown frame.
    pub fn parse(bytes: &[u8]) -> anyhow::Result<UpFrame> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

/// Where a task's input lives (claim-check): inline for small payloads, a keep
/// URL for large, or empty.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InputSource {
    Inline { bytes: Vec<u8> },
    KeepUrl { url: String },
    Empty,
}

/// Frame the proxy pushes DOWN: a fully self-describing task. The worker never
/// constructs a keep key — the proxy owns the schema — so it cannot get it wrong.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskEnvelope {
    /// Ack/dedup id = `run:node:attempt` (matches `UpFrame::Done.id`).
    pub id: String,
    pub run_id: String,
    pub node_id: String,
    pub attempt: u32,
    pub task_name: String,
    pub args: serde_json::Value,
    pub input: InputSource,
    /// Pre-resolved keep URL the worker PUTs a (large) result to.
    pub result_put_url: String,
    /// Scoped keep token issued per task (keep tokens #434/#445).
    pub token: String,
}

/// Build the self-describing envelope for a leased task. **Owns the keep key
/// schema**: input/result keys are constructed here, so the worker only does dumb
/// GET/PUT against given URLs.
pub fn build_envelope(task: &TaskMessage, keep_base: &str, token: String) -> TaskEnvelope {
    let result_key = format!("{}:{}:result", task.run_id, task.node_id);
    let input = if let Some(bytes) = &task.input_inline {
        InputSource::Inline { bytes: bytes.clone() }
    } else if let Some(first) = task.input_refs.first() {
        InputSource::KeepUrl { url: format!("{keep_base}/v1/inputs/{}", first.0) }
    } else {
        InputSource::Empty
    };
    TaskEnvelope {
        id: task.message_id(),
        run_id: task.run_id.clone(),
        node_id: task.node_id.clone(),
        attempt: task.attempt,
        task_name: task.task_name.clone(),
        args: task.args.clone(),
        input,
        result_put_url: format!("{keep_base}/v1/results/{result_key}"),
        token,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::KeepRef;
    use crate::runner::RunnerClass;

    fn task(input_refs: Vec<KeepRef>, inline: Option<Vec<u8>>) -> TaskMessage {
        TaskMessage {
            run_id: "run1".into(),
            node_id: "nodeA".into(),
            attempt: 2,
            task_name: "crunch".into(),
            args: serde_json::json!({"k": 1}),
            input_refs,
            input_inline: inline,
            runner: RunnerClass::Resident,
        }
    }

    #[test]
    fn envelope_owns_keys_and_resolves_input() {
        // input ref → keep GET url; result → keep PUT url; id = run:node:attempt
        let e = build_envelope(&task(vec![KeepRef("in:7".into())], None), "http://keep", "tok".into());
        assert_eq!(e.id, "run1:nodeA:2");
        assert_eq!(e.input, InputSource::KeepUrl { url: "http://keep/v1/inputs/in:7".into() });
        assert_eq!(e.result_put_url, "http://keep/v1/results/run1:nodeA:result");
        assert_eq!(e.token, "tok");

        // inline input wins over refs
        let e2 = build_envelope(&task(vec![], Some(b"hi".to_vec())), "http://keep", "t".into());
        assert_eq!(e2.input, InputSource::Inline { bytes: b"hi".to_vec() });

        // no input → empty
        let e3 = build_envelope(&task(vec![], None), "http://keep", "t".into());
        assert_eq!(e3.input, InputSource::Empty);
    }

    #[test]
    fn up_frames_round_trip_and_reject_malformed() {
        for f in [
            UpFrame::Subscribe { group: "w".into(), prefetch: 4 },
            UpFrame::Done { id: "run1:nodeA:2".into(), result_inline: Some(b"r".to_vec()) },
            UpFrame::Done { id: "x".into(), result_inline: None },
            UpFrame::Nack { id: "x".into() },
        ] {
            let bytes = serde_json::to_vec(&f).unwrap();
            assert_eq!(UpFrame::parse(&bytes).unwrap(), f);
        }
        // malformed / unknown frames are rejected (the format guarantee)
        assert!(UpFrame::parse(b"{}").is_err());
        assert!(UpFrame::parse(br#"{"type":"bogus"}"#).is_err());
        assert!(UpFrame::parse(b"not json").is_err());
    }
}
