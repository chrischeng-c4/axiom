---
id: projects-sdd-src-issues-labels-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# Standardized projects/agentic-workflow/src/issues/labels.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/issues/labels.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DecodedCrrr` | projects/agentic-workflow/src/issues/labels.rs | struct | pub | 113 |  |
| `FLAGGED_PREFIX` | projects/agentic-workflow/src/issues/labels.rs | constant | pub | 39 |  |
| `MANAGED_PREFIXES` | projects/agentic-workflow/src/issues/labels.rs | constant | pub | 51 |  |
| `PHASE_PREFIX` | projects/agentic-workflow/src/issues/labels.rs | constant | pub | 34 |  |
| `RETRY_PREFIX` | projects/agentic-workflow/src/issues/labels.rs | constant | pub | 36 |  |
| `REVIEW_PREFIX` | projects/agentic-workflow/src/issues/labels.rs | constant | pub | 35 |  |
| `SHIP_COMMIT_PREFIX` | projects/agentic-workflow/src/issues/labels.rs | constant | pub | 38 |  |
| `SHIP_PREFIX` | projects/agentic-workflow/src/issues/labels.rs | constant | pub | 37 |  |
| `SLUG_PREFIX` | projects/agentic-workflow/src/issues/labels.rs | constant | pub | 42 |  |
| `WORKFLOW_LOCK_LABEL` | projects/agentic-workflow/src/issues/labels.rs | constant | pub | 40 |  |
| `WORKFLOW_LOCK_OWNER_PREFIX` | projects/agentic-workflow/src/issues/labels.rs | constant | pub | 41 |  |
| `decode_labels` | projects/agentic-workflow/src/issues/labels.rs | function | pub | 127 | decode_labels(labels: &[String]) -> DecodedCrrr |
| `diff_labels` | projects/agentic-workflow/src/issues/labels.rs | function | pub | 171 | diff_labels(current: &[String], desired: &[String]) -> (Vec<String>, Vec<String>) |
| `encode_labels` | projects/agentic-workflow/src/issues/labels.rs | function | pub | 76 | encode_labels(issue: &Issue) -> Vec<String> |
| `is_managed` | projects/agentic-workflow/src/issues/labels.rs | function | pub | 63 | is_managed(label: &str) -> bool |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/issues/labels.rs -->
```rust
//! Label-encoded CRRR state for tracker backends (GitHub, GitLab).
//!
//! Mapping rule (per user directive 2026-05-04):
//!
//! 1. Use **native attributes** where they exist on the tracker:
//!    - `title` → issue title
//!    - `state` (open/closed) → issue state
//!    - `body` → issue body markdown
//!    - `labels` → user labels (`crate:*`, `priority:*`, `type:*`)
//! 2. Otherwise, **encode as labels** with the prefixes below.
//!
//! Prefix scheme (one label per CRRR slot, except `flagged:` which is multi):
//!
//! | Prefix          | Field                       | Cardinality |
//! |-----------------|-----------------------------|-------------|
//! | `phase:`        | `Issue.phase`               | 0..=1       |
//! | `review:`       | `Issue.review_count`        | 0..=1       |
//! | `retry:`        | `Issue.fill_retry_count`    | 0..=1       |
//! | `ship:`         | `Issue.ship_status`         | 0..=1       |
//! | `ship-commit:`  | `Issue.ship_commit` (sha7)  | 0..=1       |
//! | `flagged:`      | `Issue.flagged_sections[i]` | 0..=N       |
//! | `slug:`         | legacy/display alias        | unmanaged  |
//!
//! Remote tracker numbers/iids are the canonical issue identity. `slug:` is
//! retained only as a legacy alias for older callers and labels.

use crate::issues::types::{Issue, IssueSection, ShipStatus};
use std::collections::HashSet;

pub const PHASE_PREFIX: &str = "phase:";
pub const REVIEW_PREFIX: &str = "review:";
pub const RETRY_PREFIX: &str = "retry:";
pub const SHIP_PREFIX: &str = "ship:";
pub const SHIP_COMMIT_PREFIX: &str = "ship-commit:";
pub const FLAGGED_PREFIX: &str = "flagged:";
pub const SLUG_PREFIX: &str = "slug:";

/// All managed prefixes — used by the diff helper to know which existing
/// labels are score-controlled (and thus may be removed) vs user-controlled
/// (and must be left alone).
///
/// `slug:` is deliberately excluded: tracker numbers/iids are canonical, and
/// old `slug:*` labels remain valid legacy aliases until an operator removes
/// them.
pub const MANAGED_PREFIXES: &[&str] = &[
    PHASE_PREFIX,
    REVIEW_PREFIX,
    RETRY_PREFIX,
    SHIP_PREFIX,
    SHIP_COMMIT_PREFIX,
    FLAGGED_PREFIX,
];

/// True if `label` carries score-managed CRRR state (one of `MANAGED_PREFIXES`).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/labels.md#source
pub fn is_managed(label: &str) -> bool {
    MANAGED_PREFIXES.iter().any(|p| label.starts_with(p))
}

/// Build the full set of labels for `issue` — user labels (preserved) plus
/// CRRR-state labels derived from `phase / review_count / flagged_sections /
/// fill_retry_count / ship_status / ship_commit. Legacy `slug:*` labels are
/// preserved if present in `issue.labels`, but this encoder no longer emits a
/// new one from `issue.slug`.
///
/// Stable ordering: user labels first (in input order), then CRRR labels in
/// the prefix order declared by `MANAGED_PREFIXES`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/labels.md#source
pub fn encode_labels(issue: &Issue) -> Vec<String> {
    let mut out: Vec<String> = issue
        .labels
        .iter()
        .filter(|l| !is_managed(l))
        .cloned()
        .collect();

    if let Some(phase) = &issue.phase {
        out.push(format!("{}{}", PHASE_PREFIX, phase));
    }
    if let Some(n) = issue.review_count {
        out.push(format!("{}{}", REVIEW_PREFIX, n));
    }
    if let Some(n) = issue.fill_retry_count {
        out.push(format!("{}{}", RETRY_PREFIX, n));
    }
    if let Some(s) = issue.ship_status {
        out.push(format!("{}{}", SHIP_PREFIX, ship_status_str(s)));
    }
    if let Some(sha) = &issue.ship_commit {
        // Trim to 7-char short SHA to stay within GitHub's 50-char label cap.
        let short = if sha.len() > 7 { &sha[..7] } else { sha };
        out.push(format!("{}{}", SHIP_COMMIT_PREFIX, short));
    }
    if let Some(flagged) = &issue.flagged_sections {
        for sec in flagged {
            out.push(format!("{}{}", FLAGGED_PREFIX, sec.as_str()));
        }
    }
    out
}

