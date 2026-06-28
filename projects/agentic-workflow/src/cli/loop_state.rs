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

/// Apply an EC verifier result to the loop-state block in a WI body — the
/// producer side of the loop: parse the existing block (or start a fresh one),
/// `record_verification` (append the iteration + decide `next_action` from the
/// verifier), and re-emit the block in place. Pure; the caller persists the
/// returned body. This is what the EC verify step calls so `aw run`'s loop
/// engine has an up-to-date block to read (#188 E1/E4).
pub fn apply_verification(
    body: &str,
    result: LastResult,
    summary: Option<String>,
) -> Result<String> {
    let mut state = parse_loop_state(body).unwrap_or_default();
    if state.version == 0 {
        state.version = 1;
    }
    state.record_verification(result, summary);
    upsert_loop_state(body, &state)
}

/// The loop's decision, given the latest verification (EC) result: the status
/// the loop is now in, and the next command to run. Green = converged (merge);
/// Red = keep iterating (adapt and re-gen); Blocked = HITL (no auto next).
/// This is the loop-engineering "decide" step — it reads the verifier, not a
/// reviewer.
pub fn decide_next_action(last: &LastResult) -> (LoopStatus, Option<&'static str>) {
    match last {
        LastResult::Green => (LoopStatus::Converged, Some("aw td merge")),
        LastResult::Red { .. } => (LoopStatus::Iterating, Some("aw td gen")),
        LastResult::Blocked { .. } => (LoopStatus::Blocked, None),
        LastResult::None => (LoopStatus::Iterating, None),
    }
}

impl LoopState {
    /// Record a verification (EC) outcome: append it to the running log, set it
    /// as `last_result`, and derive `status` + `next_action` via
    /// [`decide_next_action`]. This is the loop's observe -> decide step,
    /// driven by the EC verifier rather than a review.
    pub fn record_verification(&mut self, result: LastResult, summary: Option<String>) {
        let n = self.iterations.len() as u32 + 1;
        let outcome = match &result {
            LastResult::Green => "green".to_string(),
            LastResult::Red { dimension, .. } => format!("red:{dimension}"),
            LastResult::Blocked { reason } => format!("blocked:{reason}"),
            LastResult::None => "none".to_string(),
        };
        self.iterations.push(Iteration {
            n,
            action: "ec".to_string(),
            outcome,
            summary,
        });
        let (status, next) = decide_next_action(&result);
        self.status = status;
        self.next_action = next.map(str::to_string);
        self.last_result = result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // @spec epic #188 E4: ec verifier drives the loop decision.
    #[test]
    fn decide_green_converges_to_merge() {
        assert_eq!(
            decide_next_action(&LastResult::Green),
            (LoopStatus::Converged, Some("aw td merge"))
        );
    }

    #[test]
    fn decide_red_iterates_to_gen() {
        let red = LastResult::Red {
            dimension: "behavior".into(),
            why: "t failed".into(),
        };
        assert_eq!(
            decide_next_action(&red),
            (LoopStatus::Iterating, Some("aw td gen"))
        );
    }

    #[test]
    fn decide_blocked_is_hitl() {
        let blocked = LastResult::Blocked {
            reason: "ec undefined".into(),
        };
        assert_eq!(decide_next_action(&blocked), (LoopStatus::Blocked, None));
    }

    #[test]
    fn record_verification_appends_and_decides() {
        let mut s = LoopState {
            version: 1,
            issue_id: "1".into(),
            status: LoopStatus::Iterating,
            ..Default::default()
        };
        s.record_verification(
            LastResult::Red {
                dimension: "behavior".into(),
                why: "t failed".into(),
            },
            Some("round 1".into()),
        );
        assert_eq!(s.iterations.len(), 1);
        assert_eq!(s.status, LoopStatus::Iterating);
        assert_eq!(s.next_action.as_deref(), Some("aw td gen"));

        s.record_verification(LastResult::Green, None);
        assert_eq!(s.iterations.len(), 2);
        assert_eq!(s.status, LoopStatus::Converged);
        assert_eq!(s.next_action.as_deref(), Some("aw td merge"));
        assert_eq!(s.last_result, LastResult::Green);
    }

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
        assert_eq!(
            parse_loop_state(&after2).unwrap().status,
            LoopStatus::Converged
        );
    }

    #[test]
    fn apply_verification_starts_then_advances_the_block_in_a_body() {
        // Fresh body (no block) + a Red verdict -> a block appears, iterating,
        // and the original body text is preserved.
        let body = "# WI 188\n\nsome description\n";
        let out = apply_verification(
            body,
            LastResult::Red {
                dimension: "behavior".into(),
                why: "t failed".into(),
            },
            Some("round 1".into()),
        )
        .unwrap();
        assert!(out.contains(LOOP_START));
        assert!(out.contains("some description"));
        let s = parse_loop_state(&out).unwrap();
        assert_eq!(s.version, 1);
        assert_eq!(s.iterations.len(), 1);
        assert_eq!(s.status, LoopStatus::Iterating);
        assert_eq!(s.next_action.as_deref(), Some("aw td gen"));

        // Re-apply a Green verdict on the same body -> converged, 2nd iteration,
        // block replaced in place (not duplicated).
        let out2 = apply_verification(&out, LastResult::Green, None).unwrap();
        let s2 = parse_loop_state(&out2).unwrap();
        assert_eq!(s2.iterations.len(), 2);
        assert_eq!(s2.status, LoopStatus::Converged);
        assert_eq!(s2.next_action.as_deref(), Some("aw td merge"));
        assert_eq!(out2.matches(LOOP_START).count(), 1);
    }
}
// HANDWRITE-END aw-workitem-loop-state
