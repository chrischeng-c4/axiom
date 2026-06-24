// SPEC-MANAGED: projects/agentic-workflow/tech-design/logic/workitem-loop-state-model-additive-foundation.md
// HANDWRITE-BEGIN aw-workitem-loop-state
//! WorkItem loop state — the durable state of the loop-engineering loop the
//! workflow runs over a WorkItem.
//!
//! Carried in the WI body as an `<!-- aw:loop-state ... -->` block, mirroring
//! the `<!-- score:workflow-state -->` projection but modelling the loop's
//! convergence (goal / verifier / iterations / last_result / next_action /
//! status / tried) rather than a CRRR phase. Additive and non-breaking: this
//! block is independent of, and does not disturb, the workflow-state block.
//!
//! @spec projects/agentic-workflow/tech-design/logic/workitem-loop-state-model-additive-foundation.md

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const LOOP_START: &str = "<!-- aw:loop-state";
const LOOP_END: &str = "-->";

/// The result of the most recent verification (EC run) in the loop.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LastResult {
    /// No verification has run yet.
    #[default]
    None,
    /// EC passed — the goal is achieved.
    Green,
    /// EC failed on a dimension; carries which and why so the next iteration
    /// can adapt rather than repeat.
    Red { dimension: String, why: String },
    /// The loop is blocked (e.g. EC is not yet defined, or a hard blocker).
    Blocked { reason: String },
}

/// Where the loop is on its way to convergence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoopStatus {
    /// Still iterating act -> verify.
    #[default]
    Iterating,
    /// EC green — goal achieved.
    Converged,
    /// Awaiting a human decision (HITL).
    Blocked,
    /// Gave up / hard failure.
    Failed,
}

/// One pass of the loop: an action and its observed outcome.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Iteration {
    /// 1-based iteration index.
    pub n: u32,
    /// What was done this pass — e.g. `td`, `ec`, `caps-adjust`.
    pub action: String,
    /// Short outcome — e.g. `implemented`, `green`, `red`.
    pub outcome: String,
    /// Optional compact summary for the running log.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// The loop's durable state, persisted in the WI body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LoopState {
    pub version: u8,
    pub issue_id: String,
    /// The capability gap this loop is driving to closure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    /// The EC verifier that decides "done".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifier: Option<String>,
    /// Running log of every pass and its outcome.
    #[serde(default)]
    pub iterations: Vec<Iteration>,
    /// The most recent verification result.
    #[serde(default)]
    pub last_result: LastResult,
    /// The command the loop should run next.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
    /// Convergence status.
    #[serde(default)]
    pub status: LoopStatus,
    /// Fingerprints of approaches already tried-and-failed, so the loop does
    /// not repeat the same failed approach.
    #[serde(default)]
    pub tried: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Parse the `<!-- aw:loop-state -->` block out of a WI body. Returns `None`
/// when the block is absent — an absent block is not an error.
pub fn parse_loop_state(body: &str) -> Option<LoopState> {
    let start = body.find(LOOP_START)?;
    let rest = &body[start + LOOP_START.len()..];
    let end = rest.find(LOOP_END)?;
    let yaml = rest[..end].trim();
    serde_yaml::from_str(yaml).ok()
}

/// Insert or replace the `<!-- aw:loop-state -->` block in a WI body. When the
/// block is absent it is appended; when present it is replaced in place. The
/// `<!-- score:workflow-state -->` block is never touched.
pub fn upsert_loop_state(body: &str, state: &LoopState) -> Result<String> {
    let yaml = serde_yaml::to_string(state).context("serializing loop state")?;
    let block = format!("{LOOP_START}\n{}\n{}\n", yaml.trim_end(), LOOP_END);
    if let Some(start) = body.find(LOOP_START) {
        let rest = &body[start + LOOP_START.len()..];
        if let Some(end_rel) = rest.find(LOOP_END) {
            let end = start + LOOP_START.len() + end_rel + LOOP_END.len();
            let mut out = String::new();
            let head = body[..start].trim_end();
            out.push_str(head);
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(&block);
            out.push_str(body[end..].trim_start_matches('\n'));
            return Ok(out);
        }
    }
    let mut out = body.trim_end().to_string();
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str(&block);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // @spec workitem-loop-state-model-additive-foundation.md R1
    #[test]
    fn loop_state_round_trips() {
        for last in [
            LastResult::None,
            LastResult::Green,
            LastResult::Red {
                dimension: "behavior".into(),
                why: "test_x failed".into(),
            },
            LastResult::Blocked {
                reason: "ec not yet defined".into(),
            },
        ] {
            let state = LoopState {
                version: 1,
                issue_id: "189".into(),
                goal: Some("workitem-loop-state-model".into()),
                verifier: Some("ec:behavior".into()),
                iterations: vec![
                    Iteration {
                        n: 1,
                        action: "td".into(),
                        outcome: "implemented".into(),
                        summary: Some("first cut".into()),
                    },
                    Iteration {
                        n: 2,
                        action: "ec".into(),
                        outcome: "red".into(),
                        summary: None,
                    },
                ],
                last_result: last.clone(),
                next_action: Some("aw td 189".into()),
                status: LoopStatus::Iterating,
                tried: vec!["approach-a".into()],
                updated_at: Some("2026-06-24T00:00:00Z".into()),
            };
            let body = upsert_loop_state("# WI body\n", &state).unwrap();
            let parsed = parse_loop_state(&body).expect("must parse the block back");
            assert_eq!(parsed, state, "round-trip must be lossless for {last:?}");
        }
    }

    // @spec workitem-loop-state-model-additive-foundation.md R2
    #[test]
    fn loop_state_absent_block_is_none() {
        assert!(parse_loop_state("# just a body, no block").is_none());
        // A workflow-state block present, but no loop-state block -> still None.
        let body =
            "# body\n\n<!-- score:workflow-state\nversion: 1\nissue_id: '1'\nlocked: false\n-->\n";
        assert!(parse_loop_state(body).is_none());
    }

    // @spec workitem-loop-state-model-additive-foundation.md R3
    #[test]
    fn loop_state_upsert_replaces_in_place() {
        let wf =
            "# body\n\n<!-- score:workflow-state\nversion: 1\nissue_id: '1'\nlocked: false\n-->\n";
        let s1 = LoopState {
            version: 1,
            issue_id: "1".into(),
            status: LoopStatus::Iterating,
            ..Default::default()
        };
        let after1 = upsert_loop_state(wf, &s1).unwrap();
        assert!(
            after1.contains("score:workflow-state"),
            "append must not disturb the workflow-state block"
        );
        assert_eq!(after1.matches(LOOP_START).count(), 1);

        let s2 = LoopState {
            version: 1,
            issue_id: "1".into(),
            status: LoopStatus::Converged,
            ..Default::default()
        };
        let after2 = upsert_loop_state(&after1, &s2).unwrap();
        assert_eq!(
            after2.matches(LOOP_START).count(),
            1,
            "second upsert must replace in place, not duplicate"
        );
        assert!(
            after2.contains("score:workflow-state"),
            "replace must not disturb the workflow-state block"
        );
        assert_eq!(parse_loop_state(&after2).unwrap().status, LoopStatus::Converged);
    }
}
// HANDWRITE-END aw-workitem-loop-state