/// Decoded CRRR state extracted from a label set. Used by `parse_gh_issue` /
/// `parse_glab_issue` to rebuild an `Issue` from a tracker payload.
#[derive(Debug, Default, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/labels.md#source
pub struct DecodedCrrr {
    pub phase: Option<String>,
    pub review_count: Option<u8>,
    pub fill_retry_count: Option<u8>,
    pub ship_status: Option<ShipStatus>,
    pub ship_commit: Option<String>,
    pub flagged_sections: Option<Vec<IssueSection>>,
    pub slug: Option<String>,
}

/// Decode CRRR state out of a label set. Unknown / malformed managed labels
/// are silently ignored (a future-version label scheme should not break older
/// readers).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/labels.md#source
pub fn decode_labels(labels: &[String]) -> DecodedCrrr {
    let mut out = DecodedCrrr::default();
    let mut flagged: Vec<IssueSection> = Vec::new();

    for label in labels {
        if let Some(rest) = label.strip_prefix(PHASE_PREFIX) {
            out.phase = Some(rest.to_string());
        } else if let Some(rest) = label.strip_prefix(REVIEW_PREFIX) {
            if let Ok(n) = rest.parse::<u8>() {
                out.review_count = Some(n);
            }
        } else if let Some(rest) = label.strip_prefix(RETRY_PREFIX) {
            if let Ok(n) = rest.parse::<u8>() {
                out.fill_retry_count = Some(n);
            }
        } else if let Some(rest) = label.strip_prefix(SHIP_PREFIX) {
            if let Some(s) = parse_ship_status(rest) {
                out.ship_status = Some(s);
            }
        } else if let Some(rest) = label.strip_prefix(SHIP_COMMIT_PREFIX) {
            out.ship_commit = Some(rest.to_string());
        } else if let Some(rest) = label.strip_prefix(FLAGGED_PREFIX) {
            if let Some(sec) = IssueSection::parse(rest) {
                flagged.push(sec);
            }
        } else if let Some(rest) = label.strip_prefix(SLUG_PREFIX) {
            out.slug = Some(rest.to_string());
        }
    }

    if !flagged.is_empty() {
        out.flagged_sections = Some(flagged);
    }
    out
}

/// Compute `(to_add, to_remove)` diff so that applying it to `current` yields
/// the union of (user labels in `desired`) ∪ (managed labels in `desired`).
///
/// Critically, this leaves user labels untouched if they are present on the
/// remote but absent in `desired` — only **managed** labels can be removed by
/// this diff. That way we don't fight humans who add their own labels via the
/// GitHub UI between two `score` writes.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/labels.md#source
pub fn diff_labels(current: &[String], desired: &[String]) -> (Vec<String>, Vec<String>) {
    let current_set: HashSet<&str> = current.iter().map(String::as_str).collect();
    let desired_set: HashSet<&str> = desired.iter().map(String::as_str).collect();

    let to_add: Vec<String> = desired
        .iter()
        .filter(|l| !current_set.contains(l.as_str()))
        .cloned()
        .collect();

    // Only remove managed labels that are no longer desired.
    let to_remove: Vec<String> = current
        .iter()
        .filter(|l| is_managed(l) && !desired_set.contains(l.as_str()))
        .cloned()
        .collect();

    (to_add, to_remove)
}

fn ship_status_str(s: ShipStatus) -> &'static str {
    match s {
        ShipStatus::NotStarted => "not_started",
        ShipStatus::Step1Shipped => "step1_shipped",
        ShipStatus::LoopClosed => "loop_closed",
        ShipStatus::Rejected => "rejected",
    }
}

fn parse_ship_status(s: &str) -> Option<ShipStatus> {
    match s {
        "not_started" => Some(ShipStatus::NotStarted),
        "step1_shipped" => Some(ShipStatus::Step1Shipped),
        "loop_closed" => Some(ShipStatus::LoopClosed),
        "rejected" => Some(ShipStatus::Rejected),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issues::types::{IssueState, IssueType};

    fn base_issue() -> Issue {
        Issue {
            issue_type: IssueType::Bug,
            title: "t".into(),
            state: IssueState::Open,
            id: None,
            github_id: None,
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec!["crate:sdd".into(), "priority:p2".into()],
            created_at: None,
            updated_at: None,
            slug: "demo".into(),
            body: String::new(),
            related: vec![],
            implements: vec![],
            phase: None,
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: vec![],
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        }
    }

    #[test]
    fn encode_includes_user_and_managed_labels() {
        let mut i = base_issue();
        i.phase = Some("td_reviewed".into());
        i.review_count = Some(1);
        i.flagged_sections = Some(vec![IssueSection::Requirements, IssueSection::Scope]);
        let labels = encode_labels(&i);
        assert!(labels.contains(&"crate:sdd".to_string()));
        assert!(labels.contains(&"priority:p2".to_string()));
        assert!(labels.contains(&"phase:td_reviewed".to_string()));
        assert!(labels.contains(&"review:1".to_string()));
        assert!(labels.contains(&"flagged:requirements".to_string()));
        assert!(labels.contains(&"flagged:scope".to_string()));
        assert!(!labels.contains(&"slug:demo".to_string()));
    }

    #[test]
    fn encode_strips_stale_managed_from_input() {
        // If issue.labels carries a stale phase: label, encode_labels should
        // drop it and emit only the value derived from issue.phase.
        let mut i = base_issue();
        i.labels.push("phase:td_inited".into());
        i.phase = Some("td_reviewed".into());
        let labels = encode_labels(&i);
        assert!(!labels.contains(&"phase:td_inited".to_string()));
        assert!(labels.contains(&"phase:td_reviewed".to_string()));
    }

    #[test]
    fn decode_round_trips_phase_and_review() {
        let labels = vec![
            "crate:sdd".into(),
            "phase:td_reviewed".into(),
            "review:2".into(),
            "retry:1".into(),
            "flagged:requirements".into(),
            "slug:demo".into(),
        ];
        let d = decode_labels(&labels);
        assert_eq!(d.phase.as_deref(), Some("td_reviewed"));
        assert_eq!(d.review_count, Some(2));
        assert_eq!(d.fill_retry_count, Some(1));
        assert_eq!(d.slug.as_deref(), Some("demo"));
        assert_eq!(d.flagged_sections, Some(vec![IssueSection::Requirements]));
    }

    #[test]
    fn diff_only_removes_managed_labels() {
        let current = vec![
            "crate:sdd".into(),         // user — must NOT be removed
            "phase:td_inited".into(),   // managed, stale — should be removed
            "slug:legacy-alias".into(), // legacy alias — must NOT be removed
            "manual-label".into(),      // user-added between writes — keep
        ];
        let desired = vec!["crate:sdd".into(), "phase:td_reviewed".into()];
        let (add, remove) = diff_labels(&current, &desired);
        assert_eq!(add, vec!["phase:td_reviewed".to_string()]);
        assert_eq!(remove, vec!["phase:td_inited".to_string()]);
    }

    #[test]
    fn ship_commit_truncated_to_short_sha() {
        let mut i = base_issue();
        i.ship_commit = Some("abcdef0123456789abcdef".into());
        let labels = encode_labels(&i);
        assert!(labels.contains(&"ship-commit:abcdef0".to_string()));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/labels.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the full issue label encoding module.
```
