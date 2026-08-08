//! Durable causal read model for Change work items (#3347).
//!
//! The tracker issue remains the WI source of truth.  This module stores only
//! the additive, workspace-local lifecycle carrier used by `aw wi show`.

use crate::issues::{Issue, IssueState};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const SCHEMA: &str = "aw.change-lifecycle.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    Wi,
    Ec,
    Td,
    Cb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OwnerVocabulary {
    Wi,
    Ec,
    Td,
    Cb,
    Migration,
}

impl OwnerVocabulary {
    fn as_str(self) -> &'static str {
        match self {
            Self::Wi => "wi",
            Self::Ec => "ec",
            Self::Td => "td",
            Self::Cb => "cb",
            Self::Migration => "migration",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausalParent {
    #[serde(rename = "id")]
    pub revision_id: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRevision {
    pub id: String,
    pub kind: ArtifactKind,
    pub digest: String,
    pub parents: Vec<CausalParent>,
    pub iteration: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidation_reason: Option<String>,
}

impl ArtifactRevision {
    fn public_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "kind": self.kind,
            "digest": self.digest,
            "parents": self.parents.iter().map(|parent| serde_json::json!({
                "id": parent.revision_id,
                "digest": parent.digest,
            })).collect::<Vec<_>>(),
        })
    }
}

/// The content identity currently active at each lifecycle stage.  Evidence is
/// valid only when it observes this complete tuple, rather than a digest from
/// a single stage in isolation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveDigestTuple {
    pub wi_digest: Option<String>,
    pub ec_digest: Option<String>,
    pub td_digest: Option<String>,
    pub cb_digest: Option<String>,
}

impl ActiveDigestTuple {
    fn matches(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceBinding {
    pub verifier: String,
    pub bound_tuple: ActiveDigestTuple,
    pub passed: bool,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvalidationRecord {
    pub trigger_revision_id: String,
    pub trigger_kind: ArtifactKind,
    pub invalidated_kinds: Vec<ArtifactKind>,
    pub invalidated_revision_ids: Vec<String>,
    #[serde(default)]
    pub evicted_evidence: Vec<EvidenceBinding>,
    pub evicted_evidence_verifiers: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleEventKind {
    WiCreate,
    WiChange,
    EcChange,
    TdChange,
    CbChange,
    EcVerify,
    TdReconcile,
    Feedback,
    Blocked,
    Rebind,
    StalePredecessor,
    Malformed,
    CbCommit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub event_id: String,
    pub predecessor_id: Option<String>,
    pub kind: LifecycleEventKind,
    pub candidate_revision: ArtifactRevision,
    #[serde(default)]
    pub bound_tuple: ActiveDigestTuple,
    pub next_command: String,
    pub next_owner: OwnerVocabulary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wi_snapshot: Option<CanonicalWiSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextObligation {
    pub command: String,
    pub owner: OwnerVocabulary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeLifecycle {
    pub schema: String,
    pub slug: String,
    pub epoch: u64,
    pub head_event_id: Option<String>,
    pub active_revisions: BTreeMap<ArtifactKind, Option<ArtifactRevision>>,
    pub events: Vec<LifecycleEvent>,
    #[serde(default)]
    pub evidence: Vec<EvidenceBinding>,
    #[serde(default)]
    pub invalidations: Vec<InvalidationRecord>,
    pub iteration: u64,
    pub terminal: bool,
    pub next: NextObligation,
}

impl ChangeLifecycle {
    pub fn wi_snapshot(&self) -> Option<CanonicalWiSnapshot> {
        self.events.iter().rev().find_map(|e| e.wi_snapshot.clone())
    }

    pub fn active_digest_tuple(&self) -> ActiveDigestTuple {
        let digest = |kind| {
            self.active_revisions
                .get(&kind)
                .and_then(|revision| revision.as_ref())
                .map(|revision| revision.digest.clone())
        };
        ActiveDigestTuple {
            wi_digest: digest(ArtifactKind::Wi),
            ec_digest: digest(ArtifactKind::Ec),
            td_digest: digest(ArtifactKind::Td),
            cb_digest: digest(ArtifactKind::Cb),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureOwnership {
    WiDrift,
    Contract,
    Design,
    Implementation,
    Infrastructure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReducerResult {
    pub lifecycle: ChangeLifecycle,
    pub accepted: bool,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilestoneWork {
    pub event_id: String,
    pub milestone: serde_json::Value,
}

const PROJECTION_MARKER_START: &str = "<!-- aw:projection";
const PROJECTION_MARKER_END: &str = "-->";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionMarker {
    pub version: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epoch: Option<u64>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub present_event_ids: BTreeSet<String>,
}

pub fn render_projection_marker(
    head_event_id: Option<&str>,
    epoch: Option<u64>,
    present_event_ids: &BTreeSet<String>,
) -> String {
    let marker = ProjectionMarker {
        version: 1,
        head_event_id: head_event_id.map(String::from),
        epoch,
        present_event_ids: present_event_ids.clone(),
    };
    let yaml = serde_yaml::to_string(&marker).unwrap_or_default();
    format!(
        "{PROJECTION_MARKER_START}\n{}\n{PROJECTION_MARKER_END}\n",
        yaml.trim_end()
    )
}

pub fn render_projection_marker_for_lifecycle(lifecycle: &ChangeLifecycle) -> String {
    let present_event_ids = lifecycle
        .events
        .iter()
        .map(|e| e.event_id.clone())
        .collect();
    render_projection_marker(
        lifecycle.head_event_id.as_deref(),
        Some(lifecycle.epoch),
        &present_event_ids,
    )
}

pub fn upsert_projection_marker(body: &str, lifecycle: &ChangeLifecycle) -> Result<String> {
    let block = render_projection_marker_for_lifecycle(lifecycle);
    if let Some(start) = body.find(PROJECTION_MARKER_START) {
        let rest = &body[start + PROJECTION_MARKER_START.len()..];
        if let Some(end_rel) = rest.find(PROJECTION_MARKER_END) {
            let end = start + PROJECTION_MARKER_START.len() + end_rel + PROJECTION_MARKER_END.len();
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

pub fn parse_projection_marker(body: &str) -> Option<ProjectionMarker> {
    let start = body.find(PROJECTION_MARKER_START)?;
    let rest = &body[start + PROJECTION_MARKER_START.len()..];
    let end = rest.find(PROJECTION_MARKER_END)?;
    let yaml = rest[..end].trim();
    serde_yaml::from_str(yaml).ok()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TrackerObservation {
    pub body: String,
    pub head_event_id: Option<String>,
    pub epoch: Option<u64>,
    pub present_event_ids: BTreeSet<String>,
    #[serde(default)]
    pub state: Option<IssueState>,
    #[serde(default)]
    pub snapshot: Option<CanonicalWiSnapshot>,
}

impl TrackerObservation {
    pub fn new(
        body: impl Into<String>,
        head_event_id: Option<impl Into<String>>,
        epoch: Option<u64>,
        present_event_ids: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            body: body.into(),
            head_event_id: head_event_id.map(Into::into),
            epoch,
            present_event_ids: present_event_ids.into_iter().map(Into::into).collect(),
            state: Some(IssueState::Open),
            snapshot: None,
        }
    }

    pub fn empty(body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            head_event_id: None,
            epoch: None,
            present_event_ids: BTreeSet::new(),
            state: Some(IssueState::Open),
            snapshot: None,
        }
    }

    pub fn from_body(body: impl Into<String>) -> Self {
        let body_str = body.into();
        if let Some(start) = body_str.find(PROJECTION_MARKER_START) {
            let rest = &body_str[start + PROJECTION_MARKER_START.len()..];
            if let Some(end) = rest.find(PROJECTION_MARKER_END) {
                let yaml = rest[..end].trim();
                if let Ok(marker) = serde_yaml::from_str::<ProjectionMarker>(yaml) {
                    return Self {
                        body: body_str,
                        head_event_id: marker.head_event_id,
                        epoch: marker.epoch,
                        present_event_ids: marker.present_event_ids,
                        state: Some(IssueState::Open),
                        snapshot: None,
                    };
                }
            }
            Self {
                body: body_str,
                head_event_id: Some("malformed-projection-marker".to_string()),
                epoch: None,
                present_event_ids: BTreeSet::new(),
                state: Some(IssueState::Open),
                snapshot: None,
            }
        } else {
            Self {
                body: body_str,
                head_event_id: None,
                epoch: None,
                present_event_ids: BTreeSet::new(),
                state: Some(IssueState::Open),
                snapshot: None,
            }
        }
    }

    pub fn from_issue(issue: &Issue) -> Self {
        let snapshot = CanonicalWiSnapshot::from_issue(issue);
        let mut obs = Self::from_body(&issue.body).with_state(issue.state);
        obs.snapshot = Some(snapshot);
        obs
    }

    pub fn wi_digest(&self, lifecycle: &ChangeLifecycle) -> String {
        if let Some(ref snapshot) = self.snapshot {
            if let Some(committed) = lifecycle.wi_snapshot() {
                if !committed.ownership_observed {
                    return committed.with_body(&self.body).digest();
                }
            }
            snapshot.digest()
        } else if let Some(committed) = lifecycle.wi_snapshot() {
            committed.with_body(&self.body).digest()
        } else {
            CanonicalWiSnapshot::from_parts(
                &lifecycle.slug,
                "",
                "change",
                "agentic-workflow",
                Vec::new(),
                Vec::new(),
                &self.body,
            )
            .digest()
        }
    }

    pub fn with_state(mut self, state: IssueState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.state, Some(IssueState::Closed))
    }

    pub fn apply_work(
        &mut self,
        target_epoch: u64,
        target_head: Option<String>,
        work: &[MilestoneWork],
    ) {
        self.epoch = Some(target_epoch);
        self.head_event_id = target_head;
        for w in work {
            self.present_event_ids.insert(w.event_id.clone());
        }
    }

    pub fn apply_close(&mut self) {
        self.state = Some(IssueState::Closed);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionDecision {
    pub accepted: bool,
    pub refusal_reason: Option<String>,
    pub target_epoch: u64,
    pub target_head_event_id: Option<String>,
    pub work: Vec<MilestoneWork>,
    pub complete: bool,
    pub close_authorized: bool,
    pub authorize_close_event_id: Option<String>,
    pub drift: bool,
    pub remediation: Vec<NextObligation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invalidated_kinds: Vec<ArtifactKind>,
}

pub(crate) fn carrier_path(project_root: &Path, slug: &str) -> PathBuf {
    crate::shared::workspace::workspace_runtime_path(project_root)
        .join("causal-lifecycle")
        .join(format!("{slug}.json"))
}

fn canonical_digest(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn strip_aw_marker_blocks(body: &str) -> String {
    let mut result = String::with_capacity(body.len());
    let mut remaining = body;

    while let Some(start_idx) = remaining.find("<!--") {
        let after_start = &remaining[start_idx + 4..];
        let trimmed_after = after_start.trim_start();
        if trimmed_after.starts_with("aw:") || trimmed_after.starts_with("score:") {
            if let Some(end_rel) = after_start.find("-->") {
                let end_idx = start_idx + 4 + end_rel + 3;
                result.push_str(&remaining[..start_idx]);
                remaining = &remaining[end_idx..];
                continue;
            }
        }
        result.push_str(&remaining[..start_idx + 4]);
        remaining = after_start;
    }
    result.push_str(remaining);
    result
}

const CANONICAL_WI_SECTIONS: &[&str] = &[
    "## Problem",
    "## Capability Alignment",
    "## Requirements",
    "## Scope",
    "## Acceptance Criteria",
    "## Reference Context",
];

fn canonicalize_wi_body(body: &str) -> String {
    let stripped = strip_aw_marker_blocks(body);
    let sections = crate::cli::issues::split_body_by_h2(&stripped);
    let canonical_sections: Vec<_> = sections
        .into_iter()
        .filter(|(heading, _)| CANONICAL_WI_SECTIONS.contains(&heading.as_str()))
        .collect();

    let target_body = if canonical_sections.is_empty() {
        stripped
    } else {
        let mut out = String::new();
        for (heading, content) in &canonical_sections {
            out.push_str(heading);
            out.push('\n');
            out.push_str(content);
            if !content.ends_with('\n') {
                out.push('\n');
            }
        }
        out
    };

    let normalized = target_body.replace("\r\n", "\n");
    let mut lines = Vec::new();
    let mut previous_blank = true;
    for line in normalized.lines() {
        let trimmed = line.trim_end();
        let blank = trimmed.is_empty();
        if blank && previous_blank {
            continue;
        }
        lines.push(trimmed);
        previous_blank = blank;
    }
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

fn normalize_reference_val(value: &str) -> String {
    crate::issues::graph::dependency_label(value)
        .strip_prefix("depends-on:")
        .unwrap_or(value)
        .to_string()
}

fn reference_sort_key(value: &str) -> (bool, u64, String) {
    let norm = normalize_reference_val(value);
    if let Ok(num) = norm.parse::<u64>() {
        (true, num, norm)
    } else {
        (false, 0, norm)
    }
}

fn normalize_references(refs: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
    let mut vec = refs
        .into_iter()
        .map(|r| normalize_reference_val(r.as_ref()))
        .filter(|r| !r.is_empty())
        .collect::<Vec<_>>();
    vec.sort_by(|left, right| reference_sort_key(left).cmp(&reference_sort_key(right)));
    vec.dedup();
    vec
}

fn dependencies_for_labels(labels: &[String]) -> Vec<String> {
    let raw = labels
        .iter()
        .filter_map(|label| label.strip_prefix("depends-on:"))
        .collect::<Vec<_>>();
    normalize_references(raw)
}

fn project_for_labels(labels: &[String]) -> &str {
    labels
        .iter()
        .find_map(|label| {
            label
                .strip_prefix("app:")
                .or_else(|| label.strip_prefix("lib:"))
                .or_else(|| label.strip_prefix("project:"))
        })
        .unwrap_or("agentic-workflow")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CanonicalWiSnapshot {
    pub identity: String,
    pub title: String,
    pub issue_type: String,
    pub project: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub epic_parents: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    pub body: String,
    #[serde(default)]
    pub ownership_observed: bool,
}

impl CanonicalWiSnapshot {
    pub fn from_issue(issue: &Issue) -> Self {
        let identity = issue.slug.trim().to_string();
        let title = issue.title.split_whitespace().collect::<Vec<_>>().join(" ");
        let issue_type = issue.issue_type.as_str().to_string();
        let project = project_for(issue).to_string();
        let epic_parents = crate::issues::graph::explicit_parent_references(issue);
        let dependencies = dependencies_for_labels(&issue.labels);
        let body = issue.body.clone();

        Self {
            identity,
            title,
            issue_type,
            project,
            epic_parents,
            dependencies,
            body,
            ownership_observed: true,
        }
    }

    pub fn from_parts(
        slug: &str,
        title: &str,
        issue_type: &str,
        project: &str,
        epic_parents: Vec<String>,
        dependencies: Vec<String>,
        body: &str,
    ) -> Self {
        let identity = slug.trim().to_string();
        let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
        let issue_type = if issue_type.is_empty() {
            "change".to_string()
        } else {
            issue_type.to_string()
        };
        let project = if project.is_empty() {
            "agentic-workflow".to_string()
        } else {
            project.to_string()
        };
        let epic_parents = normalize_references(epic_parents);
        let dependencies = normalize_references(dependencies);
        Self {
            identity,
            title,
            issue_type,
            project,
            epic_parents,
            dependencies,
            body: body.to_string(),
            ownership_observed: false,
        }
    }

    pub fn with_body(&self, new_body: &str) -> Self {
        let mut cloned = self.clone();
        cloned.body = new_body.to_string();
        cloned
    }

    pub fn digest(&self) -> String {
        let canonical_body = canonicalize_wi_body(&self.body);
        let normalized = format!(
            "identity:{}\ntitle:{}\ntype:{}\nproject:{}\nepics:{}\ndeps:{}\nbody:\n{}",
            self.identity,
            self.title,
            self.issue_type,
            self.project,
            self.epic_parents.join(","),
            self.dependencies.join(","),
            canonical_body
        );
        canonical_digest(&normalized)
    }
}

fn revision_id(kind: ArtifactKind, digest: &str, parents: &[CausalParent]) -> String {
    let mut parent_pairs = parents
        .iter()
        .map(|parent| format!("{}:{}", parent.revision_id, parent.digest))
        .collect::<Vec<_>>();
    parent_pairs.sort();
    let kind = match kind {
        ArtifactKind::Wi => "wi",
        ArtifactKind::Ec => "ec",
        ArtifactKind::Td => "td",
        ArtifactKind::Cb => "cb",
    };
    let raw = format!("{kind}:{digest}:{}", parent_pairs.join(","));
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    format!("rev-{}", &digest[..12])
}

fn project_for(issue: &Issue) -> &str {
    project_for_labels(&issue.labels)
}

fn is_legacy_loop_state(issue: &Issue) -> bool {
    issue.body.contains("<!-- aw:loop-state")
}

fn empty_revisions() -> BTreeMap<ArtifactKind, Option<ArtifactRevision>> {
    BTreeMap::from([
        (ArtifactKind::Wi, None),
        (ArtifactKind::Ec, None),
        (ArtifactKind::Td, None),
        (ArtifactKind::Cb, None),
    ])
}

fn artifact_revision(
    kind: ArtifactKind,
    digest: String,
    parents: Vec<CausalParent>,
    iteration: u64,
) -> ArtifactRevision {
    ArtifactRevision {
        id: revision_id(kind, &digest, &parents),
        kind,
        digest,
        parents,
        iteration,
        superseded_by: None,
        invalidation_reason: None,
    }
}

fn event_id(lifecycle: &ChangeLifecycle) -> String {
    format!("evt-{:03}", lifecycle.events.len() + 1)
}

fn wi_remediation(slug: &str) -> NextObligation {
    NextObligation {
        command: format!("aw wi change {slug}"),
        owner: OwnerVocabulary::Wi,
    }
}

pub fn derive_evidence_next_obligation(
    slug: &str,
    evidence: &[EvidenceBinding],
    active_tuple: &ActiveDigestTuple,
    fallback: &NextObligation,
) -> NextObligation {
    if evidence
        .iter()
        .any(|b| b.verifier == "wi-review" && b.passed && b.bound_tuple.matches(active_tuple))
    {
        NextObligation {
            command: format!("aw wi commit {slug}"),
            owner: OwnerVocabulary::Wi,
        }
    } else if evidence
        .iter()
        .any(|b| b.verifier == "wi-review" && !b.passed && b.bound_tuple.matches(active_tuple))
    {
        wi_remediation(slug)
    } else if evidence
        .iter()
        .any(|b| b.verifier == "wi-test" && b.passed && b.bound_tuple.matches(active_tuple))
    {
        NextObligation {
            command: format!("aw wi review {slug}"),
            owner: OwnerVocabulary::Wi,
        }
    } else if evidence
        .iter()
        .any(|b| b.verifier == "wi-test" && !b.passed && b.bound_tuple.matches(active_tuple))
    {
        wi_remediation(slug)
    } else {
        fallback.clone()
    }
}

/// Pure ownership routing for rejected or blocked stage evidence.
pub fn route_failure(
    failure: FailureOwnership,
    slug: &str,
    current_command: &str,
) -> NextObligation {
    match failure {
        FailureOwnership::WiDrift => wi_remediation(slug),
        FailureOwnership::Contract => NextObligation {
            command: "aw ec check".to_string(),
            owner: OwnerVocabulary::Ec,
        },
        FailureOwnership::Design => NextObligation {
            command: format!("aw td check --wi {slug}"),
            owner: OwnerVocabulary::Td,
        },
        FailureOwnership::Implementation => NextObligation {
            command: format!("aw cb check {slug}"),
            owner: OwnerVocabulary::Cb,
        },
        FailureOwnership::Infrastructure => NextObligation {
            command: current_command.to_string(),
            owner: OwnerVocabulary::Cb,
        },
    }
}

/// Return the exact revision-aware parent set required for a candidate.
pub fn expected_parent_set(
    lifecycle: &ChangeLifecycle,
    kind: ArtifactKind,
) -> Option<Vec<CausalParent>> {
    let upstream: &[ArtifactKind] = match kind {
        ArtifactKind::Wi => &[],
        ArtifactKind::Ec => &[ArtifactKind::Wi],
        ArtifactKind::Td => &[ArtifactKind::Wi, ArtifactKind::Ec],
        ArtifactKind::Cb => &[ArtifactKind::Wi, ArtifactKind::Ec, ArtifactKind::Td],
    };
    upstream
        .iter()
        .map(|kind| {
            lifecycle
                .active_revisions
                .get(kind)
                .and_then(|revision| revision.as_ref())
                .map(|revision| CausalParent {
                    revision_id: revision.id.clone(),
                    digest: revision.digest.clone(),
                })
        })
        .collect()
}

pub fn transitive_invalidation_kinds(trigger_kind: ArtifactKind) -> Vec<ArtifactKind> {
    match trigger_kind {
        ArtifactKind::Wi => vec![ArtifactKind::Ec, ArtifactKind::Td, ArtifactKind::Cb],
        ArtifactKind::Ec => vec![ArtifactKind::Td, ArtifactKind::Cb],
        ArtifactKind::Td => vec![ArtifactKind::Cb],
        ArtifactKind::Cb => Vec::new(),
    }
}

fn transitive_invalidation(
    trigger: &ArtifactRevision,
    current: &BTreeMap<ArtifactKind, Option<ArtifactRevision>>,
    evidence: &[EvidenceBinding],
) -> InvalidationRecord {
    let invalidated_kinds = transitive_invalidation_kinds(trigger.kind);
    let invalidated_revision_ids = invalidated_kinds
        .iter()
        .filter_map(|kind| {
            current
                .get(kind)
                .and_then(|revision| revision.as_ref())
                .map(|revision| revision.id.clone())
        })
        .collect();
    let mut evicted_evidence_verifiers = evidence
        .iter()
        .map(|binding| binding.verifier.clone())
        .collect::<Vec<_>>();
    evicted_evidence_verifiers.sort();
    evicted_evidence_verifiers.dedup();
    InvalidationRecord {
        trigger_revision_id: trigger.id.clone(),
        trigger_kind: trigger.kind,
        invalidated_kinds,
        invalidated_revision_ids,
        evicted_evidence: evidence.to_vec(),
        evicted_evidence_verifiers,
        reason: format!(
            "Transitive invalidation triggered by {} revision {}",
            match trigger.kind {
                ArtifactKind::Wi => "wi",
                ArtifactKind::Ec => "ec",
                ArtifactKind::Td => "td",
                ArtifactKind::Cb => "cb",
            },
            trigger.id
        ),
    }
}

pub fn fold_wi_create_from_snapshot(snapshot: CanonicalWiSnapshot) -> ChangeLifecycle {
    let slug = snapshot.identity.clone();
    let digest = snapshot.digest();
    let project = snapshot.project.clone();
    let revision = artifact_revision(ArtifactKind::Wi, digest.clone(), Vec::new(), 1);
    let command = crate::cli::run::ec_draft_command(&project, &slug);
    let mut active_revisions = empty_revisions();
    active_revisions.insert(ArtifactKind::Wi, Some(revision.clone()));
    ChangeLifecycle {
        schema: SCHEMA.to_string(),
        slug,
        epoch: 1,
        head_event_id: Some("evt-001".to_string()),
        active_revisions,
        events: vec![LifecycleEvent {
            event_id: "evt-001".to_string(),
            predecessor_id: None,
            kind: LifecycleEventKind::WiCreate,
            candidate_revision: revision,
            bound_tuple: ActiveDigestTuple {
                wi_digest: Some(digest),
                ..ActiveDigestTuple::default()
            },
            next_command: command.clone(),
            next_owner: OwnerVocabulary::Wi,
            wi_snapshot: Some(snapshot),
        }],
        evidence: Vec::new(),
        invalidations: Vec::new(),
        iteration: 1,
        terminal: false,
        next: NextObligation {
            command,
            owner: OwnerVocabulary::Wi,
        },
    }
}

pub fn fold_wi_create_from_issue(issue: &Issue) -> ChangeLifecycle {
    let snapshot = CanonicalWiSnapshot::from_issue(issue);
    fold_wi_create_from_snapshot(snapshot)
}

/// Construct the post-backend-success WI creation event and durable record.
pub fn fold_wi_create(slug: &str, body: &str, project: &str) -> ChangeLifecycle {
    let snapshot =
        CanonicalWiSnapshot::from_parts(slug, "", "change", project, Vec::new(), Vec::new(), body);
    fold_wi_create_from_snapshot(snapshot)
}

fn initial_lifecycle(issue: &Issue) -> ChangeLifecycle {
    fold_wi_create_from_issue(issue)
}

pub fn fold_wi_update_from_snapshots(
    prior_lifecycle: &ChangeLifecycle,
    new_snapshot: &CanonicalWiSnapshot,
    pre_update_snapshot: Option<&CanonicalWiSnapshot>,
) -> ReducerResult {
    let stored_digest = prior_lifecycle
        .active_revisions
        .get(&ArtifactKind::Wi)
        .and_then(|revision| revision.as_ref())
        .map(|revision| revision.digest.clone());

    if let (Some(observed_snap), Some(stored)) = (pre_update_snapshot, &stored_digest) {
        let observed_digest = observed_snap.digest();
        if observed_digest != *stored {
            return ReducerResult {
                lifecycle: prior_lifecycle.clone(),
                accepted: false,
                rejection_reason: Some(format!(
                    "Conflicting tracker observation digest {:?} does not match stored WI digest {:?}",
                    observed_digest, stored
                )),
            };
        }
    }

    let new_digest = new_snapshot.digest();
    let old_digest = stored_digest.or_else(|| pre_update_snapshot.map(|s| s.digest()));
    if old_digest.as_deref() == Some(new_digest.as_str()) {
        return ReducerResult {
            lifecycle: prior_lifecycle.clone(),
            accepted: false,
            rejection_reason: Some("No-op transition: unchanged WI content digest".to_string()),
        };
    }
    let revision = artifact_revision(
        ArtifactKind::Wi,
        new_digest.clone(),
        Vec::new(),
        prior_lifecycle.iteration + 1,
    );
    reduce_event(
        prior_lifecycle,
        LifecycleEvent {
            event_id: event_id(prior_lifecycle),
            predecessor_id: prior_lifecycle.head_event_id.clone(),
            kind: LifecycleEventKind::WiChange,
            candidate_revision: revision,
            bound_tuple: ActiveDigestTuple {
                wi_digest: Some(new_digest),
                ..ActiveDigestTuple::default()
            },
            next_command: format!("aw wi validate {}", prior_lifecycle.slug),
            next_owner: OwnerVocabulary::Wi,
            wi_snapshot: Some(new_snapshot.clone()),
        },
    )
}

pub fn fold_wi_update_from_issues(
    prior_lifecycle: &ChangeLifecycle,
    updated_issue: &Issue,
    pre_update_issue: Option<&Issue>,
) -> ReducerResult {
    let new_snapshot = CanonicalWiSnapshot::from_issue(updated_issue);
    let pre_update_snapshot = pre_update_issue.map(CanonicalWiSnapshot::from_issue);
    fold_wi_update_from_snapshots(prior_lifecycle, &new_snapshot, pre_update_snapshot.as_ref())
}

/// Fold a body update against the persisted carrier.  Equal canonical bodies
/// remain event-free so callers can preserve the carrier bytes exactly.
pub fn fold_wi_update(
    prior_lifecycle: &ChangeLifecycle,
    new_body: &str,
    pre_update_body: Option<&str>,
) -> ReducerResult {
    let slug = &prior_lifecycle.slug;
    let base_snap = prior_lifecycle.wi_snapshot().unwrap_or_else(|| {
        CanonicalWiSnapshot::from_parts(
            slug,
            "",
            "change",
            "agentic-workflow",
            Vec::new(),
            Vec::new(),
            "",
        )
    });

    let new_snapshot = base_snap.with_body(new_body);
    let pre_update_snapshot = pre_update_body.map(|b| base_snap.with_body(b));
    fold_wi_update_from_snapshots(prior_lifecycle, &new_snapshot, pre_update_snapshot.as_ref())
}

/// Deterministically accept or reject one append-only lifecycle event.
pub fn reduce_event(lifecycle: &ChangeLifecycle, event: LifecycleEvent) -> ReducerResult {
    if event.predecessor_id != lifecycle.head_event_id {
        let mut rejected = lifecycle.clone();
        rejected.next = wi_remediation(&lifecycle.slug);
        return ReducerResult {
            lifecycle: rejected,
            accepted: false,
            rejection_reason: Some(format!(
                "Conflicting/stale predecessor_id {:?} does not match current head_event_id {:?}",
                event.predecessor_id, lifecycle.head_event_id
            )),
        };
    }

    let candidate = &event.candidate_revision;
    let target_kind = candidate.kind;
    let active = lifecycle
        .active_revisions
        .get(&target_kind)
        .and_then(|revision| revision.as_ref());
    if expected_parent_set(lifecycle, target_kind).as_deref() != Some(candidate.parents.as_slice())
    {
        let mut rejected = lifecycle.clone();
        rejected.next = wi_remediation(&lifecycle.slug);
        return ReducerResult {
            lifecycle: rejected,
            accepted: false,
            rejection_reason: Some(
                "candidate parent set does not match the active causal predecessor set".to_string(),
            ),
        };
    }

    let is_commit = event.kind == LifecycleEventKind::CbCommit;
    let (active_revisions, invalidation, retained_evidence) = if is_commit {
        if candidate.kind != ArtifactKind::Cb {
            let mut rejected = lifecycle.clone();
            rejected.next = route_failure(
                FailureOwnership::Implementation,
                &lifecycle.slug,
                &lifecycle.next.command,
            );
            return ReducerResult {
                lifecycle: rejected,
                accepted: false,
                rejection_reason: Some(
                    "cb_commit candidate must be the current active CB revision".to_string(),
                ),
            };
        }
        if active != Some(candidate) {
            let mut rejected = lifecycle.clone();
            rejected.next = route_failure(
                FailureOwnership::Implementation,
                &lifecycle.slug,
                &lifecycle.next.command,
            );
            return ReducerResult {
                lifecycle: rejected,
                accepted: false,
                rejection_reason: Some(
                    "cb_commit candidate is not the current active CB revision".to_string(),
                ),
            };
        }
        (
            lifecycle.active_revisions.clone(),
            None,
            lifecycle.evidence.clone(),
        )
    } else {
        if active.is_some_and(|current| {
            current.digest == candidate.digest && current.parents == candidate.parents
        }) {
            let mut rejected = lifecycle.clone();
            rejected.next = wi_remediation(&lifecycle.slug);
            return ReducerResult {
                lifecycle: rejected,
                accepted: false,
                rejection_reason: Some(
                    "No-op transition: unchanged content and unchanged causal parents".to_string(),
                ),
            };
        }
        let invalidation =
            transitive_invalidation(candidate, &lifecycle.active_revisions, &lifecycle.evidence);
        let mut active_revisions = lifecycle.active_revisions.clone();
        active_revisions.insert(target_kind, Some(candidate.clone()));
        for kind in &invalidation.invalidated_kinds {
            active_revisions.insert(*kind, None);
        }
        (active_revisions, Some(invalidation), Vec::new())
    };

    if is_commit {
        let tuple = ChangeLifecycle {
            active_revisions: active_revisions.clone(),
            ..lifecycle.clone()
        }
        .active_digest_tuple();
        let complete = tuple.wi_digest.is_some()
            && tuple.ec_digest.is_some()
            && tuple.td_digest.is_some()
            && tuple.cb_digest.is_some();
        let required = ["cb_test", "cb_review", "td_reconcile", "ec_verify_cb"];
        let mut present = retained_evidence
            .iter()
            .filter(|binding| {
                binding.passed
                    && binding.bound_tuple.matches(&tuple)
                    && required.contains(&binding.verifier.as_str())
            })
            .map(|binding| binding.verifier.as_str())
            .collect::<Vec<_>>();
        present.sort_unstable();
        present.dedup();
        if !complete || present.len() != required.len() {
            let mut rejected = lifecycle.clone();
            rejected.next = NextObligation {
                command: "aw ec verify --stage cb".to_string(),
                owner: OwnerVocabulary::Cb,
            };
            return ReducerResult {
                lifecycle: rejected,
                accepted: false,
                rejection_reason: Some(if !complete {
                    "cb_commit rejected: active digest tuple is incomplete across WI/EC/TD/CB"
                        .to_string()
                } else {
                    "cb_commit rejected: missing valid 4D active-tuple evidence".to_string()
                }),
            };
        }
    }

    let mut accepted = lifecycle.clone();
    accepted.epoch += 1;
    accepted.head_event_id = Some(event.event_id.clone());
    accepted.events.push(event.clone());
    accepted.active_revisions = active_revisions;
    accepted.iteration = accepted.iteration.max(event.candidate_revision.iteration);
    accepted.evidence = retained_evidence;
    if let Some(invalidation) = invalidation {
        accepted.invalidations.push(invalidation);
    }
    if is_commit {
        accepted.terminal = true;
        accepted.next = NextObligation {
            command: format!("aw wi show {}", accepted.slug),
            owner: OwnerVocabulary::Cb,
        };
    } else {
        accepted.terminal = false;
        accepted.next = NextObligation {
            command: event.next_command,
            owner: event.next_owner,
        };
    }
    ReducerResult {
        lifecycle: accepted,
        accepted: true,
        rejection_reason: None,
    }
}

pub(crate) fn load(project_root: &Path, slug: &str) -> Result<Option<ChangeLifecycle>> {
    let path = carrier_path(project_root, slug);
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read(&path)
        .with_context(|| format!("failed to read causal lifecycle carrier {}", path.display()))?;
    serde_json::from_slice(&contents)
        .with_context(|| {
            format!(
                "failed to parse causal lifecycle carrier {}",
                path.display()
            )
        })
        .map(Some)
}

fn event_kind_accepts_candidate(kind: LifecycleEventKind, candidate: ArtifactKind) -> bool {
    match kind {
        LifecycleEventKind::WiCreate | LifecycleEventKind::WiChange => {
            candidate == ArtifactKind::Wi
        }
        LifecycleEventKind::EcChange | LifecycleEventKind::EcVerify => {
            candidate == ArtifactKind::Ec
        }
        LifecycleEventKind::TdChange | LifecycleEventKind::TdReconcile => {
            candidate == ArtifactKind::Td
        }
        LifecycleEventKind::CbChange | LifecycleEventKind::CbCommit => {
            candidate == ArtifactKind::Cb
        }
        LifecycleEventKind::Rebind => candidate != ArtifactKind::Wi,
        // These record a stage-local remediation result.  Their exact owner is
        // represented by the obligation; their candidate still goes through
        // the parent and active-stage checks during replay below.
        LifecycleEventKind::Feedback
        | LifecycleEventKind::Blocked
        | LifecycleEventKind::StalePredecessor
        | LifecycleEventKind::Malformed => true,
    }
}

fn replay_seed(slug: &str) -> ChangeLifecycle {
    ChangeLifecycle {
        schema: SCHEMA.to_string(),
        slug: slug.to_string(),
        epoch: 0,
        head_event_id: None,
        active_revisions: empty_revisions(),
        events: Vec::new(),
        evidence: Vec::new(),
        invalidations: Vec::new(),
        iteration: 1,
        terminal: false,
        next: wi_remediation(slug),
    }
}

fn same_invalidation_shape(persisted: &InvalidationRecord, replayed: &InvalidationRecord) -> bool {
    persisted.trigger_revision_id == replayed.trigger_revision_id
        && persisted.trigger_kind == replayed.trigger_kind
        && persisted.invalidated_kinds == replayed.invalidated_kinds
        && persisted.invalidated_revision_ids == replayed.invalidated_revision_ids
        && persisted.evicted_evidence == replayed.evicted_evidence
        && persisted.evicted_evidence_verifiers == replayed.evicted_evidence_verifiers
        && persisted.reason == replayed.reason
}

/// Validate persisted state by replaying every immutable event through the
/// same reducer that accepts live transitions.  A JSON-shaped record is not
/// authoritative until each stage's revision kind, parent set, bound tuple,
/// predecessor, terminal state, and resulting active revision set agree.
fn valid_persisted_lifecycle(lifecycle: &ChangeLifecycle, requested_slug: &str) -> bool {
    if lifecycle.schema != SCHEMA || lifecycle.slug != requested_slug {
        return false;
    }
    if lifecycle.events.is_empty() {
        return lifecycle == &replay_seed(requested_slug);
    }
    if lifecycle.events[0].predecessor_id.is_some()
        || lifecycle.head_event_id.as_deref()
            != lifecycle.events.last().map(|event| event.event_id.as_str())
    {
        return false;
    }
    let mut ids = BTreeSet::new();
    let mut replayed = replay_seed(requested_slug);
    for (index, event) in lifecycle.events.iter().enumerate() {
        if replayed.terminal
            || !ids.insert(event.event_id.as_str())
            || !event_kind_accepts_candidate(event.kind, event.candidate_revision.kind)
            || event.candidate_revision.id
                != revision_id(
                    event.candidate_revision.kind,
                    &event.candidate_revision.digest,
                    &event.candidate_revision.parents,
                )
        {
            return false;
        }
        if index == 0 {
            if event.event_id != "evt-001"
                || event.kind != LifecycleEventKind::WiCreate
                || event.candidate_revision.kind != ArtifactKind::Wi
                || !event.candidate_revision.parents.is_empty()
                || event.candidate_revision.iteration != 1
            {
                return false;
            }
            let mut active_revisions = empty_revisions();
            active_revisions.insert(ArtifactKind::Wi, Some(event.candidate_revision.clone()));
            replayed = ChangeLifecycle {
                schema: SCHEMA.to_string(),
                slug: requested_slug.to_string(),
                epoch: 1,
                head_event_id: Some(event.event_id.clone()),
                active_revisions,
                events: vec![event.clone()],
                evidence: Vec::new(),
                invalidations: Vec::new(),
                iteration: 1,
                terminal: false,
                next: NextObligation {
                    command: event.next_command.clone(),
                    owner: event.next_owner,
                },
            };
            if event.bound_tuple != replayed.active_digest_tuple() {
                return false;
            }
            continue;
        }
        if event.kind != LifecycleEventKind::CbCommit {
            let Some(invalidation) = lifecycle.invalidations.get(replayed.invalidations.len())
            else {
                return false;
            };
            let active_tuple = replayed.active_digest_tuple();
            if invalidation
                .evicted_evidence
                .iter()
                .any(|binding| !binding.bound_tuple.matches(&active_tuple))
            {
                return false;
            }
            replayed.evidence = invalidation.evicted_evidence.clone();
        }
        // Evidence is persisted separately from its observation events.  It
        // is needed only to validate a terminal commit against the final 4D
        // tuple, so seed it precisely for that event and nowhere else.
        if event.kind == LifecycleEventKind::CbCommit {
            if index + 1 != lifecycle.events.len() {
                return false;
            }
            replayed.evidence = lifecycle.evidence.clone();
        }
        let result = reduce_event(&replayed, event.clone());
        if !result.accepted || event.bound_tuple != result.lifecycle.active_digest_tuple() {
            return false;
        }
        replayed = result.lifecycle;
    }
    let expected_next = derive_evidence_next_obligation(
        requested_slug,
        &lifecycle.evidence,
        &replayed.active_digest_tuple(),
        &replayed.next,
    );

    replayed.epoch == lifecycle.epoch
        && replayed.head_event_id == lifecycle.head_event_id
        && replayed.active_revisions == lifecycle.active_revisions
        && replayed.iteration == lifecycle.iteration
        && replayed.terminal == lifecycle.terminal
        && expected_next == lifecycle.next
        && lifecycle.invalidations.len() == replayed.invalidations.len()
        && lifecycle
            .invalidations
            .iter()
            .zip(&replayed.invalidations)
            .all(|(persisted, replayed)| same_invalidation_shape(persisted, replayed))
}

pub(crate) fn save(project_root: &Path, lifecycle: &ChangeLifecycle) -> Result<()> {
    let path = carrier_path(project_root, &lifecycle.slug);
    let parent = path.parent().expect("carrier path has parent");
    std::fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create causal lifecycle directory {}",
            parent.display()
        )
    })?;
    let bytes = serde_json::to_vec_pretty(lifecycle)?;
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, bytes)
        .with_context(|| format!("failed to write causal lifecycle carrier {}", tmp.display()))?;
    std::fs::rename(&tmp, &path).with_context(|| {
        format!(
            "failed to publish causal lifecycle carrier {}",
            path.display()
        )
    })?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PublishOutcome {
    Applied,
    AlreadyApplied(String),
    Refused(String),
}

impl PublishOutcome {
    pub(crate) fn is_applied(&self) -> bool {
        matches!(self, Self::Applied)
    }

    pub(crate) fn is_already_applied(&self) -> bool {
        matches!(self, Self::AlreadyApplied(_))
    }

    pub(crate) fn is_refused(&self) -> bool {
        matches!(self, Self::Refused(_))
    }
}

pub(crate) struct LeaseGuard {
    lock_path: PathBuf,
}

impl Drop for LeaseGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

pub(crate) fn acquire_project_lease(project_root: &Path) -> Result<Option<LeaseGuard>> {
    let dir =
        crate::shared::workspace::workspace_runtime_path(project_root).join("causal-lifecycle");
    std::fs::create_dir_all(&dir)?;
    let lock_path = dir.join(".publish.lock");
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    {
        Ok(_) => Ok(Some(LeaseGuard { lock_path })),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn publish_lifecycle_cas(
    project_root: &Path,
    expected_head_event_id: Option<&str>,
    candidate: &ChangeLifecycle,
) -> Result<PublishOutcome> {
    let _lease = match acquire_project_lease(project_root)? {
        Some(guard) => guard,
        None => {
            return Ok(PublishOutcome::Refused(
                "refused: held lease: exclusive project publish lease is currently held"
                    .to_string(),
            ));
        }
    };

    let candidate_event = candidate
        .events
        .last()
        .context("candidate lifecycle has no events")?;
    let cand_predecessor = candidate_event.predecessor_id.as_deref();
    let cand_rev_id = &candidate_event.candidate_revision.id;

    let disk_carrier_opt = load(project_root, &candidate.slug)?;

    if let Some(disk_carrier) = &disk_carrier_opt {
        let already_landed = disk_carrier.events.iter().any(|evt| {
            evt.predecessor_id.as_deref() == cand_predecessor
                && &evt.candidate_revision.id == cand_rev_id
        });
        if already_landed {
            return Ok(PublishOutcome::AlreadyApplied(format!(
                "already applied: candidate revision {} with predecessor {:?} is already on disk",
                cand_rev_id, cand_predecessor
            )));
        }
    }

    let disk_head = disk_carrier_opt
        .as_ref()
        .and_then(|c| c.head_event_id.as_deref());

    if disk_head != expected_head_event_id {
        return Ok(PublishOutcome::Refused(format!(
            "refused: moved head: expected head {:?} does not match disk head {:?}",
            expected_head_event_id, disk_head
        )));
    }

    save(project_root, candidate)?;
    Ok(PublishOutcome::Applied)
}

/// Fold a successful issue creation into the durable carrier.  Legacy
/// loop-state WIs deliberately remain unmigrated and are rendered fail-closed.
pub fn record_create(project_root: &Path, issue: &Issue) -> Result<()> {
    if !issue.issue_type.is_change() || is_legacy_loop_state(issue) {
        return Ok(());
    }
    let candidate = initial_lifecycle(issue);
    match publish_lifecycle_cas(project_root, None, &candidate)? {
        PublishOutcome::Applied | PublishOutcome::AlreadyApplied(_) => Ok(()),
        PublishOutcome::Refused(reason) => anyhow::bail!("{reason}"),
    }
}

/// Fold a successful WI body update.  A same-content update is deliberately
/// carrier-byte-stable; every semantic body change gets a new head and epoch.
pub fn record_update(project_root: &Path, before: &Issue, updated: &Issue) -> Result<()> {
    if !updated.issue_type.is_change() || is_legacy_loop_state(updated) {
        return Ok(());
    }
    let Some(lifecycle) = load(project_root, &updated.slug)? else {
        return Ok(());
    };
    let expected_head = lifecycle.head_event_id.clone();
    let result = fold_wi_update_from_issues(&lifecycle, updated, Some(before));
    if !result.accepted {
        return Ok(());
    }
    match publish_lifecycle_cas(project_root, expected_head.as_deref(), &result.lifecycle)? {
        PublishOutcome::Applied | PublishOutcome::AlreadyApplied(_) => Ok(()),
        PublishOutcome::Refused(reason) => anyhow::bail!("{reason}"),
    }
}

fn zero_head_projection(slug: &str, owner: OwnerVocabulary) -> serde_json::Value {
    serde_json::json!({
        "schema": SCHEMA,
        "wi_revision": serde_json::Value::Null,
        "ec_revision": serde_json::Value::Null,
        "td_revision": serde_json::Value::Null,
        "cb_revision": serde_json::Value::Null,
        "ledger": {"head_event_id": serde_json::Value::Null, "epoch": 0},
        "evidence": [],
        "invalidations": [],
        "iteration": 1,
        "next": {"command": format!("aw wi validate {slug}"), "owner": owner.as_str()},
        "terminal": false,
        "milestones": [],
        "drift": false,
        "remediation": [],
        "invalidated_kinds": [],
        "close_authorized": false,
        "authorize_close_event_id": serde_json::Value::Null,
    })
}

pub fn render_milestone(lifecycle: &ChangeLifecycle, event: &LifecycleEvent) -> serde_json::Value {
    let mut milestone_evidence = Vec::new();
    for binding in &lifecycle.evidence {
        if binding.bound_tuple.matches(&event.bound_tuple) {
            milestone_evidence.push(binding.clone());
        }
    }
    for invalidation in &lifecycle.invalidations {
        for binding in &invalidation.evicted_evidence {
            if binding.bound_tuple.matches(&event.bound_tuple) {
                milestone_evidence.push(binding.clone());
            }
        }
    }
    let next = if event.kind == LifecycleEventKind::CbCommit {
        serde_json::json!({
            "command": format!("aw wi show {}", lifecycle.slug),
            "owner": OwnerVocabulary::Cb.as_str(),
        })
    } else {
        serde_json::json!({
            "command": event.next_command.clone(),
            "owner": event.next_owner.as_str(),
        })
    };
    serde_json::json!({
        "event_id": event.event_id,
        "artifact": event.candidate_revision.public_json(),
        "evidence": milestone_evidence,
        "outcome": "accepted",
        "iteration": event.candidate_revision.iteration,
        "next": next,
    })
}

/// Decide, as a pure function of the committed lifecycle and an observed tracker
/// state, which milestones may be written to the tracker and whether the issue
/// may be closed.
pub fn decide_projection(
    lifecycle: &ChangeLifecycle,
    observation: &TrackerObservation,
) -> ProjectionDecision {
    let target_epoch = lifecycle.epoch;
    let target_head_event_id = lifecycle.head_event_id.clone();

    if let Some(active_wi_digest) = lifecycle
        .active_revisions
        .get(&ArtifactKind::Wi)
        .and_then(|r| r.as_ref())
        .map(|r| &r.digest)
    {
        let obs_wi_digest = observation.wi_digest(lifecycle);
        if obs_wi_digest != *active_wi_digest {
            let drift_remediation = vec![route_failure(
                FailureOwnership::WiDrift,
                &lifecycle.slug,
                &lifecycle.next.command,
            )];
            return ProjectionDecision {
                accepted: false,
                refusal_reason: Some(format!(
                    "Conflicting tracker WI body digest {:?} does not match active WI revision digest {:?}",
                    obs_wi_digest, active_wi_digest
                )),
                target_epoch,
                target_head_event_id,
                work: Vec::new(),
                complete: false,
                close_authorized: false,
                authorize_close_event_id: None,
                drift: true,
                remediation: drift_remediation,
                invalidated_kinds: transitive_invalidation_kinds(ArtifactKind::Wi),
            };
        }
    }

    if let Some(ref obs_head) = observation.head_event_id {
        let is_committed = lifecycle
            .events
            .iter()
            .any(|event| &event.event_id == obs_head);
        if !is_committed {
            return ProjectionDecision {
                accepted: false,
                refusal_reason: Some(format!(
                    "Observed tracker head {:?} was never committed in ledger",
                    obs_head
                )),
                target_epoch,
                target_head_event_id,
                work: Vec::new(),
                complete: false,
                close_authorized: false,
                authorize_close_event_id: None,
                drift: false,
                remediation: Vec::new(),
                invalidated_kinds: Vec::new(),
            };
        }
    }

    if let Some(obs_epoch) = observation.epoch {
        let expected_epoch = match observation.head_event_id.as_deref() {
            Some(obs_head) => lifecycle
                .events
                .iter()
                .position(|event| event.event_id == obs_head)
                .map(|idx| (idx + 1) as u64),
            None => Some(0),
        };
        if let Some(expected) = expected_epoch {
            if obs_epoch != expected {
                return ProjectionDecision {
                    accepted: false,
                    refusal_reason: Some(format!(
                        "Observed tracker epoch {} disagrees with ledger epoch {} for head {:?}",
                        obs_epoch, expected, observation.head_event_id
                    )),
                    target_epoch,
                    target_head_event_id,
                    work: Vec::new(),
                    complete: false,
                    close_authorized: false,
                    authorize_close_event_id: None,
                    drift: false,
                    remediation: Vec::new(),
                    invalidated_kinds: Vec::new(),
                };
            }
        }
    }

    let committed_event_ids: BTreeSet<&str> = lifecycle
        .events
        .iter()
        .map(|e| e.event_id.as_str())
        .collect();
    for present_id in &observation.present_event_ids {
        if !committed_event_ids.contains(present_id.as_str()) {
            return ProjectionDecision {
                accepted: false,
                refusal_reason: Some(format!(
                    "Observed milestone {:?} was never committed in ledger",
                    present_id
                )),
                target_epoch,
                target_head_event_id,
                work: Vec::new(),
                complete: false,
                close_authorized: false,
                authorize_close_event_id: None,
                drift: false,
                remediation: Vec::new(),
                invalidated_kinds: Vec::new(),
            };
        }
    }

    let mut work = Vec::new();
    for event in &lifecycle.events {
        if !observation.present_event_ids.contains(&event.event_id) {
            work.push(MilestoneWork {
                event_id: event.event_id.clone(),
                milestone: render_milestone(lifecycle, event),
            });
        }
    }

    let head_event = lifecycle.events.last();
    let is_terminal_commit = head_event.map_or(false, |evt| {
        evt.kind == LifecycleEventKind::CbCommit
            && lifecycle.head_event_id.as_deref() == Some(evt.event_id.as_str())
    });

    let complete = is_terminal_commit;
    let obs_closed = observation.is_closed();

    let (close_authorized, authorize_close_event_id, drift, remediation) = if is_terminal_commit {
        if obs_closed {
            (false, None, false, Vec::new())
        } else {
            (
                true,
                Some(head_event.unwrap().event_id.clone()),
                false,
                Vec::new(),
            )
        }
    } else {
        if obs_closed {
            let reopen_cmd = NextObligation {
                command: format!("aw wi update {} --state open", lifecycle.slug),
                owner: OwnerVocabulary::Wi,
            };
            (false, None, true, vec![reopen_cmd, lifecycle.next.clone()])
        } else {
            (false, None, false, Vec::new())
        }
    };

    ProjectionDecision {
        accepted: true,
        refusal_reason: None,
        target_epoch,
        target_head_event_id,
        work,
        complete,
        close_authorized,
        authorize_close_event_id,
        drift,
        remediation,
        invalidated_kinds: Vec::new(),
    }
}

fn render(lifecycle: &ChangeLifecycle, decision: &ProjectionDecision) -> serde_json::Value {
    let revision = |kind| match lifecycle
        .active_revisions
        .get(&kind)
        .and_then(|value| value.as_ref())
    {
        Some(value) => value.public_json(),
        None => serde_json::Value::Null,
    };
    let milestones = lifecycle
        .events
        .iter()
        .map(|event| render_milestone(lifecycle, event))
        .collect::<Vec<_>>();
    let remediation = decision
        .remediation
        .iter()
        .map(|ob| {
            serde_json::json!({
                "command": ob.command,
                "owner": ob.owner.as_str(),
            })
        })
        .collect::<Vec<_>>();
    let invalidated_kinds = decision
        .invalidated_kinds
        .iter()
        .map(|k| match k {
            ArtifactKind::Wi => "wi",
            ArtifactKind::Ec => "ec",
            ArtifactKind::Td => "td",
            ArtifactKind::Cb => "cb",
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema": SCHEMA,
        "wi_revision": revision(ArtifactKind::Wi),
        "ec_revision": revision(ArtifactKind::Ec),
        "td_revision": revision(ArtifactKind::Td),
        "cb_revision": revision(ArtifactKind::Cb),
        "ledger": {"head_event_id": lifecycle.head_event_id, "epoch": lifecycle.epoch},
        "evidence": lifecycle.evidence,
        "invalidations": lifecycle.invalidations,
        "iteration": lifecycle.iteration,
        "next": {"command": lifecycle.next.command, "owner": lifecycle.next.owner.as_str()},
        "terminal": lifecycle.terminal,
        "milestones": milestones,
        "drift": decision.drift,
        "remediation": remediation,
        "invalidated_kinds": invalidated_kinds,
        "close_authorized": decision.close_authorized,
        "authorize_close_event_id": decision.authorize_close_event_id,
        "accepted": decision.accepted,
        "refusal_reason": decision.refusal_reason,
    })
}

/// Read-only `aw wi show --json` projection.  It never initializes, repairs,
/// or rewrites a carrier: absent or legacy state remains migration-owned.
pub fn projection_for_issue(project_root: &Path, issue: &Issue) -> serde_json::Value {
    if !issue.issue_type.is_change() || is_legacy_loop_state(issue) {
        return zero_head_projection(&issue.slug, OwnerVocabulary::Migration);
    }
    match load(project_root, &issue.slug) {
        Ok(Some(lifecycle)) if valid_persisted_lifecycle(&lifecycle, &issue.slug) => {
            let observation = TrackerObservation::from_issue(issue);
            let decision = decide_projection(&lifecycle, &observation);
            render(&lifecycle, &decision)
        }
        Ok(Some(_)) | Ok(None) | Err(_) => zero_head_projection(&issue.slug, OwnerVocabulary::Wi),
    }
}

/// Sync leaf execution for `aw wi change <id>`.
///
/// Creates the WI-bound lifecycle when there is no durable carrier, or resumes
/// the existing one without advancing it, reporting drift rather than folding it away.
pub fn run_change_leaf(project_root: &Path, issue: &Issue) -> Result<serde_json::Value> {
    if load(project_root, &issue.slug)?.is_none() {
        record_create(project_root, issue)?;
    }
    Ok(projection_for_issue(project_root, issue))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiDimensionResult {
    pub dimension: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiValidationOutcome {
    pub passed: bool,
    pub dimensions: Vec<WiDimensionResult>,
    pub summary: String,
}

/// Sync leaf execution for `aw wi test <id>`.
///
/// Validates the WI-bound lifecycle's committed CanonicalWiSnapshot along R4's 6 dimensions,
/// appends the outcome as a wi-test EvidenceBinding without advancing the lifecycle,
/// and sets next.command to `aw wi review <slug>`.
pub fn run_test_leaf(
    project_root: &Path,
    issue: &Issue,
    all_issues: &[Issue],
) -> Result<serde_json::Value> {
    if load(project_root, &issue.slug)?.is_none() {
        record_create(project_root, issue)?;
    }
    let mut lifecycle = load(project_root, &issue.slug)?
        .context("failed to load causal lifecycle carrier after creation")?;

    let snapshot = lifecycle
        .wi_snapshot()
        .unwrap_or_else(|| CanonicalWiSnapshot::from_issue(issue));

    let bound_tuple = lifecycle.active_digest_tuple();
    let outcome = validate_canonical_wi_snapshot(&snapshot, all_issues);

    let binding = EvidenceBinding {
        verifier: "wi-test".to_string(),
        bound_tuple,
        passed: outcome.passed,
        summary: outcome.summary,
    };

    lifecycle.evidence.retain(|b| b.verifier != "wi-test");
    lifecycle.evidence.push(binding);

    lifecycle.next = derive_evidence_next_obligation(
        &lifecycle.slug,
        &lifecycle.evidence,
        &lifecycle.active_digest_tuple(),
        &lifecycle.next,
    );

    save(project_root, &lifecycle)?;

    Ok(projection_for_issue(project_root, issue))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiReviewPayload {
    #[serde(default)]
    pub reviewer_kind: Option<String>,
    #[serde(default)]
    pub decision: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
}

/// Sync leaf execution for `aw wi review <id> --evidence-file <path>`.
///
/// Reads a semantic review payload from `--evidence-file`, validates that a green `wi-test`
/// evidence binding exists on the active digest tuple, records the verdict as a `wi-review`
/// EvidenceBinding without advancing the lifecycle, and updates next.command to
/// `aw wi commit <slug>` for `accepted` or `aw wi change <slug>` for `needs_revision`.
pub fn run_review_leaf(
    project_root: &Path,
    issue: &Issue,
    evidence_file: &Path,
) -> Result<serde_json::Value> {
    if load(project_root, &issue.slug)?.is_none() {
        record_create(project_root, issue)?;
    }
    let mut lifecycle = load(project_root, &issue.slug)?
        .context("failed to load causal lifecycle carrier after creation")?;

    let active_tuple = lifecycle.active_digest_tuple();

    let test_binding = match lifecycle.evidence.iter().find(|b| b.verifier == "wi-test") {
        None => anyhow::bail!("stage `review` refused: missing `wi-test` evidence binding"),
        Some(b) => b,
    };

    if !test_binding.passed {
        anyhow::bail!("stage `review` refused: `wi-test` evidence binding is failing");
    }

    if !test_binding.bound_tuple.matches(&active_tuple) {
        anyhow::bail!("stage `review` refused: `wi-test` evidence binding is stale");
    }

    let content = std::fs::read_to_string(evidence_file).with_context(|| {
        format!(
            "failed to read evidence payload from {}",
            evidence_file.display()
        )
    })?;

    let payload: WiReviewPayload = serde_json::from_str(&content).with_context(|| {
        format!(
            "failed to parse review payload from {}",
            evidence_file.display()
        )
    })?;

    let _reviewer_kind = match payload.reviewer_kind.as_deref() {
        Some(kind) if kind == "human" || kind == "agent" => kind,
        Some(other) => {
            anyhow::bail!("unknown reviewer_kind `{other}`: expected `human` or `agent`")
        }
        None => anyhow::bail!("review payload missing required field `reviewer_kind`"),
    };

    let decision = match payload.decision.as_deref() {
        Some(dec) if dec == "accepted" || dec == "needs_revision" => dec,
        Some(other) => {
            anyhow::bail!("unknown decision `{other}`: expected `accepted` or `needs_revision`")
        }
        None => anyhow::bail!("review payload missing required field `decision`"),
    };

    let passed = decision == "accepted";
    let summary = payload.summary.unwrap_or_default();

    let binding = EvidenceBinding {
        verifier: "wi-review".to_string(),
        bound_tuple: test_binding.bound_tuple.clone(),
        passed,
        summary,
    };

    lifecycle.evidence.retain(|b| b.verifier != "wi-review");
    lifecycle.evidence.push(binding);

    lifecycle.next = derive_evidence_next_obligation(
        &lifecycle.slug,
        &lifecycle.evidence,
        &lifecycle.active_digest_tuple(),
        &lifecycle.next,
    );

    save(project_root, &lifecycle)?;

    Ok(projection_for_issue(project_root, issue))
}

/// Validate a CanonicalWiSnapshot along R4's 6 dimensions.
pub fn validate_canonical_wi_snapshot(
    snapshot: &CanonicalWiSnapshot,
    all_issues: &[Issue],
) -> WiValidationOutcome {
    let temp_issue = Issue {
        issue_type: match snapshot.issue_type.as_str() {
            "epic" => crate::issues::IssueType::Epic,
            "spike" => crate::issues::IssueType::Spike,
            "report" => crate::issues::IssueType::Report,
            _ => crate::issues::IssueType::Change,
        },
        title: snapshot.title.clone(),
        state: crate::issues::IssueState::Open,
        id: Some(snapshot.identity.clone()),
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: {
            let mut labels = vec![format!("app:{}", snapshot.project)];
            for ep in &snapshot.epic_parents {
                labels.push(format!("epic:{ep}"));
            }
            for dep in &snapshot.dependencies {
                labels.push(format!("depends-on:{dep}"));
            }
            labels
        },
        created_at: None,
        updated_at: None,
        slug: snapshot.identity.clone(),
        body: snapshot.body.clone(),
        related: Vec::new(),
        implements: Vec::new(),
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
        validation_errors: Vec::new(),
        review_count: None,
        flagged_sections: None,
        fill_retry_count: None,
        ship_status: None,
        ship_commit: None,
        regen_verified_at: None,
    };

    let mut dimensions = Vec::new();

    // 1. Section Structure
    dimensions.push(validate_dim_section_structure(snapshot, &temp_issue));

    // 2. Boundedness
    dimensions.push(validate_dim_boundedness(snapshot, &temp_issue));

    // 3. Requirement/AC/Gate Coverage
    dimensions.push(validate_dim_coverage(snapshot, &temp_issue));

    // 4. References
    dimensions.push(validate_dim_references(snapshot, &temp_issue, all_issues));

    // 5. Ownership
    dimensions.push(validate_dim_ownership(snapshot, &temp_issue, all_issues));

    // 6. Dependency Graph
    dimensions.push(validate_dim_dependency_graph(snapshot, &temp_issue));

    let passed = dimensions.iter().all(|d| d.passed);
    let summary = if passed {
        "pass: all 6 structural dimensions valid".to_string()
    } else {
        let fails: Vec<String> = dimensions
            .iter()
            .filter(|d| !d.passed)
            .map(|d| format!("[{}]: {}", d.dimension, d.detail))
            .collect();
        format!("fail: {}", fails.join("; "))
    };

    WiValidationOutcome {
        passed,
        dimensions,
        summary,
    }
}

fn validate_dim_section_structure(
    snapshot: &CanonicalWiSnapshot,
    _temp_issue: &Issue,
) -> WiDimensionResult {
    let dimension = "section_structure".to_string();
    if let Err(err) =
        crate::services::issue_parser::validate_structured_issue(&snapshot.body, IssueState::Open)
    {
        return WiDimensionResult {
            dimension,
            passed: false,
            detail: err.to_string(),
        };
    }
    let Some(structured) = crate::services::issue_parser::parse_structured_issue(&snapshot.body)
    else {
        return WiDimensionResult {
            dimension,
            passed: false,
            detail: "body missing required section headers".to_string(),
        };
    };
    if structured.requirements.is_empty() {
        return WiDimensionResult {
            dimension,
            passed: false,
            detail: "## Requirements section contains no requirement items".to_string(),
        };
    }
    WiDimensionResult {
        dimension,
        passed: true,
        detail: "section structure valid".to_string(),
    }
}

fn validate_dim_boundedness(
    snapshot: &CanonicalWiSnapshot,
    temp_issue: &Issue,
) -> WiDimensionResult {
    let dimension = "boundedness".to_string();
    if crate::issues::planner::looks_too_large_for_atomic_wi(temp_issue) {
        return WiDimensionResult {
            dimension,
            passed: false,
            detail: format!(
                "work item `{}` looks too large for an atomic WI",
                snapshot.identity
            ),
        };
    }
    WiDimensionResult {
        dimension,
        passed: true,
        detail: "work item is bounded".to_string(),
    }
}

fn validate_dim_coverage(snapshot: &CanonicalWiSnapshot, temp_issue: &Issue) -> WiDimensionResult {
    let dimension = "coverage".to_string();
    let Some(structured) = crate::services::issue_parser::parse_structured_issue(&snapshot.body)
    else {
        return WiDimensionResult {
            dimension,
            passed: false,
            detail: "cannot parse structured issue body".to_string(),
        };
    };

    let verif_errors =
        crate::issues::planner::validate_requirement_verification_inventory(temp_issue);
    let mut missing = Vec::new();

    let ac_text = snapshot
        .body
        .split("## Acceptance Criteria")
        .nth(1)
        .unwrap_or("")
        .split("## ")
        .next()
        .unwrap_or("");

    for req in &structured.requirements {
        let rid = req.id.trim();
        if rid.is_empty() {
            continue;
        }

        let ac_covers = ac_text.contains(rid);
        let verif_covers = !verif_errors
            .iter()
            .any(|e| e.contains(&format!("must map {rid} to")));

        if !ac_covers || !verif_covers {
            missing.push(rid.to_string());
        }
    }

    if !missing.is_empty() {
        missing.sort();
        missing.dedup();
        return WiDimensionResult {
            dimension,
            passed: false,
            detail: format!("uncovered requirement id(s): {}", missing.join(", ")),
        };
    }

    WiDimensionResult {
        dimension,
        passed: true,
        detail: "requirement/AC/gate coverage complete".to_string(),
    }
}

fn validate_dim_references(
    snapshot: &CanonicalWiSnapshot,
    temp_issue: &Issue,
    all_issues: &[Issue],
) -> WiDimensionResult {
    let dimension = "references".to_string();
    let mut deps = snapshot.dependencies.clone();
    for d in crate::issues::graph::body_dependency_references(temp_issue) {
        if !deps.contains(&d) {
            deps.push(d);
        }
    }

    if !all_issues.is_empty() {
        let known_keys: BTreeSet<String> = all_issues
            .iter()
            .flat_map(|i| {
                let mut keys = vec![i.slug.clone()];
                if let Some(id) = &i.id {
                    keys.push(id.clone());
                }
                keys
            })
            .collect();

        for dep in &deps {
            let norm = dep.trim_start_matches('#').trim();
            if !norm.is_empty() && !known_keys.contains(norm) {
                return WiDimensionResult {
                    dimension,
                    passed: false,
                    detail: format!("referenced dependency work item `{dep}` does not exist"),
                };
            }
        }
    }

    WiDimensionResult {
        dimension,
        passed: true,
        detail: "all references exist".to_string(),
    }
}

fn validate_dim_ownership(
    snapshot: &CanonicalWiSnapshot,
    _temp_issue: &Issue,
    all_issues: &[Issue],
) -> WiDimensionResult {
    let dimension = "ownership".to_string();
    let parents = &snapshot.epic_parents;

    if parents.len() > 1 {
        return WiDimensionResult {
            dimension,
            passed: false,
            detail: format!(
                "work item `{}` has multiple declared parent epics: {}",
                snapshot.identity,
                parents.join(", ")
            ),
        };
    }

    if !parents.is_empty() && !all_issues.is_empty() {
        let parent_id = &parents[0];
        let parent_issue = all_issues
            .iter()
            .find(|i| i.slug == *parent_id || i.id.as_deref() == Some(parent_id.as_str()));
        match parent_issue {
            None => {
                return WiDimensionResult {
                    dimension,
                    passed: false,
                    detail: format!("parent epic `{parent_id}` does not exist"),
                };
            }
            Some(p) if p.issue_type != crate::issues::IssueType::Epic => {
                return WiDimensionResult {
                    dimension,
                    passed: false,
                    detail: format!("parent `{parent_id}` is not an epic"),
                };
            }
            Some(_) => {}
        }
    }

    WiDimensionResult {
        dimension,
        passed: true,
        detail: if parents.is_empty() {
            "ownership unconstrained".to_string()
        } else {
            format!("owned by epic `{}`", parents[0])
        },
    }
}

fn validate_dim_dependency_graph(
    snapshot: &CanonicalWiSnapshot,
    temp_issue: &Issue,
) -> WiDimensionResult {
    let dimension = "dependency_graph".to_string();
    let dep_errors =
        crate::issues::planner::validate_requirement_verification_inventory(temp_issue);
    let graph_errors: Vec<_> = dep_errors
        .into_iter()
        .filter(|e| {
            e.contains("depends on unknown")
                || e.contains("cannot depend on itself")
                || e.contains("must be acyclic")
        })
        .collect();

    if !graph_errors.is_empty() {
        return WiDimensionResult {
            dimension,
            passed: false,
            detail: graph_errors.join("; "),
        };
    }

    WiDimensionResult {
        dimension,
        passed: true,
        detail: "dependency graph valid and acyclic".to_string(),
    }
}

#[cfg(test)]
fn candidate_tuple(lifecycle: &ChangeLifecycle, candidate: &ArtifactRevision) -> ActiveDigestTuple {
    let mut tuple = lifecycle.active_digest_tuple();
    match candidate.kind {
        ArtifactKind::Wi => tuple.wi_digest = Some(candidate.digest.clone()),
        ArtifactKind::Ec => tuple.ec_digest = Some(candidate.digest.clone()),
        ArtifactKind::Td => tuple.td_digest = Some(candidate.digest.clone()),
        ArtifactKind::Cb => tuple.cb_digest = Some(candidate.digest.clone()),
    }
    tuple
}

#[cfg(test)]
fn reduce_stage(
    lifecycle: &ChangeLifecycle,
    kind: ArtifactKind,
    event_kind: LifecycleEventKind,
    source: &str,
    command: &str,
    owner: OwnerVocabulary,
) -> ChangeLifecycle {
    let candidate = artifact_revision(
        kind,
        canonical_digest(source),
        expected_parent_set(lifecycle, kind).expect("all stage parents are active"),
        lifecycle.epoch + 1,
    );
    let result = reduce_event(
        lifecycle,
        LifecycleEvent {
            event_id: event_id(lifecycle),
            predecessor_id: lifecycle.head_event_id.clone(),
            kind: event_kind,
            bound_tuple: candidate_tuple(lifecycle, &candidate),
            candidate_revision: candidate,
            next_command: command.to_string(),
            next_owner: owner,
            wi_snapshot: None,
        },
    );
    assert!(result.accepted, "{:?}", result.rejection_reason);
    result.lifecycle
}

/// Persists a valid 5-event terminal lifecycle carrier for `slug` under `project_root`.
///
/// Builds `wi_create`, `ec_change`, `td_change`, `cb_change`, and `cb_commit` events
/// with green 4D evidence (`cb_test`, `cb_review`, `td_reconcile`, `ec_verify_cb`)
/// through the reducer and persists the resulting carrier via `save`.
#[cfg(test)]
pub(crate) fn record_terminal_lifecycle(project_root: &std::path::Path, slug: &str) {
    let body = if slug == "change-row3" {
        "body-1"
    } else {
        "wi-v1"
    };
    let created = fold_wi_create(slug, body, "agentic-workflow");
    let ec = reduce_stage(
        &created,
        ArtifactKind::Ec,
        LifecycleEventKind::EcChange,
        "ec-v1",
        "aw td check",
        OwnerVocabulary::Td,
    );
    let td = reduce_stage(
        &ec,
        ArtifactKind::Td,
        LifecycleEventKind::TdChange,
        "td-v1",
        "aw cb check",
        OwnerVocabulary::Cb,
    );
    let mut cb = reduce_stage(
        &td,
        ArtifactKind::Cb,
        LifecycleEventKind::CbChange,
        "cb-v1",
        "aw cb check",
        OwnerVocabulary::Cb,
    );
    let tuple = cb.active_digest_tuple();
    cb.evidence = ["cb_test", "cb_review", "td_reconcile", "ec_verify_cb"]
        .into_iter()
        .map(|v| EvidenceBinding {
            verifier: v.to_string(),
            bound_tuple: tuple.clone(),
            passed: true,
            summary: "pass".to_string(),
        })
        .collect();
    let active_cb = cb.active_revisions[&ArtifactKind::Cb].clone().unwrap();
    let commit_evt = LifecycleEvent {
        event_id: event_id(&cb),
        predecessor_id: cb.head_event_id.clone(),
        kind: LifecycleEventKind::CbCommit,
        candidate_revision: active_cb,
        bound_tuple: tuple,
        next_command: format!("aw wi show {slug}"),
        next_owner: OwnerVocabulary::Cb,
        wi_snapshot: None,
    };
    let committed = reduce_event(&cb, commit_evt);
    assert!(committed.accepted);
    save(project_root, &committed.lifecycle).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::issues::{ensure_change_id, ensure_change_issue};
    use crate::issues::{IssueState, IssueType};

    fn canonical_wi_digest(body: &str) -> String {
        CanonicalWiSnapshot::from_parts(
            "causal",
            "",
            "change",
            "agentic-workflow",
            Vec::new(),
            Vec::new(),
            body,
        )
        .digest()
    }

    fn change(body: &str) -> Issue {
        Issue {
            issue_type: IssueType::Change,
            title: "Causal lifecycle".to_string(),
            state: IssueState::Open,
            id: Some("causal".to_string()),
            github_id: None,
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec!["app:agentic-workflow".to_string()],
            created_at: None,
            updated_at: None,
            slug: "causal".to_string(),
            body: body.to_string(),
            related: Vec::new(),
            implements: Vec::new(),
            phase: Some("created".to_string()),
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
            validation_errors: Vec::new(),
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        }
    }

    fn complete_lifecycle() -> ChangeLifecycle {
        let created = fold_wi_create("causal", "wi-v1", "agentic-workflow");
        let ec = reduce_stage(
            &created,
            ArtifactKind::Ec,
            LifecycleEventKind::EcChange,
            "ec-v1",
            "aw td check",
            OwnerVocabulary::Td,
        );
        let td = reduce_stage(
            &ec,
            ArtifactKind::Td,
            LifecycleEventKind::TdChange,
            "td-v1",
            "aw cb check",
            OwnerVocabulary::Cb,
        );
        reduce_stage(
            &td,
            ArtifactKind::Cb,
            LifecycleEventKind::CbChange,
            "cb-v1",
            "aw cb check",
            OwnerVocabulary::Cb,
        )
    }

    #[test]
    fn durable_update_advances_epoch_and_keeps_predecessor() {
        let root = tempfile::tempdir().unwrap();
        let before = change("before");
        record_create(root.path(), &before).unwrap();
        let original = std::fs::read(carrier_path(root.path(), &before.slug)).unwrap();
        record_update(root.path(), &before, &before).unwrap();
        assert_eq!(
            std::fs::read(carrier_path(root.path(), &before.slug)).unwrap(),
            original
        );

        let after = change("after");
        record_update(root.path(), &before, &after).unwrap();
        let lifecycle = load(root.path(), &after.slug).unwrap().unwrap();
        assert_eq!(lifecycle.epoch, 2);
        assert_eq!(
            lifecycle.events[1].predecessor_id.as_deref(),
            Some("evt-001")
        );
    }

    #[test]
    fn legacy_projection_is_fail_closed_without_carrier() {
        let root = tempfile::tempdir().unwrap();
        let legacy = change("<!-- aw:loop-state\nversion: 1\n-->");
        record_create(root.path(), &legacy).unwrap();
        assert!(!carrier_path(root.path(), &legacy.slug).exists());
        assert_eq!(
            projection_for_issue(root.path(), &legacy)["ledger"]["epoch"],
            0
        );
    }

    #[test]
    fn malformed_or_conflicting_carrier_is_fail_closed_to_wi() {
        let root = tempfile::tempdir().unwrap();
        let issue = change("before");
        record_create(root.path(), &issue).unwrap();
        let carrier = carrier_path(root.path(), &issue.slug);
        let mut payload: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&carrier).unwrap()).unwrap();
        payload["slug"] = serde_json::json!("wrong-slug");
        std::fs::write(&carrier, serde_json::to_vec(&payload).unwrap()).unwrap();
        let projection = projection_for_issue(root.path(), &issue);
        assert_eq!(projection["next"]["owner"], "wi");
        assert!(projection["wi_revision"].is_null());

        payload["slug"] = serde_json::json!(issue.slug);
        payload["events"][0]["kind"] = serde_json::json!("unknown");
        std::fs::write(&carrier, serde_json::to_vec(&payload).unwrap()).unwrap();
        let projection = projection_for_issue(root.path(), &issue);
        assert_eq!(projection["next"]["owner"], "wi");
        assert!(projection["wi_revision"].is_null());
    }

    #[test]
    fn persisted_multistage_carrier_with_td_event_vocabulary_hydrates() {
        let root = tempfile::tempdir().unwrap();
        let issue = change("before");
        let initial = initial_lifecycle(&issue);
        let lifecycle = reduce_stage(
            &initial,
            ArtifactKind::Ec,
            LifecycleEventKind::EcChange,
            "ec-v1",
            "aw td create causal --project agentic-workflow",
            OwnerVocabulary::Td,
        );
        save(root.path(), &lifecycle).unwrap();

        let projection = projection_for_issue(root.path(), &issue);
        let ec_revision = lifecycle.active_revisions[&ArtifactKind::Ec]
            .as_ref()
            .unwrap();
        assert_eq!(projection["ledger"]["head_event_id"], "evt-002");
        assert_eq!(projection["ec_revision"]["id"], ec_revision.id);
        assert_eq!(projection["next"]["owner"], "td");
    }

    #[test]
    fn hydration_rejects_carrier_with_missing_invalidation_record() {
        let root = tempfile::tempdir().unwrap();
        let issue = change("before");
        let mut lifecycle = initial_lifecycle(&issue);
        let ec_digest = canonical_digest("ec-v1");
        let ec_revision = artifact_revision(
            ArtifactKind::Ec,
            ec_digest,
            expected_parent_set(&lifecycle, ArtifactKind::Ec).unwrap(),
            1,
        );
        lifecycle.events.push(LifecycleEvent {
            event_id: "evt-002".to_string(),
            predecessor_id: Some("evt-001".to_string()),
            kind: LifecycleEventKind::EcChange,
            candidate_revision: ec_revision.clone(),
            bound_tuple: ActiveDigestTuple {
                wi_digest: lifecycle.active_digest_tuple().wi_digest,
                ec_digest: Some(ec_revision.digest.clone()),
                ..ActiveDigestTuple::default()
            },
            next_command: "aw td create causal --project agentic-workflow".to_string(),
            next_owner: OwnerVocabulary::Td,
            wi_snapshot: None,
        });
        lifecycle.epoch = 2;
        lifecycle.head_event_id = Some("evt-002".to_string());
        lifecycle
            .active_revisions
            .insert(ArtifactKind::Ec, Some(ec_revision.clone()));
        lifecycle.next = NextObligation {
            command: "aw td create causal --project agentic-workflow".to_string(),
            owner: OwnerVocabulary::Td,
        };
        save(root.path(), &lifecycle).unwrap();

        let projection = projection_for_issue(root.path(), &issue);
        assert!(projection["wi_revision"].is_null());
        assert_eq!(projection["next"]["owner"], "wi");
    }

    #[test]
    fn hydration_rejects_stage_revision_with_missing_causal_parent() {
        let root = tempfile::tempdir().unwrap();
        let issue = change("before");
        let lifecycle = initial_lifecycle(&issue);
        let invalid_ec =
            artifact_revision(ArtifactKind::Ec, canonical_digest("ec-v1"), Vec::new(), 1);
        let mut persisted = lifecycle.clone();
        persisted.epoch = 2;
        persisted.head_event_id = Some("evt-002".to_string());
        persisted.events.push(LifecycleEvent {
            event_id: "evt-002".to_string(),
            predecessor_id: Some("evt-001".to_string()),
            kind: LifecycleEventKind::EcChange,
            bound_tuple: candidate_tuple(&lifecycle, &invalid_ec),
            candidate_revision: invalid_ec.clone(),
            next_command: "aw td check".to_string(),
            next_owner: OwnerVocabulary::Td,
            wi_snapshot: None,
        });
        persisted
            .active_revisions
            .insert(ArtifactKind::Ec, Some(invalid_ec));
        persisted.next = NextObligation {
            command: "aw td check".to_string(),
            owner: OwnerVocabulary::Td,
        };
        save(root.path(), &persisted).unwrap();
        let projection = projection_for_issue(root.path(), &issue);
        assert!(projection["wi_revision"].is_null());
        assert_eq!(projection["next"]["owner"], "wi");
    }

    #[test]
    fn hydration_rejects_tampered_empty_carrier() {
        let root = tempfile::tempdir().unwrap();
        let issue = change("before");
        let mut empty = replay_seed(&issue.slug);
        empty.terminal = true;
        empty.next = NextObligation {
            command: "aw wi show causal".to_string(),
            owner: OwnerVocabulary::Cb,
        };
        save(root.path(), &empty).unwrap();
        assert!(!valid_persisted_lifecycle(&empty, &issue.slug));
        let projection = projection_for_issue(root.path(), &issue);
        assert!(!projection["terminal"].as_bool().unwrap());
        assert_eq!(projection["next"]["command"], "aw wi validate causal");
        assert_eq!(projection["next"]["owner"], "wi");
    }

    #[test]
    fn hydration_rejects_tampered_sorted_eviction_witnesses() {
        let root = tempfile::tempdir().unwrap();
        let issue = change("wi-v2");
        let mut full = complete_lifecycle();
        full.evidence.push(EvidenceBinding {
            verifier: "cb_test".to_string(),
            bound_tuple: full.active_digest_tuple(),
            passed: true,
            summary: "current CB test".to_string(),
        });
        let updated = fold_wi_update(&full, "wi-v2", Some("wi-v1"));
        assert!(updated.accepted, "{:?}", updated.rejection_reason);
        save(root.path(), &updated.lifecycle).unwrap();
        let carrier = carrier_path(root.path(), &issue.slug);
        let mut payload: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&carrier).unwrap()).unwrap();
        let invalidations = payload["invalidations"].as_array_mut().unwrap();
        invalidations.last_mut().unwrap()["evicted_evidence_verifiers"] =
            serde_json::json!(["tampered"]);
        std::fs::write(&carrier, serde_json::to_vec(&payload).unwrap()).unwrap();
        let projection = projection_for_issue(root.path(), &issue);
        assert!(projection["wi_revision"].is_null());
        assert_eq!(projection["next"]["owner"], "wi");
    }

    #[test]
    fn revisioned_change_wi_ledger() {
        let root = tempfile::tempdir().unwrap();
        let mut lifecycle = complete_lifecycle();
        let tuple = lifecycle.active_digest_tuple();
        lifecycle.evidence.push(EvidenceBinding {
            verifier: "cb_test".to_string(),
            bound_tuple: tuple,
            passed: true,
            summary: "bound evidence".to_string(),
        });
        save(root.path(), &lifecycle).unwrap();
        let reloaded = load(root.path(), "causal").unwrap().unwrap();
        assert_eq!(reloaded, lifecycle);
        assert!(valid_persisted_lifecycle(&reloaded, "causal"));
        assert_eq!(reloaded.events.len(), 4);
        assert_eq!(reloaded.head_event_id.as_deref(), Some("evt-004"));
        assert_eq!(
            reloaded.active_revisions[&ArtifactKind::Cb]
                .as_ref()
                .unwrap()
                .parents
                .len(),
            3
        );
        assert_eq!(
            route_failure(FailureOwnership::Infrastructure, "causal", "aw cb check").command,
            "aw cb check"
        );
        assert_eq!(
            route_failure(FailureOwnership::Contract, "causal", "ignored").owner,
            OwnerVocabulary::Ec
        );
    }

    #[test]
    fn route_failure_all_variants_are_chain_valid_and_slug_sensitive() {
        use crate::cli::chain::validate_aw_command_string;

        let variants = [
            FailureOwnership::WiDrift,
            FailureOwnership::Contract,
            FailureOwnership::Design,
            FailureOwnership::Implementation,
            FailureOwnership::Infrastructure,
        ];

        let slug1 = "slug-alpha";
        let slug2 = "slug-beta";
        let current_cmd = "aw cb check slug-alpha";

        for variant in variants {
            let obl1 = route_failure(variant, slug1, current_cmd);
            let obl2 = route_failure(variant, slug2, current_cmd);

            assert!(
                validate_aw_command_string(&obl1.command).is_ok(),
                "route_failure({variant:?}, {slug1}) produced chain-invalid command `{}`",
                obl1.command
            );
            assert!(
                validate_aw_command_string(&obl2.command).is_ok(),
                "route_failure({variant:?}, {slug2}) produced chain-invalid command `{}`",
                obl2.command
            );
        }

        let design1 = route_failure(FailureOwnership::Design, slug1, current_cmd);
        let design2 = route_failure(FailureOwnership::Design, slug2, current_cmd);
        assert_eq!(design1.command, format!("aw td check --wi {slug1}"));
        assert_eq!(design2.command, format!("aw td check --wi {slug2}"));
        assert_ne!(design1.command, design2.command);

        let impl1 = route_failure(FailureOwnership::Implementation, slug1, current_cmd);
        let impl2 = route_failure(FailureOwnership::Implementation, slug2, current_cmd);
        assert_eq!(impl1.command, format!("aw cb check {slug1}"));
        assert_eq!(impl2.command, format!("aw cb check {slug2}"));
        assert_ne!(impl1.command, impl2.command);
    }

    #[test]
    fn revisioned_change_wi_parent_rebind() {
        let created = fold_wi_create("causal", "wi-v1", "agentic-workflow");
        let with_ec = reduce_stage(
            &created,
            ArtifactKind::Ec,
            LifecycleEventKind::EcChange,
            "ec-v1",
            "aw ec check",
            OwnerVocabulary::Ec,
        );
        let original_ec = with_ec.active_revisions[&ArtifactKind::Ec]
            .clone()
            .expect("active EC");
        let updated = fold_wi_update(&with_ec, "wi-v2", Some("wi-v1"));
        assert!(updated.accepted);
        let after_wi = updated.lifecycle;
        assert!(after_wi.active_revisions[&ArtifactKind::Ec].is_none());

        let rebound = artifact_revision(
            ArtifactKind::Ec,
            original_ec.digest.clone(),
            expected_parent_set(&after_wi, ArtifactKind::Ec).unwrap(),
            original_ec.iteration + 1,
        );
        assert_ne!(rebound.id, original_ec.id);
        let result = reduce_event(
            &after_wi,
            LifecycleEvent {
                event_id: event_id(&after_wi),
                predecessor_id: after_wi.head_event_id.clone(),
                kind: LifecycleEventKind::Rebind,
                bound_tuple: candidate_tuple(&after_wi, &rebound),
                candidate_revision: rebound.clone(),
                next_command: "aw ec check".to_string(),
                next_owner: OwnerVocabulary::Ec,
                wi_snapshot: None,
            },
        );
        assert!(result.accepted, "{:?}", result.rejection_reason);
        assert_eq!(
            result.lifecycle.active_revisions[&ArtifactKind::Ec]
                .as_ref()
                .map(|revision| &revision.id),
            Some(&rebound.id)
        );
    }

    #[test]
    fn revisioned_change_wi_invalidation() {
        let mut full = complete_lifecycle();
        let tuple = full.active_digest_tuple();
        full.evidence = vec![
            EvidenceBinding {
                verifier: "cb_test".to_string(),
                bound_tuple: tuple.clone(),
                passed: true,
                summary: "pass".to_string(),
            },
            EvidenceBinding {
                verifier: "cb_test".to_string(),
                bound_tuple: tuple,
                passed: true,
                summary: "duplicate verifier is deduplicated in the record".to_string(),
            },
        ];
        let old_ids = [ArtifactKind::Ec, ArtifactKind::Td, ArtifactKind::Cb]
            .into_iter()
            .map(|kind| full.active_revisions[&kind].as_ref().unwrap().id.clone())
            .collect::<Vec<_>>();
        let result = fold_wi_update(&full, "wi-v2", Some("wi-v1"));
        assert!(result.accepted, "{:?}", result.rejection_reason);
        for kind in [ArtifactKind::Ec, ArtifactKind::Td, ArtifactKind::Cb] {
            assert!(result.lifecycle.active_revisions[&kind].is_none());
        }
        assert!(result.lifecycle.evidence.is_empty());
        let invalidation = result.lifecycle.invalidations.last().unwrap();
        assert_eq!(
            invalidation.invalidated_kinds,
            vec![ArtifactKind::Ec, ArtifactKind::Td, ArtifactKind::Cb]
        );
        assert_eq!(invalidation.invalidated_revision_ids, old_ids);
        assert_eq!(invalidation.evicted_evidence_verifiers, vec!["cb_test"]);
    }

    #[test]
    fn revisioned_change_wi_reducer() {
        let created = fold_wi_create("causal", "wi-v1", "agentic-workflow");
        let stale_ec = artifact_revision(
            ArtifactKind::Ec,
            canonical_digest("ec-v1"),
            expected_parent_set(&created, ArtifactKind::Ec).unwrap(),
            1,
        );
        let stale = reduce_event(
            &created,
            LifecycleEvent {
                event_id: "evt-002".to_string(),
                predecessor_id: Some("evt-stale".to_string()),
                kind: LifecycleEventKind::EcChange,
                bound_tuple: candidate_tuple(&created, &stale_ec),
                candidate_revision: stale_ec,
                next_command: "aw ec check".to_string(),
                next_owner: OwnerVocabulary::Ec,
                wi_snapshot: None,
            },
        );
        assert!(!stale.accepted);
        assert_eq!(stale.lifecycle.next.command, "aw wi change causal");

        let noop = fold_wi_update(&created, "wi-v1", None);
        assert!(!noop.accepted);
        assert_eq!(noop.lifecycle, created);

        let mut full = complete_lifecycle();
        let tuple = full.active_digest_tuple();
        full.evidence = ["cb_test", "cb_review", "td_reconcile", "ec_verify_cb"]
            .into_iter()
            .map(|verifier| EvidenceBinding {
                verifier: verifier.to_string(),
                bound_tuple: tuple.clone(),
                passed: true,
                summary: "pass".to_string(),
            })
            .collect();
        let active_cb = full.active_revisions[&ArtifactKind::Cb]
            .clone()
            .expect("active CB");
        let commit = LifecycleEvent {
            event_id: event_id(&full),
            predecessor_id: full.head_event_id.clone(),
            kind: LifecycleEventKind::CbCommit,
            candidate_revision: active_cb.clone(),
            bound_tuple: tuple,
            next_command: "ignored: reducer emits canonical show".to_string(),
            next_owner: OwnerVocabulary::Wi,
            wi_snapshot: None,
        };
        let committed = reduce_event(&full, commit.clone());
        assert!(committed.accepted, "{:?}", committed.rejection_reason);
        assert!(committed.lifecycle.terminal);
        assert_eq!(committed.lifecycle.next.command, "aw wi show causal");
        assert_eq!(committed.lifecycle.next.owner, OwnerVocabulary::Cb);

        let non_cb_commit = reduce_event(
            &full,
            LifecycleEvent {
                event_id: event_id(&full),
                predecessor_id: full.head_event_id.clone(),
                candidate_revision: full.active_revisions[&ArtifactKind::Ec].clone().unwrap(),
                bound_tuple: full.active_digest_tuple(),
                ..commit.clone()
            },
        );
        assert!(!non_cb_commit.accepted);
        assert!(!non_cb_commit.lifecycle.terminal);
        assert_eq!(non_cb_commit.lifecycle.next.command, "aw cb check causal");

        let no_evidence = complete_lifecycle();
        let rejected = reduce_event(
            &no_evidence,
            LifecycleEvent {
                event_id: event_id(&no_evidence),
                predecessor_id: no_evidence.head_event_id.clone(),
                candidate_revision: no_evidence.active_revisions[&ArtifactKind::Cb]
                    .clone()
                    .unwrap(),
                bound_tuple: no_evidence.active_digest_tuple(),
                ..commit
            },
        );
        assert!(!rejected.accepted);
        assert_eq!(rejected.lifecycle.next.command, "aw ec verify --stage cb");
    }

    #[test]
    fn revisioned_change_wi_hydration() {
        let root = tempfile::tempdir().unwrap();
        let issue = change("wi-v1");
        let lifecycle = complete_lifecycle();
        save(root.path(), &lifecycle).unwrap();
        let carrier = carrier_path(root.path(), &issue.slug);
        let before = std::fs::read(&carrier).unwrap();
        let first = projection_for_issue(root.path(), &issue);
        let second = projection_for_issue(root.path(), &issue);
        assert_eq!(before, std::fs::read(&carrier).unwrap());
        assert_eq!(first, second);
        assert_eq!(first["ledger"]["epoch"], 4);
        assert_eq!(first["cb_revision"]["digest"], canonical_digest("cb-v1"));

        let mut payload: serde_json::Value = serde_json::from_slice(&before).unwrap();
        payload["events"][2]["predecessor_id"] = serde_json::json!("evt-conflict");
        std::fs::write(&carrier, serde_json::to_vec(&payload).unwrap()).unwrap();
        let fail_closed = projection_for_issue(root.path(), &issue);
        assert!(fail_closed["wi_revision"].is_null());
        assert_eq!(fail_closed["next"]["owner"], "wi");
    }

    #[test]
    fn revisioned_change_wi_ec_draft_command_round_trips_live_cli_parser() {
        let command = crate::cli::run::ec_draft_command("agentic-workflow", "causal");
        crate::cli::chain::validate_aw_command_string(&command).unwrap();
    }

    #[test]
    fn library_labels_route_the_initial_ec_command_to_the_library_project() {
        let mut issue = change("library");
        issue.labels = vec!["lib:service-auth".to_string()];
        let lifecycle = initial_lifecycle(&issue);
        assert_eq!(
            lifecycle.next.command,
            "aw ec draft causal --project service-auth --wi causal"
        );
    }

    #[test]
    fn wi_contract_projection_digest_boundary() {
        let baseline_body = r#"## Problem

This is the problem description for WI 3348.

<!-- aw:planning-transaction:sha256:3d830d401234567890abcdef1234567890abcdef1234567890abcdef12345678 -->

<!-- aw:loop-state
version: 1
issue_id: '3348'
goal: ec-first:3348
verifier: ec
iterations: []
last_result: none
next_action: aw ec verify --project agentic-workflow --required-only --stage td --wi 3348
status: iterating
tried: []
updated_at: 2026-08-04T08:27:44.814163+00:00
-->

<!-- score:workflow-state
version: 1
issue_id: '3348'
locked: false
active_phase: td_inited
active_branch: codex/aw-3347-ec-review-verify
remaining_sections: []
dirty_paths: []
updated_at: 2026-08-05T08:28:25.127361+00:00
-->
"#;

        let baseline_lc = fold_wi_create("3348", baseline_body, "agentic-workflow");
        let baseline_digest = baseline_lc
            .active_revisions
            .get(&ArtifactKind::Wi)
            .and_then(|rev| rev.as_ref())
            .map(|rev| rev.digest.clone())
            .expect("WI revision digest present");

        // 1. updated_at bump inside aw:loop-state
        let body_loop_updated_at = baseline_body.replace(
            "updated_at: 2026-08-04T08:27:44.814163+00:00",
            "updated_at: 2026-08-06T10:00:00.000000+00:00",
        );
        let lc_loop_updated = fold_wi_create("3348", &body_loop_updated_at, "agentic-workflow");
        assert_eq!(
            lc_loop_updated
                .active_revisions
                .get(&ArtifactKind::Wi)
                .and_then(|rev| rev.as_ref())
                .unwrap()
                .digest,
            baseline_digest
        );

        // 2. added iterations: entry inside aw:loop-state
        let body_iterations_added = baseline_body.replace(
            "iterations: []",
            "iterations:\n- n: 1\n  action: ec\n  outcome: green\n  summary: verified",
        );
        let lc_iterations_added =
            fold_wi_create("3348", &body_iterations_added, "agentic-workflow");
        assert_eq!(
            lc_iterations_added
                .active_revisions
                .get(&ArtifactKind::Wi)
                .and_then(|rev| rev.as_ref())
                .unwrap()
                .digest,
            baseline_digest
        );

        // 3. updated_at bump inside score:workflow-state
        let body_score_updated_at = baseline_body.replace(
            "updated_at: 2026-08-05T08:28:25.127361+00:00",
            "updated_at: 2026-08-06T12:00:00.000000+00:00",
        );
        let lc_score_updated = fold_wi_create("3348", &body_score_updated_at, "agentic-workflow");
        assert_eq!(
            lc_score_updated
                .active_revisions
                .get(&ArtifactKind::Wi)
                .and_then(|rev| rev.as_ref())
                .unwrap()
                .digest,
            baseline_digest
        );

        // 4. appending a whole new AW-owned block
        let body_appended_block = format!(
            "{baseline_body}\n<!-- aw:custom-projection\nversion: 1\nstatus: active\n-->\n"
        );
        let lc_appended_block = fold_wi_create("3348", &body_appended_block, "agentic-workflow");
        assert_eq!(
            lc_appended_block
                .active_revisions
                .get(&ArtifactKind::Wi)
                .and_then(|rev| rev.as_ref())
                .unwrap()
                .digest,
            baseline_digest
        );

        // 5. different across a prose edit inside the ## Problem section
        let body_prose_edited = baseline_body.replace(
            "This is the problem description for WI 3348.",
            "This is an edited problem description for WI 3348.",
        );
        let lc_prose_edited = fold_wi_create("3348", &body_prose_edited, "agentic-workflow");
        assert_ne!(
            lc_prose_edited
                .active_revisions
                .get(&ArtifactKind::Wi)
                .and_then(|rev| rev.as_ref())
                .unwrap()
                .digest,
            baseline_digest
        );

        // 6. fold_wi_update returns accepted: false when only marker blocks changed
        let update_marker_only = fold_wi_update(&baseline_lc, &body_loop_updated_at, None);
        assert!(!update_marker_only.accepted);

        // 7. fold_wi_update returns accepted: true when ## Problem prose changed
        let update_prose_edited = fold_wi_update(&baseline_lc, &body_prose_edited, None);
        assert!(update_prose_edited.accepted);
    }

    #[test]
    fn wi_contract_conflicting_tracker_observation_fails_closed() {
        let body_v1 = "## Description\nInitial WI body.";
        let body_v2 = "## Description\nUpdated WI body v2.";
        let body_v3 = "## Description\nUpdated WI body v3.";
        let body_v1_marker_churn = format!(
            "{body_v1}\n<!-- aw:loop-state\nversion: 1\nupdated_at: 2026-08-06T10:00:00.000000+00:00\n-->\n"
        );

        let initial_lc = fold_wi_create("causal", body_v1, "agentic-workflow");

        // Measurement 1: pre_update_body digest equals stored WI digest, new_body differs
        let res_ordinary = fold_wi_update(&initial_lc, body_v2, Some(body_v1));
        assert!(res_ordinary.accepted, "{:?}", res_ordinary.rejection_reason);
        assert_eq!(res_ordinary.lifecycle.epoch, initial_lc.epoch + 1);
        assert_eq!(
            res_ordinary
                .lifecycle
                .active_revisions
                .get(&ArtifactKind::Wi)
                .and_then(|rev| rev.as_ref())
                .unwrap()
                .digest,
            canonical_wi_digest(body_v2)
        );

        // Measurement 2: pre_update_body digest differs from stored WI digest (out-of-band edit)
        let res_conflict = fold_wi_update(&initial_lc, body_v3, Some(body_v2));
        assert!(!res_conflict.accepted);
        assert!(
            res_conflict
                .rejection_reason
                .as_deref()
                .unwrap_or("")
                .contains("Conflicting tracker observation"),
            "rejection reason should name conflicting tracker observation: {:?}",
            res_conflict.rejection_reason
        );
        assert_eq!(res_conflict.lifecycle, initial_lc);

        // Measurement 3: pre_update_body differs from stored revision ONLY inside AW-owned marker block (negative control)
        let res_marker = fold_wi_update(&initial_lc, body_v2, Some(&body_v1_marker_churn));
        assert!(res_marker.accepted, "{:?}", res_marker.rejection_reason);
        assert_eq!(res_marker.lifecycle.epoch, initial_lc.epoch + 1);

        // Measurement 4: pre_update_body is None (fallback to stored revision)
        let res_none = fold_wi_update(&initial_lc, body_v2, None);
        assert!(res_none.accepted, "{:?}", res_none.rejection_reason);
        assert_eq!(res_none.lifecycle.epoch, initial_lc.epoch + 1);
    }

    #[test]
    fn revisioned_change_wi_milestone_projection() {
        let root = tempfile::tempdir().unwrap();
        let issue = change("wi-v1");

        // Row 7: issue with no carrier or zero-head carries milestones: []
        let zero_proj = zero_head_projection("causal", OwnerVocabulary::Wi);
        assert_eq!(zero_proj["milestones"], serde_json::json!([]));
        let no_carrier_proj = projection_for_issue(root.path(), &issue);
        assert_eq!(no_carrier_proj["milestones"], serde_json::json!([]));

        // Create a 4-stage lifecycle: wi_create, ec_change, td_change, cb_change
        let created = fold_wi_create("causal", "wi-v1", "agentic-workflow");
        let ec = reduce_stage(
            &created,
            ArtifactKind::Ec,
            LifecycleEventKind::EcChange,
            "ec-v1",
            "aw td check",
            OwnerVocabulary::Td,
        );
        let td = reduce_stage(
            &ec,
            ArtifactKind::Td,
            LifecycleEventKind::TdChange,
            "td-v1",
            "aw cb check",
            OwnerVocabulary::Cb,
        );
        let mut cb = reduce_stage(
            &td,
            ArtifactKind::Cb,
            LifecycleEventKind::CbChange,
            "cb-v1",
            "aw cb check",
            OwnerVocabulary::Cb,
        );

        // Row 1: milestones is an array of exactly 4 entries in commit order, matching event_ids
        let proj_4 = render(
            &cb,
            &decide_projection(&cb, &TrackerObservation::empty("wi-v1")),
        );
        let milestones_4 = proj_4["milestones"]
            .as_array()
            .expect("milestones is array");
        assert_eq!(milestones_4.len(), 4);
        for (i, event) in cb.events.iter().enumerate() {
            assert_eq!(milestones_4[i]["event_id"], event.event_id);
            assert_eq!(milestones_4[i]["outcome"], "accepted");
            assert_eq!(
                milestones_4[i]["iteration"],
                event.candidate_revision.iteration
            );
        }

        // Row 2: every milestone carries non-null artifact matching candidate_revision
        for (i, event) in cb.events.iter().enumerate() {
            let artifact = &milestones_4[i]["artifact"];
            assert_ne!(artifact, &serde_json::Value::Null);
            assert_eq!(artifact["id"], event.candidate_revision.id);
            assert_eq!(artifact["digest"], event.candidate_revision.digest);
            let expected_kind = match event.candidate_revision.kind {
                ArtifactKind::Wi => "wi",
                ArtifactKind::Ec => "ec",
                ArtifactKind::Td => "td",
                ArtifactKind::Cb => "cb",
            };
            assert_eq!(artifact["kind"], expected_kind);
        }

        // Row 3: td_change milestone artifact.parents is non-empty matching candidate_revision.parents
        let td_milestone = &milestones_4[2];
        let td_parents = td_milestone["artifact"]["parents"]
            .as_array()
            .expect("parents is array");
        assert!(!td_parents.is_empty());
        let expected_td_parents = &cb.events[2].candidate_revision.parents;
        assert_eq!(td_parents.len(), expected_td_parents.len());
        for (j, parent) in expected_td_parents.iter().enumerate() {
            assert_eq!(td_parents[j]["id"], parent.revision_id);
            assert_eq!(td_parents[j]["digest"], parent.digest);
        }

        // Add 4D evidence to cb and commit cb_commit with proposed event next obligation
        let tuple = cb.active_digest_tuple();
        cb.evidence = ["cb_test", "cb_review", "td_reconcile", "ec_verify_cb"]
            .into_iter()
            .map(|verifier| EvidenceBinding {
                verifier: verifier.to_string(),
                bound_tuple: tuple.clone(),
                passed: true,
                summary: "pass".to_string(),
            })
            .collect();
        let active_cb = cb.active_revisions[&ArtifactKind::Cb].clone().unwrap();
        let commit_event = LifecycleEvent {
            event_id: event_id(&cb),
            predecessor_id: cb.head_event_id.clone(),
            kind: LifecycleEventKind::CbCommit,
            candidate_revision: active_cb,
            bound_tuple: tuple.clone(),
            next_command: "PROPOSED-BY-EVENT".to_string(),
            next_owner: OwnerVocabulary::Wi,
            wi_snapshot: None,
        };
        let committed = reduce_event(&cb, commit_event);
        assert!(committed.accepted);
        let terminal_lc = committed.lifecycle;

        // Row 4: cb_commit milestone next.command is "aw wi show causal", next.owner is "cb"
        let terminal_proj = render(
            &terminal_lc,
            &decide_projection(&terminal_lc, &TrackerObservation::empty("wi-v1")),
        );
        let term_milestones = terminal_proj["milestones"].as_array().unwrap();
        assert_eq!(term_milestones.len(), 5);
        let cb_commit_ms = &term_milestones[4];
        assert_eq!(cb_commit_ms["next"]["command"], "aw wi show causal");
        assert_eq!(cb_commit_ms["next"]["owner"], "cb");

        // Row 5: cb_commit milestone evidence lists 4 verifiers on committed tuple, earlier milestones list []
        let cb_commit_ev = cb_commit_ms["evidence"].as_array().unwrap();
        assert_eq!(cb_commit_ev.len(), 4);
        for i in 0..3 {
            assert_eq!(term_milestones[i]["evidence"], serde_json::json!([]));
        }

        // Row 6: negative control: rejected event (stale predecessor_id) does not alter milestones
        let stale_event = LifecycleEvent {
            event_id: "evt-stale".to_string(),
            predecessor_id: Some("evt-bad".to_string()),
            kind: LifecycleEventKind::WiChange,
            candidate_revision: terminal_lc.events[0].candidate_revision.clone(),
            bound_tuple: ActiveDigestTuple::default(),
            next_command: "aw wi validate causal".to_string(),
            next_owner: OwnerVocabulary::Wi,
            wi_snapshot: None,
        };
        let rej = reduce_event(&terminal_lc, stale_event);
        assert!(!rej.accepted);
        let proj_after_rej = render(
            &rej.lifecycle,
            &decide_projection(&rej.lifecycle, &TrackerObservation::empty("wi-v1")),
        );
        assert_eq!(proj_after_rej["milestones"], terminal_proj["milestones"]);

        // Row 8: empty evidence on terminal lifecycle: cb_commit milestone still reports aw wi show and owner cb, and evidence []
        let mut emptied_lc = terminal_lc.clone();
        emptied_lc.evidence.clear();
        let emptied_proj = render(
            &emptied_lc,
            &decide_projection(&emptied_lc, &TrackerObservation::empty("wi-v1")),
        );
        let emptied_ms = emptied_proj["milestones"].as_array().unwrap();
        let emptied_cb_ms = &emptied_ms[4];
        assert_eq!(emptied_cb_ms["next"]["command"], "aw wi show causal");
        assert_eq!(emptied_cb_ms["next"]["owner"], "cb");
        assert_eq!(emptied_cb_ms["evidence"], serde_json::json!([]));
    }

    #[test]
    fn projection_for_issue_reflects_tracker_observation_state() {
        let root = tempfile::tempdir().unwrap();

        // Row 1: valid, non-terminal carrier & closed issue -> drift reported with reopen remediation
        let issue_open = change("wi-v1");
        let non_terminal_lc = fold_wi_create("causal", "wi-v1", "agentic-workflow");
        save(root.path(), &non_terminal_lc).unwrap();

        let mut issue_closed = issue_open.clone();
        issue_closed.state = IssueState::Closed;

        let proj_row1 = projection_for_issue(root.path(), &issue_closed);
        assert_eq!(proj_row1["drift"], true);
        let remed = proj_row1["remediation"]
            .as_array()
            .expect("remediation is array");
        assert_eq!(remed.len(), 2);
        assert_eq!(remed[0]["command"], "aw wi update causal --state open");
        assert_eq!(remed[0]["owner"], "wi");
        assert_eq!(remed[1]["command"], non_terminal_lc.next.command);
        assert_eq!(remed[1]["owner"], non_terminal_lc.next.owner.as_str());
        assert_eq!(proj_row1["close_authorized"], false);
        assert!(proj_row1["authorize_close_event_id"].is_null());

        // Row 2: valid, non-terminal carrier & open issue (negative control) -> no drift, no remediation
        let proj_row2 = projection_for_issue(root.path(), &issue_open);
        assert_eq!(proj_row2["drift"], false);
        assert_eq!(proj_row2["remediation"], serde_json::json!([]));
        assert_eq!(proj_row2["close_authorized"], false);
        assert!(proj_row2["authorize_close_event_id"].is_null());

        // Construct terminal lifecycle (with CbCommit and 4D evidence)
        let ec = reduce_stage(
            &non_terminal_lc,
            ArtifactKind::Ec,
            LifecycleEventKind::EcChange,
            "ec-v1",
            "aw td check",
            OwnerVocabulary::Td,
        );
        let td = reduce_stage(
            &ec,
            ArtifactKind::Td,
            LifecycleEventKind::TdChange,
            "td-v1",
            "aw cb check",
            OwnerVocabulary::Cb,
        );
        let mut cb = reduce_stage(
            &td,
            ArtifactKind::Cb,
            LifecycleEventKind::CbChange,
            "cb-v1",
            "aw cb check",
            OwnerVocabulary::Cb,
        );
        let tuple = cb.active_digest_tuple();
        cb.evidence = ["cb_test", "cb_review", "td_reconcile", "ec_verify_cb"]
            .into_iter()
            .map(|v| EvidenceBinding {
                verifier: v.to_string(),
                bound_tuple: tuple.clone(),
                passed: true,
                summary: "pass".to_string(),
            })
            .collect();
        let active_cb = cb.active_revisions[&ArtifactKind::Cb].clone().unwrap();
        let commit_evt = LifecycleEvent {
            event_id: event_id(&cb),
            predecessor_id: cb.head_event_id.clone(),
            kind: LifecycleEventKind::CbCommit,
            candidate_revision: active_cb,
            bound_tuple: tuple,
            next_command: "aw wi show causal".to_string(),
            next_owner: OwnerVocabulary::Cb,
            wi_snapshot: None,
        };
        let committed = reduce_event(&cb, commit_evt);
        assert!(committed.accepted);
        let terminal_lc = committed.lifecycle;
        save(root.path(), &terminal_lc).unwrap();

        // Row 3: valid, terminal carrier (head event is cb_commit) & open issue -> close_authorized true with authorize_close_event_id
        let proj_row3 = projection_for_issue(root.path(), &issue_open);
        assert_eq!(proj_row3["drift"], false);
        assert_eq!(proj_row3["close_authorized"], true);
        assert_eq!(proj_row3["authorize_close_event_id"], "evt-005");

        // Row 4: valid, terminal carrier & closed issue (negative control) -> no drift, close_authorized false
        let proj_row4 = projection_for_issue(root.path(), &issue_closed);
        assert_eq!(proj_row4["drift"], false);
        assert_eq!(proj_row4["close_authorized"], false);
        assert!(proj_row4["authorize_close_event_id"].is_null());

        // Row 5: absent, malformed, or legacy carrier & closed issue -> fail-closed projection, no drift
        let legacy_issue = change("<!-- aw:loop-state\nversion: 1\n-->");
        let mut legacy_closed = legacy_issue.clone();
        legacy_closed.state = IssueState::Closed;
        let proj_row5 = projection_for_issue(root.path(), &legacy_closed);
        assert_eq!(proj_row5["drift"], false);
        assert_eq!(proj_row5["close_authorized"], false);
        assert!(proj_row5["authorize_close_event_id"].is_null());
        assert_eq!(proj_row5["ledger"]["epoch"], 0);
    }

    #[test]
    fn revisioned_change_wi_projection_recovery() {
        let root = tempfile::tempdir().unwrap();

        let lc = complete_lifecycle();
        let wi_body = "wi-v1";
        assert_eq!(lc.events.len(), 4);
        assert_eq!(lc.epoch, 4);
        assert_eq!(lc.head_event_id.as_deref(), Some("evt-004"));

        // Row 1: no AW projection
        let obs1 = TrackerObservation::empty(wi_body);
        let dec1 = decide_projection(&lc, &obs1);
        assert!(dec1.accepted);
        assert_eq!(dec1.refusal_reason, None);
        assert_eq!(dec1.target_epoch, 4);
        assert_eq!(dec1.target_head_event_id.as_deref(), Some("evt-004"));
        let covered_events_1: Vec<&str> = dec1.work.iter().map(|w| w.event_id.as_str()).collect();
        assert_eq!(
            covered_events_1,
            vec!["evt-001", "evt-002", "evt-003", "evt-004"]
        );

        // Row 2: already carrying projection with all milestones present
        let obs2 = TrackerObservation::new(
            wi_body,
            Some("evt-004"),
            Some(4),
            vec!["evt-001", "evt-002", "evt-003", "evt-004"],
        );
        let dec2 = decide_projection(&lc, &obs2);
        assert!(dec2.accepted);
        assert_eq!(dec2.refusal_reason, None);
        assert!(
            dec2.work.is_empty(),
            "expected zero writes when all milestones are present"
        );

        // Row 3: older committed head (first 2 events)
        let obs3 = TrackerObservation::new(
            wi_body,
            Some("evt-002"),
            Some(2),
            vec!["evt-001", "evt-002"],
        );
        let dec3 = decide_projection(&lc, &obs3);
        assert!(dec3.accepted);
        let covered_events_3: Vec<&str> = dec3.work.iter().map(|w| w.event_id.as_str()).collect();
        assert_eq!(covered_events_3, vec!["evt-003", "evt-004"]);

        // Row 4: milestone for event 2 absent while 1, 3, and 4 present
        let obs4 = TrackerObservation::new(
            wi_body,
            Some("evt-004"),
            Some(4),
            vec!["evt-001", "evt-003", "evt-004"],
        );
        let dec4 = decide_projection(&lc, &obs4);
        assert!(dec4.accepted);
        let covered_events_4: Vec<&str> = dec4.work.iter().map(|w| w.event_id.as_str()).collect();
        assert_eq!(covered_events_4, vec!["evt-002"]);

        // Row 5: head names an event id the ledger has never committed
        let obs5 = TrackerObservation::new(
            wi_body,
            Some("evt-999-uncommitted"),
            Some(99),
            Vec::<String>::new(),
        );
        let dec5 = decide_projection(&lc, &obs5);
        assert!(!dec5.accepted);
        assert!(dec5.work.is_empty(), "refusal must yield zero writes");
        let reason5 = dec5.refusal_reason.expect("refusal reason present");
        assert!(
            reason5.contains("evt-999-uncommitted"),
            "refusal reason should name conflicting head: {}",
            reason5
        );

        // Row 6: canonical WI body digest differs from ledger active WI revision digest
        let obs6 = TrackerObservation::empty("unseen human prose edit on tracker");
        let dec6 = decide_projection(&lc, &obs6);
        assert!(!dec6.accepted);
        assert!(dec6.work.is_empty(), "refusal must yield zero writes");
        assert!(dec6.refusal_reason.is_some());

        // Row 7: rejected event applied to lifecycle (negative control)
        let stale_event = LifecycleEvent {
            event_id: "evt-005".to_string(),
            predecessor_id: Some("evt-stale".to_string()),
            kind: LifecycleEventKind::WiChange,
            candidate_revision: lc.events[0].candidate_revision.clone(),
            bound_tuple: ActiveDigestTuple::default(),
            next_command: "aw wi validate causal".to_string(),
            next_owner: OwnerVocabulary::Wi,
            wi_snapshot: None,
        };
        let rejected_res = reduce_event(&lc, stale_event);
        assert!(!rejected_res.accepted);
        let dec7 = decide_projection(&rejected_res.lifecycle, &obs1);
        assert_eq!(dec7, dec1);

        // Row 8: lifecycle re-read from persisted carrier after save
        save(root.path(), &lc).unwrap();
        let reloaded_lc = load(root.path(), "causal").unwrap().unwrap();
        let mut obs8 = TrackerObservation::empty(wi_body);
        let dec8_first = decide_projection(&reloaded_lc, &obs8);
        assert_eq!(dec8_first, dec1);

        obs8.apply_work(
            dec8_first.target_epoch,
            dec8_first.target_head_event_id.clone(),
            &dec8_first.work,
        );
        let dec8_second = decide_projection(&reloaded_lc, &obs8);
        assert!(dec8_second.accepted);
        assert!(
            dec8_second.work.is_empty(),
            "recomputed decision after applying yielded work must yield no work"
        );
    }

    #[test]
    fn tracker_observation_from_marker_measurements() {
        let root = tempfile::tempdir().unwrap();

        // Build a 3-event lifecycle carrier (evt-001, evt-002, evt-003, head = evt-003, epoch = 3)
        let created = fold_wi_create("causal", "## Problem\n\nInitial prose", "agentic-workflow");
        let ec = reduce_stage(
            &created,
            ArtifactKind::Ec,
            LifecycleEventKind::EcChange,
            "ec-v1",
            "aw td check",
            OwnerVocabulary::Td,
        );
        let lc = reduce_stage(
            &ec,
            ArtifactKind::Td,
            LifecycleEventKind::TdChange,
            "td-v1",
            "aw cb check",
            OwnerVocabulary::Cb,
        );
        assert_eq!(lc.events.len(), 3);
        assert_eq!(lc.epoch, 3);
        assert_eq!(lc.head_event_id.as_deref(), Some("evt-003"));
        save(root.path(), &lc).unwrap();

        let base_body = "## Problem\n\nInitial prose";

        // Row 1: Change WI; tracker body carries the marker rendered for that lifecycle
        let body_with_marker = upsert_projection_marker(base_body, &lc).unwrap();
        let issue_row1 = change(&body_with_marker);
        let proj_row1 = projection_for_issue(root.path(), &issue_row1);
        assert_eq!(proj_row1["accepted"], true);
        assert!(proj_row1["refusal_reason"].is_null());
        let parsed1 = parse_projection_marker(&body_with_marker).expect("marker must parse back");
        assert_eq!(parsed1.head_event_id.as_deref(), Some("evt-003"));
        assert_eq!(parsed1.epoch, Some(3));
        let expected_present: BTreeSet<String> = vec!["evt-001", "evt-002", "evt-003"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(parsed1.present_event_ids, expected_present);

        // Row 2: the same ledger; the body's marker names head evt-999, an id the ledger never committed
        let marker_row2 = render_projection_marker(Some("evt-999"), Some(4), &expected_present);
        let body_row2 = format!("{base_body}\n\n{marker_row2}");
        let issue_row2 = change(&body_row2);
        let proj_row2 = projection_for_issue(root.path(), &issue_row2);
        assert_eq!(proj_row2["accepted"], false);
        let reason2 = proj_row2["refusal_reason"]
            .as_str()
            .expect("refusal reason string");
        assert!(
            reason2.contains("evt-999"),
            "reason should contain evt-999: {reason2}"
        );

        // Row 3: the same ledger; the body's marker names head evt-002, which is committed, together with epoch 7
        let marker_row3 = render_projection_marker(
            Some("evt-002"),
            Some(7),
            &vec!["evt-001", "evt-002"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        let body_row3 = format!("{base_body}\n\n{marker_row3}");
        let issue_row3 = change(&body_row3);
        let proj_row3 = projection_for_issue(root.path(), &issue_row3);
        assert_eq!(proj_row3["accepted"], false);
        let reason3 = proj_row3["refusal_reason"]
            .as_str()
            .expect("refusal reason string");
        assert!(
            reason3.contains("epoch 7"),
            "reason should contain epoch 7: {reason3}"
        );
        assert!(
            reason3.contains("ledger epoch 2"),
            "reason should contain ledger epoch 2: {reason3}"
        );
        assert_ne!(reason2, reason3);

        // Row 4: one body carrying the marker, and the same body with the marker block deleted, prose byte-identical otherwise
        let digest_with_marker = canonical_wi_digest(&body_with_marker);
        let digest_without_marker = canonical_wi_digest(base_body);
        assert_eq!(digest_with_marker, digest_without_marker);

        // Row 5: row 4's pair again, but with one word of ## Problem prose changed and the same marker on both (negative control)
        let body_modified_prose = "## Problem\n\nModified prose";
        let body_modified_with_marker = upsert_projection_marker(body_modified_prose, &lc).unwrap();
        let digest_modified = canonical_wi_digest(&body_modified_with_marker);
        assert_ne!(digest_with_marker, digest_modified);

        // Row 6: a Change WI body carrying <!-- aw:loop-state ... --> and no lifecycle marker (negative control)
        let legacy_body = "## Problem\n\n<!-- aw:loop-state\nversion: 1\n-->";
        let issue_legacy = change(legacy_body);
        let proj_legacy = projection_for_issue(root.path(), &issue_legacy);
        assert_eq!(proj_legacy["ledger"]["epoch"], 0);
        assert_eq!(proj_legacy["next"]["owner"], "migration");
        assert!(proj_legacy.get("accepted").is_none());

        // Row 7: the same ledger; the body's marker names head evt-003 and epoch 3 — both agreeing — and a projected-event set that includes one id the ledger never committed
        let marker_row7 = render_projection_marker(
            Some("evt-003"),
            Some(3),
            &vec!["evt-001", "evt-002", "evt-003", "evt-bogus"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        let body_row7 = format!("{base_body}\n\n{marker_row7}");
        let issue_row7 = change(&body_row7);
        let proj_row7 = projection_for_issue(root.path(), &issue_row7);
        assert_eq!(proj_row7["accepted"], false);
        let reason7 = proj_row7["refusal_reason"]
            .as_str()
            .expect("refusal reason string");
        assert!(
            reason7.contains("evt-bogus"),
            "reason should contain uncommitted id evt-bogus: {reason7}"
        );
        assert!(
            reason7.contains("milestone"),
            "reason should identify it as a milestone: {reason7}"
        );
        assert_ne!(reason7, reason2);
        assert_ne!(reason7, reason3);
    }

    #[test]
    fn tracker_observation_malformed_marker_fails_closed() {
        let root = tempfile::tempdir().unwrap();
        let lc = fold_wi_create("causal", "wi-v1", "agentic-workflow");
        save(root.path(), &lc).unwrap();

        let malformed_body = "wi-v1\n\n<!-- aw:projection\nnot: yaml: ::::\n-->";
        let issue = change(malformed_body);
        let proj = projection_for_issue(root.path(), &issue);
        assert_eq!(proj["accepted"], false);
        assert!(!proj["refusal_reason"].is_null());
    }

    #[test]
    fn change_wi_terminal_projection() {
        let root = tempfile::tempdir().unwrap();

        // Row 1: non-terminal lifecycle (folded through wi_create, ec_change, td_change, cb_change, no commit) and open issue
        let created = fold_wi_create("causal", "wi-v1", "agentic-workflow");
        let ec = reduce_stage(
            &created,
            ArtifactKind::Ec,
            LifecycleEventKind::EcChange,
            "ec-v1",
            "aw td check",
            OwnerVocabulary::Td,
        );
        let td = reduce_stage(
            &ec,
            ArtifactKind::Td,
            LifecycleEventKind::TdChange,
            "td-v1",
            "aw cb check",
            OwnerVocabulary::Cb,
        );
        let lc1 = reduce_stage(
            &td,
            ArtifactKind::Cb,
            LifecycleEventKind::CbChange,
            "cb-v1",
            "aw cb check",
            OwnerVocabulary::Cb,
        );

        let obs_open = TrackerObservation::new(
            "wi-v1",
            lc1.head_event_id.clone(),
            Some(lc1.epoch),
            lc1.events.iter().map(|e| e.event_id.clone()),
        );

        let dec1 = decide_projection(&lc1, &obs_open);
        assert!(dec1.accepted);
        assert!(!dec1.complete);
        assert!(!dec1.close_authorized);
        assert_eq!(dec1.authorize_close_event_id, None);
        assert!(!dec1.drift);
        assert!(dec1.remediation.is_empty());

        // Row 2: terminal lifecycle (advanced by accepted cb_commit) and open issue
        let mut cb_with_evidence = lc1.clone();
        let tuple = cb_with_evidence.active_digest_tuple();
        cb_with_evidence.evidence = ["cb_test", "cb_review", "td_reconcile", "ec_verify_cb"]
            .into_iter()
            .map(|verifier| EvidenceBinding {
                verifier: verifier.to_string(),
                bound_tuple: tuple.clone(),
                passed: true,
                summary: "pass".to_string(),
            })
            .collect();
        let active_cb = cb_with_evidence.active_revisions[&ArtifactKind::Cb]
            .clone()
            .unwrap();
        let commit_event = LifecycleEvent {
            event_id: event_id(&cb_with_evidence),
            predecessor_id: cb_with_evidence.head_event_id.clone(),
            kind: LifecycleEventKind::CbCommit,
            candidate_revision: active_cb,
            bound_tuple: tuple,
            next_command: "aw wi show causal".to_string(),
            next_owner: OwnerVocabulary::Cb,
            wi_snapshot: None,
        };
        let commit_res = reduce_event(&cb_with_evidence, commit_event);
        assert!(commit_res.accepted);
        let lc2 = commit_res.lifecycle;

        let obs2 = TrackerObservation::new(
            "wi-v1",
            lc1.head_event_id.clone(),
            Some(lc1.epoch),
            lc1.events.iter().map(|e| e.event_id.clone()),
        );

        let dec2 = decide_projection(&lc2, &obs2);
        assert!(dec2.accepted);
        assert!(dec2.complete);
        assert!(dec2.close_authorized);
        assert_eq!(dec2.authorize_close_event_id, lc2.head_event_id);
        assert_eq!(dec2.authorize_close_event_id, Some("evt-005".to_string()));
        assert!(!dec2.drift);

        // Row 3: terminal lifecycle and closed issue already at that terminal event
        let obs3 = TrackerObservation::new(
            "wi-v1",
            lc2.head_event_id.clone(),
            Some(lc2.epoch),
            lc2.events.iter().map(|e| e.event_id.clone()),
        )
        .with_state(IssueState::Closed);

        let dec3 = decide_projection(&lc2, &obs3);
        assert!(dec3.accepted);
        assert!(dec3.complete);
        assert!(!dec3.close_authorized);
        assert_eq!(dec3.authorize_close_event_id, None);
        assert!(!dec3.drift);

        // Row 4: non-terminal lifecycle of row 1 and closed issue (drift)
        let obs4 = TrackerObservation::new(
            "wi-v1",
            lc1.head_event_id.clone(),
            Some(lc1.epoch),
            lc1.events.iter().map(|e| e.event_id.clone()),
        )
        .with_state(IssueState::Closed);

        let dec4 = decide_projection(&lc1, &obs4);
        assert!(dec4.accepted);
        assert!(!dec4.complete);
        assert!(!dec4.close_authorized);
        assert_eq!(dec4.authorize_close_event_id, None);
        assert!(dec4.drift);
        assert_eq!(dec4.remediation.len(), 2);
        assert_eq!(dec4.remediation[0].owner, OwnerVocabulary::Wi);
        assert_eq!(
            dec4.remediation[0].command,
            format!("aw wi update {} --state open", lc1.slug)
        );
        assert_eq!(dec4.remediation[1].command, lc1.next.command);
        assert_eq!(dec4.remediation[1].owner, lc1.next.owner);

        let created_b = fold_wi_create("other-slug", "wi-v1", "agentic-workflow");
        let lc1_b = reduce_stage(
            &created_b,
            ArtifactKind::Ec,
            LifecycleEventKind::EcChange,
            "ec-v1",
            "aw ec check",
            OwnerVocabulary::Ec,
        );
        let obs4_b = TrackerObservation::new(
            "wi-v1",
            lc1_b.head_event_id.clone(),
            Some(lc1_b.epoch),
            lc1_b.events.iter().map(|e| e.event_id.clone()),
        )
        .with_state(IssueState::Closed);

        let dec4_b = decide_projection(&lc1_b, &obs4_b);
        assert!(dec4_b.accepted);
        assert!(!dec4_b.complete);
        assert!(!dec4_b.close_authorized);
        assert_eq!(dec4_b.authorize_close_event_id, None);
        assert!(dec4_b.drift);
        assert_eq!(dec4_b.remediation.len(), 2);
        assert_eq!(dec4_b.remediation[0].owner, OwnerVocabulary::Wi);
        assert_eq!(
            dec4_b.remediation[0].command,
            format!("aw wi update {} --state open", lc1_b.slug)
        );
        assert_eq!(dec4_b.remediation[1].command, lc1_b.next.command);
        assert_eq!(dec4_b.remediation[1].owner, lc1_b.next.owner);

        // Row 5: lifecycle whose terminal field is true but head event is td_change (not cb_commit)
        let mut lc5 = td.clone();
        lc5.terminal = true;
        let obs5 = TrackerObservation::new(
            "wi-v1",
            lc5.head_event_id.clone(),
            Some(lc5.epoch),
            lc5.events.iter().map(|e| e.event_id.clone()),
        );

        let dec5 = decide_projection(&lc5, &obs5);
        assert!(dec5.accepted);
        assert!(!dec5.close_authorized);
        assert_eq!(dec5.authorize_close_event_id, None);
        assert!(!dec5.complete);

        // Row 6: cb_commit rejected by reduce_event (missing 4D evidence) applied to row 1 lifecycle against open observation
        let no_evidence_cb = lc1.clone();
        let active_cb6 = no_evidence_cb.active_revisions[&ArtifactKind::Cb]
            .clone()
            .unwrap();
        let rejected_commit_event = LifecycleEvent {
            event_id: event_id(&no_evidence_cb),
            predecessor_id: no_evidence_cb.head_event_id.clone(),
            kind: LifecycleEventKind::CbCommit,
            candidate_revision: active_cb6,
            bound_tuple: no_evidence_cb.active_digest_tuple(),
            next_command: "aw wi show causal".to_string(),
            next_owner: OwnerVocabulary::Cb,
            wi_snapshot: None,
        };
        let rejected_res = reduce_event(&no_evidence_cb, rejected_commit_event);
        assert!(!rejected_res.accepted);
        let dec6 = decide_projection(&rejected_res.lifecycle, &obs_open);
        assert_eq!(dec6, dec1);
        assert!(!dec6.close_authorized);

        // Row 7: terminal lifecycle of row 2 and observation of open issue whose head names an event id the ledger never committed
        let obs7 = TrackerObservation::new(
            "wi-v1",
            Some("evt-999-uncommitted"),
            Some(99),
            Vec::<String>::new(),
        );

        let dec7 = decide_projection(&lc2, &obs7);
        assert!(!dec7.accepted);
        assert!(!dec7.close_authorized);
        assert_eq!(dec7.authorize_close_event_id, None);
        assert!(
            dec7.work.is_empty(),
            "refusal must yield zero milestone work"
        );

        // Row 8: terminal lifecycle re-read from persisted carrier after save against open observation; then updated by close authorized
        save(root.path(), &lc2).unwrap();
        let reloaded_lc2 = load(root.path(), "causal").unwrap().unwrap();
        let mut obs8 = TrackerObservation::new(
            "wi-v1",
            lc1.head_event_id.clone(),
            Some(lc1.epoch),
            lc1.events.iter().map(|e| e.event_id.clone()),
        );

        let dec8_first = decide_projection(&reloaded_lc2, &obs8);
        assert_eq!(dec8_first, dec2);
        assert!(dec8_first.close_authorized);
        assert_eq!(
            dec8_first.authorize_close_event_id,
            reloaded_lc2.head_event_id
        );

        obs8.apply_close();
        let dec8_second = decide_projection(&reloaded_lc2, &obs8);
        assert!(dec8_second.accepted);
        assert!(!dec8_second.close_authorized);
        assert_eq!(dec8_second.authorize_close_event_id, None);
        assert!(!dec8_second.drift);
    }

    #[test]
    fn revisioned_change_wi_conflicting_observation() {
        let root = tempfile::tempdir().unwrap();

        let created = fold_wi_create("causal", "wi-v1", "agentic-workflow");
        let ec = reduce_stage(
            &created,
            ArtifactKind::Ec,
            LifecycleEventKind::EcChange,
            "ec-v1",
            "aw td check",
            OwnerVocabulary::Td,
        );
        let td = reduce_stage(
            &ec,
            ArtifactKind::Td,
            LifecycleEventKind::TdChange,
            "td-v1",
            "aw cb check",
            OwnerVocabulary::Cb,
        );
        let lc4 = reduce_stage(
            &td,
            ArtifactKind::Cb,
            LifecycleEventKind::CbChange,
            "cb-v1",
            "aw cb check",
            OwnerVocabulary::Cb,
        );
        let wi_body = "wi-v1";

        // Row 1: Observation naming uncommitted event `evt-900`
        let obs1 = TrackerObservation::new(
            wi_body,
            lc4.head_event_id.clone(),
            Some(lc4.epoch),
            vec!["evt-001", "evt-002", "evt-003", "evt-004", "evt-900"],
        );
        let dec1 = decide_projection(&lc4, &obs1);
        assert!(!dec1.accepted);
        assert!(dec1.work.is_empty());
        let reason1 = dec1
            .refusal_reason
            .as_deref()
            .expect("refusal reason present");
        assert!(
            reason1.contains("evt-900"),
            "refusal reason should name foreign milestone id: {}",
            reason1
        );

        // Row 9: Observation with foreign milestone `evt-0025` sorting among committed ids
        let obs9 = TrackerObservation::new(
            wi_body,
            lc4.head_event_id.clone(),
            Some(lc4.epoch),
            vec!["evt-001", "evt-002", "evt-0025", "evt-003", "evt-004"],
        );
        let dec9 = decide_projection(&lc4, &obs9);
        assert!(!dec9.accepted);
        assert!(dec9.work.is_empty());
        let reason9 = dec9
            .refusal_reason
            .as_deref()
            .expect("refusal reason present");
        assert!(
            reason9.contains("evt-0025"),
            "refusal reason should name foreign milestone id: {}",
            reason9
        );

        // Row 2: Four-event lifecycle after rejected cb_commit vs observation with evt-001..evt-005;
        // and paired control with accepted cb_commit.
        let no_evidence_cb = lc4.clone();
        let active_cb = no_evidence_cb.active_revisions[&ArtifactKind::Cb]
            .clone()
            .unwrap();
        let cb_commit_event = LifecycleEvent {
            event_id: event_id(&no_evidence_cb),
            predecessor_id: no_evidence_cb.head_event_id.clone(),
            kind: LifecycleEventKind::CbCommit,
            candidate_revision: active_cb.clone(),
            bound_tuple: no_evidence_cb.active_digest_tuple(),
            next_command: "aw wi show causal".to_string(),
            next_owner: OwnerVocabulary::Cb,
            wi_snapshot: None,
        };
        let rejected_res = reduce_event(&no_evidence_cb, cb_commit_event.clone());
        assert!(!rejected_res.accepted);
        let lc_rejected = rejected_res.lifecycle;

        let obs2 = TrackerObservation::new(
            wi_body,
            lc4.head_event_id.clone(),
            Some(lc4.epoch),
            vec!["evt-001", "evt-002", "evt-003", "evt-004", "evt-005"],
        );
        let dec2_rejected = decide_projection(&lc_rejected, &obs2);
        assert!(!dec2_rejected.accepted);
        assert!(dec2_rejected.work.is_empty());
        assert!(dec2_rejected
            .refusal_reason
            .as_deref()
            .unwrap_or("")
            .contains("evt-005"));

        let mut cb_with_evidence = lc4.clone();
        let tuple = cb_with_evidence.active_digest_tuple();
        cb_with_evidence.evidence = ["cb_test", "cb_review", "td_reconcile", "ec_verify_cb"]
            .into_iter()
            .map(|verifier| EvidenceBinding {
                verifier: verifier.to_string(),
                bound_tuple: tuple.clone(),
                passed: true,
                summary: "pass".to_string(),
            })
            .collect();
        let accepted_res = reduce_event(&cb_with_evidence, cb_commit_event);
        assert!(accepted_res.accepted);
        let lc_accepted = accepted_res.lifecycle;

        let dec2_accepted = decide_projection(&lc_accepted, &obs2);
        assert!(dec2_accepted.accepted);
        assert!(dec2_accepted.work.is_empty());

        // Row 3: Observed head is committed `evt-002` but observed epoch is 4 (epoch disagreement)
        let obs3 = TrackerObservation::new(
            wi_body,
            Some("evt-002"),
            Some(4),
            vec!["evt-001", "evt-002"],
        );
        let dec3 = decide_projection(&lc4, &obs3);
        assert!(!dec3.accepted);
        assert!(dec3.work.is_empty());
        let reason3 = dec3
            .refusal_reason
            .as_deref()
            .expect("refusal reason present");
        assert!(
            reason3.contains("epoch") || reason3.contains("4"),
            "refusal reason should name epoch disagreement: {}",
            reason3
        );

        // Row 4: Negative control for Row 3: Observed head `evt-002` with epoch 2
        let obs4 = TrackerObservation::new(
            wi_body,
            Some("evt-002"),
            Some(2),
            vec!["evt-001", "evt-002"],
        );
        let dec4 = decide_projection(&lc4, &obs4);
        assert!(dec4.accepted);
        let covered_events_4: Vec<&str> = dec4.work.iter().map(|w| w.event_id.as_str()).collect();
        assert_eq!(covered_events_4, vec!["evt-003", "evt-004"]);

        // Row 5: Terminal lifecycle against three refusing observations:
        // (a) foreign milestone id (`evt-900`)
        // (b) epoch disagreement (head `evt-002`, epoch 99)
        // (c) head never committed (`evt-999-uncommitted`)
        let obs5_a = TrackerObservation::new(
            wi_body,
            lc_accepted.head_event_id.clone(),
            Some(lc_accepted.epoch),
            vec![
                "evt-001", "evt-002", "evt-003", "evt-004", "evt-005", "evt-900",
            ],
        );
        let obs5_b = TrackerObservation::new(
            wi_body,
            Some("evt-002"),
            Some(99),
            vec!["evt-001", "evt-002"],
        );
        let obs5_c = TrackerObservation::new(
            wi_body,
            Some("evt-999-uncommitted"),
            Some(99),
            Vec::<String>::new(),
        );

        for obs in [&obs5_a, &obs5_b, &obs5_c] {
            let dec = decide_projection(&lc_accepted, obs);
            assert!(!dec.accepted);
            assert!(dec.work.is_empty());
            assert!(!dec.close_authorized);
            assert_eq!(dec.authorize_close_event_id, None);
        }

        // Row 6: Negative control: 4-event lifecycle against:
        // (a) head names uncommitted event -> refuses with zero work
        // (b) body digest does not match -> refuses with zero work
        // (c) self-consistent observation carrying every committed milestone at ledger's identity -> does not refuse, yields no work
        let obs6_a = TrackerObservation::new(
            wi_body,
            Some("evt-999-uncommitted"),
            Some(99),
            Vec::<String>::new(),
        );
        let obs6_b = TrackerObservation::empty("unseen human prose edit on tracker");
        let obs6_c = TrackerObservation::new(
            wi_body,
            lc4.head_event_id.clone(),
            Some(lc4.epoch),
            vec!["evt-001", "evt-002", "evt-003", "evt-004"],
        );

        let dec6_a = decide_projection(&lc4, &obs6_a);
        assert!(!dec6_a.accepted);
        assert!(dec6_a.work.is_empty());

        let dec6_b = decide_projection(&lc4, &obs6_b);
        assert!(!dec6_b.accepted);
        assert!(dec6_b.work.is_empty());

        let dec6_c = decide_projection(&lc4, &obs6_c);
        assert!(dec6_c.accepted);
        assert!(dec6_c.work.is_empty());

        // Row 7: Negative control: 4-event lifecycle and observation with no epoch (None) at older committed head (evt-002) with present milestones
        let obs7 =
            TrackerObservation::new(wi_body, Some("evt-002"), None, vec!["evt-001", "evt-002"]);
        let dec7 = decide_projection(&lc4, &obs7);
        assert!(dec7.accepted);
        let covered_events_7: Vec<&str> = dec7.work.iter().map(|w| w.event_id.as_str()).collect();
        assert_eq!(covered_events_7, vec!["evt-003", "evt-004"]);

        // Row 8: Lifecycle re-read from persisted carrier after save against Row 1's and Row 4's observations
        save(root.path(), &lc4).unwrap();
        let reloaded_lc4 = load(root.path(), "causal").unwrap().unwrap();

        let dec8_row1 = decide_projection(&reloaded_lc4, &obs1);
        assert_eq!(dec8_row1, dec1);

        let dec8_row4 = decide_projection(&reloaded_lc4, &obs4);
        assert_eq!(dec8_row4, dec4);
    }

    #[test]
    fn reducer_evidence_eviction() {
        use crate::cli::ec_verdict::{decide_target_verdict, VerificationTarget};

        // Assemble starting lifecycle with 4 accepted revisions (WI, EC, TD, CB)
        let mut start_lc = complete_lifecycle();
        let starting_tuple = start_lc.active_digest_tuple();

        // 8 passing evidence bindings: 4 commit verifiers + 4 dimension verifiers
        let verifiers = [
            "cb_test",
            "cb_review",
            "td_reconcile",
            "ec_verify_cb",
            "behavior",
            "efficiency",
            "security",
            "stability",
        ];
        start_lc.evidence = verifiers
            .iter()
            .map(|v| EvidenceBinding {
                verifier: v.to_string(),
                bound_tuple: starting_tuple.clone(),
                passed: true,
                summary: format!("evidence for {v}"),
            })
            .collect();

        let initial_evidence = start_lc.evidence.clone();
        assert_eq!(initial_evidence.len(), 8);

        // Row 1: Reduce starting lifecycle with an accepted CbChange for a new CB revision
        let cb_candidate = artifact_revision(
            ArtifactKind::Cb,
            canonical_digest("cb-v2"),
            expected_parent_set(&start_lc, ArtifactKind::Cb).expect("active CB parents"),
            start_lc.epoch + 1,
        );
        let cb_change_event = LifecycleEvent {
            event_id: event_id(&start_lc),
            predecessor_id: start_lc.head_event_id.clone(),
            kind: LifecycleEventKind::CbChange,
            bound_tuple: candidate_tuple(&start_lc, &cb_candidate),
            candidate_revision: cb_candidate.clone(),
            next_command: "aw cb check".to_string(),
            next_owner: OwnerVocabulary::Cb,
            wi_snapshot: None,
        };
        let res1 = reduce_event(&start_lc, cb_change_event);
        assert!(
            res1.accepted,
            "Row 1 event must be accepted: {:?}",
            res1.rejection_reason
        );
        let lc_row1 = res1.lifecycle;

        assert!(
            lc_row1.evidence.is_empty(),
            "Row 1: returned evidence must be empty"
        );
        assert_eq!(
            lc_row1.active_revisions[&ArtifactKind::Td],
            start_lc.active_revisions[&ArtifactKind::Td],
            "Row 1: active Td revision must be unchanged and still active"
        );
        assert!(
            lc_row1.active_revisions[&ArtifactKind::Td].is_some(),
            "Row 1: Td revision must still be active"
        );
        assert_eq!(
            lc_row1.active_revisions[&ArtifactKind::Cb]
                .as_ref()
                .map(|r| &r.id),
            Some(&cb_candidate.id),
            "Row 1: active Cb revision must be the new candidate"
        );
        assert_eq!(
            lc_row1.invalidations.len(),
            start_lc.invalidations.len() + 1,
            "Row 1: exactly one invalidation record must be appended"
        );
        let inv1 = lc_row1.invalidations.last().unwrap();
        assert!(
            inv1.invalidated_kinds.is_empty(),
            "Row 1: invalidated_kinds must be empty"
        );

        // Row 2: Invalidation record appended in Row 1
        let mut expected_verifiers = verifiers.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        expected_verifiers.sort();
        assert_eq!(
            inv1.evicted_evidence_verifiers, expected_verifiers,
            "Row 2: evicted_evidence_verifiers must name all eight verifiers"
        );
        assert_eq!(
            inv1.evicted_evidence, initial_evidence,
            "Row 2: evicted_evidence must carry all eight bindings"
        );

        // Row 3: Target verdict for Cb target on Row 1's lifecycle
        let verdict_row3 = decide_target_verdict(&lc_row1, VerificationTarget::Cb);
        assert!(
            !verdict_row3.is_green(),
            "Row 3: Cb verdict must not be green"
        );
        assert!(
            verdict_row3.stale_dimensions.is_empty(),
            "Row 3: stale_dimensions must be empty"
        );
        assert_eq!(
            verdict_row3.missing_dimensions,
            vec!["behavior", "efficiency", "security", "stability"],
            "Row 3: missing_dimensions must name all four CB required dimensions"
        );
        assert_eq!(
            verdict_row3.reason(),
            Some("missing required dimension(s): behavior, efficiency, security, stability"),
            "Row 3: reason must match exact missing dimensions string"
        );

        // Row 4: Starting lifecycle reduced with accepted WiChange, passed to decide_target_verdict for Td target
        let res4 = fold_wi_update(&start_lc, "wi-v2", Some("wi-v1"));
        assert!(
            res4.accepted,
            "Row 4: WiChange event must be accepted: {:?}",
            res4.rejection_reason
        );
        let lc_row4 = res4.lifecycle;

        assert!(
            lc_row4.active_revisions[&ArtifactKind::Ec].is_none(),
            "Row 4: Ec must be None"
        );
        assert!(
            lc_row4.active_revisions[&ArtifactKind::Td].is_none(),
            "Row 4: Td must be None"
        );
        assert!(
            lc_row4.active_revisions[&ArtifactKind::Cb].is_none(),
            "Row 4: Cb must be None"
        );
        assert!(lc_row4.evidence.is_empty(), "Row 4: evidence must be empty");

        let verdict_row4 = decide_target_verdict(&lc_row4, VerificationTarget::Td);
        assert!(
            !verdict_row4.is_green(),
            "Row 4: Td verdict must not be green"
        );
        assert!(
            verdict_row4.stale_dimensions.is_empty(),
            "Row 4: stale_dimensions must be empty"
        );
        assert_eq!(
            verdict_row4.reason(),
            Some("missing required dimension(s): behavior, security"),
            "Row 4: reason must match exact missing Td dimensions string"
        );

        // Row 5: Negative control - starting lifecycle reduced with accepted CbCommit for already-active CB revision
        let active_cb = start_lc.active_revisions[&ArtifactKind::Cb]
            .clone()
            .expect("active CB revision present");
        let commit_event = LifecycleEvent {
            event_id: event_id(&start_lc),
            predecessor_id: start_lc.head_event_id.clone(),
            kind: LifecycleEventKind::CbCommit,
            candidate_revision: active_cb,
            bound_tuple: starting_tuple,
            next_command: "aw wi show causal".to_string(),
            next_owner: OwnerVocabulary::Cb,
            wi_snapshot: None,
        };
        let res5 = reduce_event(&start_lc, commit_event);
        assert!(
            res5.accepted,
            "Row 5: CbCommit event must be accepted: {:?}",
            res5.rejection_reason
        );
        assert_eq!(
            res5.lifecycle.evidence, initial_evidence,
            "Row 5: CbCommit retaining branch must retain all eight evidence bindings"
        );
    }

    #[test]
    fn test_carrier_publish_cas_lease_rows_1_to_8() {
        let root = tempfile::tempdir().unwrap();
        let project_root = root.path();

        // Initial setup: carrier at head H (H = Some("evt-001")) for "causal"
        let issue_a = change("wi-v1");
        record_create(project_root, &issue_a).unwrap();
        let lc_h = load(project_root, "causal").unwrap().unwrap();
        let head_h = lc_h.head_event_id.clone();
        assert_eq!(head_h, Some("evt-001".to_string()));

        // Two writers load at head H
        // Writer 1 folds candidate 1
        let res1 = fold_wi_update(&lc_h, "wi-v2-w1", Some("wi-v1"));
        assert!(res1.accepted);
        let cand1 = res1.lifecycle;

        // Writer 2 folds candidate 2
        let res2 = fold_wi_update(&lc_h, "wi-v2-w2", Some("wi-v1"));
        assert!(res2.accepted);
        let cand2 = res2.lifecycle;

        // Row 1: First publish (Writer 1) succeeds; second publish (Writer 2) is refused for moved head
        let outcome1 = publish_lifecycle_cas(project_root, head_h.as_deref(), &cand1).unwrap();
        assert_eq!(outcome1, PublishOutcome::Applied);

        let bytes_before_refusal = std::fs::read(carrier_path(project_root, "causal")).unwrap();

        let outcome2 = publish_lifecycle_cas(project_root, head_h.as_deref(), &cand2).unwrap();
        let outcome2_refusal_text = match &outcome2 {
            PublishOutcome::Refused(reason) => {
                assert!(
                    reason.contains("moved head"),
                    "Row 1: refusal reason must contain 'moved head', got: {reason}"
                );
                reason.clone()
            }
            other => panic!("Row 1: expected Refused, got: {other:?}"),
        };

        let reloaded_after_w2 = load(project_root, "causal").unwrap().unwrap();
        assert_eq!(
            reloaded_after_w2.head_event_id,
            Some("evt-002".to_string()),
            "Row 1: head_event_id must be Writer 1's event"
        );
        assert_eq!(
            reloaded_after_w2.events.len(),
            2,
            "Row 1: events must not contain Writer 2's event"
        );
        assert_eq!(
            reloaded_after_w2
                .events
                .last()
                .unwrap()
                .candidate_revision
                .id,
            cand1.events.last().unwrap().candidate_revision.id,
            "Row 1: last event must be Writer 1's event"
        );
        assert_eq!(
            reloaded_after_w2.epoch, 2,
            "Row 1: epoch must advance exactly once"
        );

        // Row 2: Bytes of carrier file read immediately before and immediately after refused publish are identical
        let bytes_after_refusal = std::fs::read(carrier_path(project_root, "causal")).unwrap();
        assert_eq!(
            bytes_before_refusal, bytes_after_refusal,
            "Row 2: carrier file bytes must be identical before and after refused publish"
        );

        // Row 3: Retry winning publish with same (predecessor, candidate revision id) -> reported as already applied
        let outcome3 = publish_lifecycle_cas(project_root, head_h.as_deref(), &cand1).unwrap();
        let outcome3_text = match &outcome3 {
            PublishOutcome::AlreadyApplied(reason) => {
                assert!(
                    reason.contains("already applied"),
                    "Row 3: outcome text must contain 'already applied', got: {reason}"
                );
                reason.clone()
            }
            other => panic!("Row 3: expected AlreadyApplied, got: {other:?}"),
        };
        assert_ne!(
            outcome3_text, outcome2_refusal_text,
            "Row 3: already applied outcome text must be distinct from Row 1's refusal text"
        );

        let bytes_after_row3 = std::fs::read(carrier_path(project_root, "causal")).unwrap();
        assert_eq!(
            bytes_after_row3, bytes_after_refusal,
            "Row 3: carrier bytes must be identical to immediately after winning publish"
        );

        let reloaded_after_row3 = load(project_root, "causal").unwrap().unwrap();
        assert_eq!(
            reloaded_after_row3.events.len(),
            2,
            "Row 3: no second event appended"
        );
        assert_eq!(reloaded_after_row3.epoch, 2, "Row 3: no second epoch bump");

        // Row 4: Publish with same predecessor H but different candidate revision id -> refused for moved head, text differs from Row 3
        let outcome4 = publish_lifecycle_cas(project_root, head_h.as_deref(), &cand2).unwrap();
        let outcome4_refusal_text = match &outcome4 {
            PublishOutcome::Refused(reason) => {
                assert!(
                    reason.contains("moved head"),
                    "Row 4: refusal reason must contain 'moved head', got: {reason}"
                );
                reason.clone()
            }
            other => panic!("Row 4: expected Refused, got: {other:?}"),
        };
        assert_ne!(
            outcome4_refusal_text, outcome3_text,
            "Row 4: refusal text must differ from Row 3's already-applied outcome"
        );

        // Row 5: Publish for slug A while lease held -> refused for held lease
        let lease = acquire_project_lease(project_root).unwrap().unwrap();
        let bytes_before_row5 = std::fs::read(carrier_path(project_root, "causal")).unwrap();

        let outcome5 = publish_lifecycle_cas(project_root, Some("evt-002"), &cand1).unwrap();
        let outcome5_refusal_text = match &outcome5 {
            PublishOutcome::Refused(reason) => {
                assert!(
                    reason.contains("held lease"),
                    "Row 5: refusal reason must contain 'held lease', got: {reason}"
                );
                reason.clone()
            }
            other => panic!("Row 5: expected Refused for held lease, got: {other:?}"),
        };
        assert_ne!(
            outcome5_refusal_text, outcome2_refusal_text,
            "Row 5: held lease refusal must be textually distinct from moved head refusal"
        );
        assert_ne!(
            outcome5_refusal_text, outcome4_refusal_text,
            "Row 5: held lease refusal must be textually distinct from Row 4 refusal"
        );

        let bytes_after_row5 = std::fs::read(carrier_path(project_root, "causal")).unwrap();
        assert_eq!(
            bytes_before_row5, bytes_after_row5,
            "Row 5: carrier bytes must remain unchanged"
        );

        // Row 6: Publish for slug A attempted after refused publish's lease released -> succeeds
        drop(lease);
        let res3 = fold_wi_update(&reloaded_after_w2, "wi-v3", Some("wi-v2-w1"));
        assert!(res3.accepted);
        let cand3 = res3.lifecycle;

        let outcome6 = publish_lifecycle_cas(project_root, Some("evt-002"), &cand3).unwrap();
        assert_eq!(
            outcome6,
            PublishOutcome::Applied,
            "Row 6: publish after lease release must succeed"
        );

        // Row 7: Publish for slug B attempted while slug A holds project lease -> refused for held lease; succeeds after release
        let lease2 = acquire_project_lease(project_root).unwrap().unwrap();
        let cand_b = fold_wi_create("slug-b", "wi-b1", "agentic-workflow");

        let outcome7_held = publish_lifecycle_cas(project_root, None, &cand_b).unwrap();
        match &outcome7_held {
            PublishOutcome::Refused(reason) => {
                assert!(
                    reason.contains("held lease"),
                    "Row 7: refusal while held must contain 'held lease', got: {reason}"
                );
                assert_eq!(
                    reason, &outcome5_refusal_text,
                    "Row 7: refusal reason must match Row 5's held lease refusal"
                );
            }
            other => panic!("Row 7: expected Refused for held lease, got: {other:?}"),
        }

        drop(lease2);
        let outcome7_released = publish_lifecycle_cas(project_root, None, &cand_b).unwrap();
        assert_eq!(
            outcome7_released,
            PublishOutcome::Applied,
            "Row 7: publish after project lease release must succeed"
        );

        // Row 8: Negative control: publish with expected_head = None against populated carrier -> refused for moved head
        let issue_a_new = change("wi-v1-new");
        let cand_a_new = initial_lifecycle(&issue_a_new); // slug is "causal", head event "evt-001", predecessor None
        let outcome8 = publish_lifecycle_cas(project_root, None, &cand_a_new).unwrap();
        match &outcome8 {
            PublishOutcome::Refused(reason) => {
                assert!(
                    reason.contains("moved head"),
                    "Row 8: refusal reason must contain 'moved head', got: {reason}"
                );
            }
            other => panic!("Row 8: expected Refused for moved head, got: {other:?}"),
        }

        let reloaded_after_row8 = load(project_root, "causal").unwrap().unwrap();
        assert_eq!(
            reloaded_after_row8.head_event_id,
            Some("evt-003".to_string()),
            "Row 8: populated carrier must not be replaced"
        );
    }

    #[test]
    fn test_record_writers_refusal_and_retry_outcomes() {
        let root = tempfile::tempdir().unwrap();
        let project_root = root.path();

        let issue_v1 = change("wi-v1");

        // 1. record_create against project whose publish lease is held returns Err with held-lease reason
        let lease_create = acquire_project_lease(project_root).unwrap().unwrap();
        let err_create = record_create(project_root, &issue_v1).unwrap_err();
        let err_create_text = err_create.to_string();
        assert!(
            err_create_text.contains("held lease"),
            "record_create while lease held must return Err containing 'held lease', got: {err_create_text}"
        );
        drop(lease_create);

        // First successful record_create after lease released
        record_create(project_root, &issue_v1).unwrap();
        let bytes_after_first_create = std::fs::read(carrier_path(project_root, "causal")).unwrap();

        // 2. record_create called twice with identical issue returns Ok(()) both times and carrier bytes are identical
        record_create(project_root, &issue_v1).unwrap();
        let bytes_after_second_create =
            std::fs::read(carrier_path(project_root, "causal")).unwrap();
        assert_eq!(
            bytes_after_first_create, bytes_after_second_create,
            "record_create second call with identical issue must leave carrier bytes identical"
        );

        // 3. record_update against project whose publish lease is held, for change WI whose body genuinely changed, returns Err with held-lease reason
        let issue_v2 = change("wi-v2");
        let lease_update = acquire_project_lease(project_root).unwrap().unwrap();
        let err_update = record_update(project_root, &issue_v1, &issue_v2).unwrap_err();
        let err_update_text = err_update.to_string();
        assert!(
            err_update_text.contains("held lease"),
            "record_update while lease held must return Err containing 'held lease', got: {err_update_text}"
        );
        drop(lease_update);

        // Successful record_update after lease released
        record_update(project_root, &issue_v1, &issue_v2).unwrap();
        let lc_after_update = load(project_root, "causal").unwrap().unwrap();
        assert_eq!(
            lc_after_update.head_event_id,
            Some("evt-002".to_string()),
            "record_update after lease release must publish updated lifecycle"
        );
    }

    #[test]
    fn test_canonical_wi_digest_six_sections_allowlist() {
        let base_body = "\
Preamble line to be ignored.

## Problem
Problem section text.

## Capability Alignment
Capability alignment text.

## Requirements
Requirements section text.

## Scope
Scope section text.

## Acceptance Criteria
Acceptance criteria text.

## Reference Context
Reference context text.
";

        // Row 1: base body vs base body with AW-authored ## Status section appended
        let body_row1 = format!("{base_body}\n\n## Status\nStatus section text.");
        assert_eq!(
            canonical_wi_digest(base_body),
            canonical_wi_digest(&body_row1),
            "Row 1: Appended ## Status section must not change canonical digest"
        );

        // Row 2: same pair, but with ## Status between ## Requirements and ## Scope
        let body_row2 = "\
Preamble line to be ignored.

## Problem
Problem section text.

## Capability Alignment
Capability alignment text.

## Requirements
Requirements section text.

## Status
Interleaved status section text.

## Scope
Scope section text.

## Acceptance Criteria
Acceptance criteria text.

## Reference Context
Reference context text.
";
        assert_eq!(
            canonical_wi_digest(base_body),
            canonical_wi_digest(body_row2),
            "Row 2: Interleaved ## Status section must not change canonical digest"
        );

        // Row 3: row 1 body with one word changed inside ## Problem
        let body_row3 =
            base_body.replace("Problem section text.", "Problem section text modified.");
        assert_ne!(
            canonical_wi_digest(base_body),
            canonical_wi_digest(&body_row3),
            "Row 3: Editing ## Problem must change canonical digest"
        );

        // Row 4: row 1 body with one word changed inside ## Reference Context
        let body_row4 = base_body.replace(
            "Reference context text.",
            "Reference context text modified.",
        );
        assert_ne!(
            canonical_wi_digest(base_body),
            canonical_wi_digest(&body_row4),
            "Row 4: Editing ## Reference Context must change canonical digest"
        );

        // Row 5: preamble added before first heading
        let body_row5 = format!("Extra preamble paragraph.\n\n{base_body}");
        assert_eq!(
            canonical_wi_digest(base_body),
            canonical_wi_digest(&body_row5),
            "Row 5: Adding preamble must not change canonical digest"
        );

        // Row 6: unstructured bodies (negative control)
        let body_v1 = "## Description\nInitial WI body.";
        let body_v2 = "## Description\nUpdated WI body v2.";
        assert_ne!(
            canonical_wi_digest(body_v1),
            canonical_wi_digest(body_v2),
            "Row 6: Unstructured bodies must yield distinct digests"
        );

        // Row 7: marker stripping on canonical section body
        let marker = "<!-- aw:projection\nversion: 1\n-->";
        let body_with_marker = format!("{base_body}\n\n{marker}");
        assert_eq!(
            canonical_wi_digest(base_body),
            canonical_wi_digest(&body_with_marker),
            "Row 7: Marker stripping must compose with section allowlist"
        );

        // Row 8: section order swapped
        let body_row8 = "\
Preamble line to be ignored.

## Problem
Problem section text.

## Capability Alignment
Capability alignment text.

## Scope
Scope section text.

## Requirements
Requirements section text.

## Acceptance Criteria
Acceptance criteria text.

## Reference Context
Reference context text.
";
        assert_ne!(
            canonical_wi_digest(base_body),
            canonical_wi_digest(body_row8),
            "Row 8: Swapping section order must change canonical digest"
        );

        // Prefix test: ## Problem Statement alongside six recognized sections
        let body_problem_statement =
            format!("{base_body}\n\n## Problem Statement\nProblem statement text.");
        let body_problem_statement_edited =
            format!("{base_body}\n\n## Problem Statement\nProblem statement text modified.");
        assert_eq!(
            canonical_wi_digest(&body_problem_statement),
            canonical_wi_digest(&body_problem_statement_edited),
            "Editing unrecognized heading ## Problem Statement must not change canonical digest"
        );

        // Prefix test: ## Problems alongside six recognized sections
        let body_problems = format!("{base_body}\n\n## Problems\nProblems text.");
        let body_problems_edited = format!("{base_body}\n\n## Problems\nProblems text modified.");
        assert_eq!(
            canonical_wi_digest(&body_problems),
            canonical_wi_digest(&body_problems_edited),
            "Editing unrecognized heading ## Problems must not change canonical digest"
        );

        // Duplicate section test: ## Scope carried twice
        let body_double_scope = "\
Preamble line to be ignored.

## Problem
Problem section text.

## Capability Alignment
Capability alignment text.

## Requirements
Requirements section text.

## Scope
First scope section text.

## Acceptance Criteria
Acceptance criteria text.

## Reference Context
Reference context text.

## Scope
Second scope section text.
";
        let body_double_scope_edited = body_double_scope.replace(
            "Second scope section text.",
            "Second scope section text modified.",
        );
        assert_ne!(
            canonical_wi_digest(body_double_scope),
            canonical_wi_digest(&body_double_scope_edited),
            "Editing text under second occurrence of ## Scope must change canonical digest"
        );
    }

    #[test]
    fn test_wi_contract_canonical_snapshot_measurements() {
        let base_body = "\
## Problem
Problem section text.

## Capability Alignment
Capability alignment text.

## Requirements
Requirements section text.

## Scope
Scope section text.

## Acceptance Criteria
Acceptance criteria text.

## Reference Context
Reference context text.
";

        let make_base_issue = || Issue {
            issue_type: IssueType::Change,
            title: "Bounded change".to_string(),
            state: IssueState::Open,
            id: Some("id-1".to_string()),
            github_id: Some(101),
            gitlab_id: None,
            url: Some("https://github.com/org/repo/issues/101".to_string()),
            author: Some("alice".to_string()),
            labels: vec![
                "type:change".to_string(),
                "app:demo".to_string(),
                "epic:10".to_string(),
                "depends-on:9".to_string(),
                "depends-on:12".to_string(),
            ],
            created_at: Some("2026-08-01T00:00:00Z".to_string()),
            updated_at: Some("2026-08-01T00:00:00Z".to_string()),
            slug: "bounded-change".to_string(),
            body: base_body.to_string(),
            related: Vec::new(),
            implements: Vec::new(),
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
            validation_errors: Vec::new(),
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        };

        // 1. Two Issues alike in body and labels, differing only in title ("Bounded change" vs "Bounded change, revised")
        let issue1 = make_base_issue();
        let mut issue1_revised = make_base_issue();
        issue1_revised.title = "Bounded change, revised".to_string();

        let lc1 = initial_lifecycle(&issue1);
        let lc1_revised = initial_lifecycle(&issue1_revised);

        let digest1 = lc1
            .active_revisions
            .get(&ArtifactKind::Wi)
            .unwrap()
            .as_ref()
            .unwrap()
            .digest
            .clone();
        let digest1_revised = lc1_revised
            .active_revisions
            .get(&ArtifactKind::Wi)
            .unwrap()
            .as_ref()
            .unwrap()
            .digest
            .clone();

        assert_ne!(
            digest1, digest1_revised,
            "Measurement 1: Two issues differing only in title must produce different WI revision digests"
        );

        // 2. One Issue with ["type:change", "app:demo", "epic:10", "depends-on:9", "depends-on:12"], and three variants: permuted, duplicated, epic:20
        let orig_snap = CanonicalWiSnapshot::from_issue(&issue1);

        let mut issue2_permuted = make_base_issue();
        issue2_permuted.labels = vec![
            "depends-on:12".to_string(),
            "epic:10".to_string(),
            "app:demo".to_string(),
            "depends-on:9".to_string(),
            "type:change".to_string(),
        ];
        let permuted_snap = CanonicalWiSnapshot::from_issue(&issue2_permuted);

        let mut issue2_duplicated = make_base_issue();
        issue2_duplicated.labels = vec![
            "type:change".to_string(),
            "app:demo".to_string(),
            "epic:10".to_string(),
            "depends-on:9".to_string(),
            "depends-on:12".to_string(),
            "depends-on:12".to_string(),
        ];
        let duplicated_snap = CanonicalWiSnapshot::from_issue(&issue2_duplicated);

        let mut issue2_epic20 = make_base_issue();
        issue2_epic20.labels = vec![
            "type:change".to_string(),
            "app:demo".to_string(),
            "epic:20".to_string(),
            "depends-on:9".to_string(),
            "depends-on:12".to_string(),
        ];
        let epic20_snap = CanonicalWiSnapshot::from_issue(&issue2_epic20);

        assert_eq!(
            orig_snap.digest(),
            permuted_snap.digest(),
            "Measurement 2: Permuted ownership labels must produce the same WI digest"
        );
        assert_eq!(
            orig_snap.digest(),
            duplicated_snap.digest(),
            "Measurement 2: Duplicated depends-on label must produce the same WI digest"
        );
        assert_ne!(
            orig_snap.digest(),
            epic20_snap.digest(),
            "Measurement 2: Changed epic parent must produce a different WI digest"
        );

        // 3. Two Issues alike but for state: Closed, priority:p0 -> priority:p2, phase: Some("impl"), updated_at, url/author/github_id
        let mut issue3 = make_base_issue();
        issue3.state = IssueState::Closed;
        issue3.labels.push("priority:p2".to_string());
        issue3.phase = Some("impl".to_string());
        issue3.updated_at = Some("2026-08-02T12:00:00Z".to_string());
        issue3.url = Some("https://example.com/3".to_string());
        issue3.author = Some("bob".to_string());
        issue3.github_id = Some(999);

        let snap1 = CanonicalWiSnapshot::from_issue(&issue1);
        let snap3 = CanonicalWiSnapshot::from_issue(&issue3);
        assert_eq!(
            snap1.digest(),
            snap3.digest(),
            "Measurement 3: Editing state, priority, phase, updated_at, url, author, github_id must leave WI digest unchanged"
        );

        // 4. Committed lifecycle + TrackerObservation built the way projection_for_issue builds one from an Issue whose title alone was edited on tracker (negative control)
        let observation_title_edited = TrackerObservation::from_issue(&issue1_revised);
        let decision4 = decide_projection(&lc1, &observation_title_edited);
        assert!(
            !decision4.accepted,
            "Measurement 4: decide_projection must fail when tracker title was edited"
        );
        assert!(
            decision4
                .refusal_reason
                .as_ref()
                .map_or(false, |r| r.contains("digest")),
            "Measurement 4: refusal reason must name the WI digest mismatch"
        );

        // 5. The same lifecycle, and the observation built the same way from the unedited Issue
        let observation_unedited = TrackerObservation::from_issue(&issue1);
        let decision5 = decide_projection(&lc1, &observation_unedited);
        assert!(
            decision5.accepted,
            "Measurement 5: decide_projection must accept when tracker issue is unedited"
        );

        // 6. Refusal holds for a lifecycle whose committed title is empty and whose committed epic and dependency lists are empty, created through the path that observes ownership
        let issue_empty_title_no_ownership = Issue {
            issue_type: IssueType::Change,
            title: "".to_string(),
            state: IssueState::Open,
            id: Some("empty-title-slug".to_string()),
            github_id: None,
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec!["app:agentic-workflow".to_string()],
            created_at: None,
            updated_at: None,
            slug: "empty-title-slug".to_string(),
            body: issue1.body.clone(),
            related: Vec::new(),
            implements: Vec::new(),
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
            validation_errors: Vec::new(),
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        };
        let lc_empty_title = fold_wi_create_from_issue(&issue_empty_title_no_ownership);
        assert!(lc_empty_title.wi_snapshot().unwrap().ownership_observed);

        let mut issue_with_added_title = issue_empty_title_no_ownership.clone();
        issue_with_added_title.title = "Added Title".to_string();
        let obs_added_title = TrackerObservation::from_issue(&issue_with_added_title);

        let decision6 = decide_projection(&lc_empty_title, &obs_added_title);
        assert!(
            !decision6.accepted,
            "Measurement 6: decide_projection must fail when tracker title was edited on a lifecycle with empty committed title created via observed path"
        );
        assert!(
            decision6
                .refusal_reason
                .as_ref()
                .map_or(false, |r| r.contains("digest")),
            "Measurement 6: refusal reason must name the WI digest mismatch"
        );
    }

    #[test]
    fn test_wi_contract_drift_invalidation_measurements_1_to_6() {
        let base_body = "\
## Problem
Problem description.

## Capability Alignment
Alignment info.

## Requirements
Requirements text.

## Scope
Scope text.

## Acceptance Criteria
Criteria text.

## Reference Context
Reference info.
";
        let mut issue = change(base_body);
        issue.labels = vec![
            "app:agentic-workflow".to_string(),
            "priority:p1".to_string(),
        ];
        issue.phase = Some("created".to_string());
        let lc = fold_wi_create_from_issue(&issue);

        // Row 1: Observation whose canonical snapshot digest differs from committed WI revision
        let drifted_body = base_body.replace(
            "Problem description.",
            "Problem description edited on tracker.",
        );
        let obs_drifted = TrackerObservation::from_issue(&change(&drifted_body));
        let decision_row1 = decide_projection(&lc, &obs_drifted);

        assert!(!decision_row1.accepted, "Row 1: accepted must be false");
        assert!(decision_row1.drift, "Row 1: drift must be true");
        assert_eq!(
            decision_row1.remediation.len(),
            1,
            "Row 1: remediation must carry single obligation"
        );
        assert_eq!(
            decision_row1.remediation[0].command,
            format!("aw wi change {}", lc.slug),
            "Row 1: remediation command must be aw wi change <slug>"
        );
        assert_eq!(
            decision_row1.remediation[0].owner,
            OwnerVocabulary::Wi,
            "Row 1: remediation owner must be Wi"
        );

        // Row 2: (negative control) Observation whose canonical snapshot digest matches
        let obs_matching = TrackerObservation::from_issue(&issue);
        let decision_row2 = decide_projection(&lc, &obs_matching);
        assert!(decision_row2.accepted, "Row 2: accepted must be true");
        assert!(!decision_row2.drift, "Row 2: drift must be false");
        assert!(
            decision_row2.remediation.is_empty(),
            "Row 2: remediation must be empty"
        );

        // Row 3: Decision from row 1 names Ec, Td, and Cb as invalidated — exact set from transitive_invalidation_kinds
        assert_eq!(
            decision_row1.invalidated_kinds,
            transitive_invalidation_kinds(ArtifactKind::Wi),
            "Row 3: invalidated_kinds must match transitive_invalidation_kinds(ArtifactKind::Wi)"
        );
        assert_eq!(
            decision_row1.invalidated_kinds,
            vec![ArtifactKind::Ec, ArtifactKind::Td, ArtifactKind::Cb],
            "Row 3: invalidated_kinds must be Ec, Td, Cb and not name Wi"
        );
        assert!(
            !decision_row1.invalidated_kinds.contains(&ArtifactKind::Wi),
            "Row 3: must not name Wi"
        );

        // Row 4: (negative control) Observation differing only in excluded R2 fields
        let mut issue_excluded_diff = issue.clone();
        issue_excluded_diff.phase = Some("in_progress".to_string());
        issue_excluded_diff.labels = vec![
            "app:agentic-workflow".to_string(),
            "priority:p0".to_string(),
        ];
        issue_excluded_diff.updated_at = Some("2026-08-08T12:00:00Z".to_string());
        let comment_marker = "\n\n<!-- aw:projection\nversion: 1\n-->";
        issue_excluded_diff.body = format!("{base_body}{comment_marker}");
        let obs_excluded_diff = TrackerObservation::from_issue(&issue_excluded_diff);
        let decision_row4 = decide_projection(&lc, &obs_excluded_diff);
        assert!(decision_row4.accepted, "Row 4: accepted must be true");
        assert!(!decision_row4.drift, "Row 4: drift must be false");
        assert!(
            decision_row4.remediation.is_empty(),
            "Row 4: remediation must be empty"
        );

        // Row 5: route_failure(FailureOwnership::WiDrift, slug, current_command)
        let routed_remediation = route_failure(FailureOwnership::WiDrift, &lc.slug, "aw ec check");
        assert_eq!(
            routed_remediation.command,
            format!("aw wi change {}", lc.slug),
            "Row 5: route_failure for WiDrift must return aw wi change <slug>"
        );
        assert_eq!(
            routed_remediation.owner,
            OwnerVocabulary::Wi,
            "Row 5: route_failure owner must be Wi"
        );

        // Row 6: (negative control) Non-terminal lifecycle whose issue was closed by hand, with no digest drift
        let obs_closed_no_drift =
            TrackerObservation::from_issue(&issue).with_state(IssueState::Closed);
        let decision_row6 = decide_projection(&lc, &obs_closed_no_drift);
        assert!(decision_row6.accepted, "Row 6: accepted must be true");
        assert!(decision_row6.drift, "Row 6: drift must be true");
        assert_eq!(
            decision_row6.remediation.len(),
            2,
            "Row 6: remediation must be 2 steps"
        );
        assert_eq!(
            decision_row6.remediation[0].command,
            format!("aw wi update {} --state open", lc.slug),
            "Row 6: first step must be aw wi update <slug> --state open"
        );
        assert_eq!(decision_row6.remediation[0].owner, OwnerVocabulary::Wi);
        assert_eq!(
            decision_row6.remediation[1], lc.next,
            "Row 6: second step must be lifecycle's own next obligation"
        );
    }

    #[test]
    fn test_wi_contract_change_leaf_measurements_1_to_5() {
        let root = tempfile::tempdir().unwrap();
        let base_body = "\
## Problem
Problem description.

## Capability Alignment
Alignment info.

## Requirements
Requirements text.

## Scope
Scope text.

## Acceptance Criteria
Criteria text.

## Reference Context
Reference info.
";
        let issue = change(base_body);
        let slug = issue.slug.clone();

        // Measurement 1: a type:change issue whose slug has no durable carrier under the project root
        let carrier_file = carrier_path(root.path(), &slug);
        assert!(
            !carrier_file.exists(),
            "carrier must not exist before first call"
        );

        let proj1 =
            run_change_leaf(root.path(), &issue).expect("run_change_leaf must succeed on row 1");
        assert!(carrier_file.exists(), "carrier must be created on row 1");

        let carrier1 = load(root.path(), &slug)
            .unwrap()
            .expect("carrier must be loadable after row 1");
        assert_eq!(carrier1.epoch, 1, "carrier epoch must be 1 on row 1");
        assert_eq!(
            carrier1.events.len(),
            1,
            "carrier events must hold exactly one WiCreate"
        );
        assert_eq!(
            carrier1.events[0].kind,
            LifecycleEventKind::WiCreate,
            "event must be WiCreate"
        );

        assert!(
            !proj1["wi_revision"].is_null(),
            "rendered projection must have non-null wi_revision on row 1"
        );
        assert_eq!(proj1["ledger"]["epoch"], 1);
        assert_eq!(
            proj1["ledger"]["head_event_id"],
            serde_json::json!(carrier1.head_event_id)
        );

        let head1 = carrier1.head_event_id.clone();
        let bytes1 = std::fs::read(&carrier_file).unwrap();

        // Measurement 2: the same issue and carrier, called a second time with tracker text untouched
        let proj2 =
            run_change_leaf(root.path(), &issue).expect("run_change_leaf must succeed on row 2");
        let carrier2 = load(root.path(), &slug)
            .unwrap()
            .expect("carrier must be loadable on row 2");
        let bytes2 = std::fs::read(&carrier_file).unwrap();

        assert_eq!(
            carrier2.head_event_id, head1,
            "head_event_id must be identical on row 2"
        );
        assert_eq!(carrier2.epoch, 1, "epoch must be identical on row 2");
        assert_eq!(
            carrier2.events.len(),
            1,
            "event count must be identical on row 2"
        );
        assert_eq!(
            bytes1, bytes2,
            "carrier file bytes must be untouched on row 2"
        );
        assert_eq!(
            proj1, proj2,
            "emitted projection must equal row 1 projection"
        );
        assert_eq!(
            proj2,
            projection_for_issue(root.path(), &issue),
            "emitted projection must equal projection_for_issue"
        );

        // Measurement 3: a carrier whose committed WI contract has drifted from tracker body
        let drifted_body = base_body.replace(
            "Problem description.",
            "Problem description edited on tracker.",
        );
        let drifted_issue = change(&drifted_body);
        let proj3 = run_change_leaf(root.path(), &drifted_issue)
            .expect("run_change_leaf must succeed on row 3");
        let carrier3 = load(root.path(), &slug)
            .unwrap()
            .expect("carrier must be loadable on row 3");
        let bytes3 = std::fs::read(&carrier_file).unwrap();

        assert_eq!(
            bytes2, bytes3,
            "carrier file bytes must be untouched on row 3"
        );
        assert_eq!(carrier3.epoch, 1, "epoch must remain 1 on row 3");
        assert_eq!(
            carrier3.events.len(),
            1,
            "event count must remain 1 on row 3"
        );
        assert_eq!(
            proj3["drift"],
            serde_json::json!(true),
            "projection must report drift: true"
        );
        assert_eq!(
            proj3["remediation"].as_array().unwrap().len(),
            1,
            "remediation must have 1 element"
        );
        assert_eq!(
            proj3["remediation"][0]["command"],
            serde_json::json!(format!("aw wi change {slug}")),
            "remediation command must be aw wi change <slug>"
        );
        assert_eq!(
            proj3["remediation"][0]["owner"],
            serde_json::json!("wi"),
            "remediation owner must be wi"
        );
        assert_eq!(
            proj3,
            projection_for_issue(root.path(), &drifted_issue),
            "emitted projection must equal projection_for_issue for drifted issue"
        );

        // Measurement 4: the set of paths under project root created or modified in rows 1 through 3
        fn collect_files(dir: &Path) -> Vec<PathBuf> {
            let mut files = Vec::new();
            if dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            files.extend(collect_files(&path));
                        } else {
                            files.push(path);
                        }
                    }
                }
            }
            files.sort();
            files
        }

        let carrier_file = carrier_path(root.path(), &slug);
        let project_files = collect_files(root.path());
        assert!(
            project_files.is_empty(),
            "no files created directly under project root (git/refs/index/tracker untouched)"
        );

        let runtime_files = collect_files(&crate::shared::workspace::workspace_runtime_path(
            root.path(),
        ));
        assert_eq!(
            runtime_files,
            vec![carrier_file.clone()],
            "exactly carrier_path and nothing else created under workspace runtime root"
        );

        // Measurement 5: non-change types do not get carriers created
        let mut spike_issue = change("spike body");
        spike_issue.issue_type = IssueType::Spike;
        spike_issue.slug = "3363-spike".to_string();

        let proj_spike = run_change_leaf(root.path(), &spike_issue).unwrap();
        assert!(proj_spike["wi_revision"].is_null());
        assert!(!carrier_path(root.path(), &spike_issue.slug).exists());
    }

    fn complete_change_body() -> &'static str {
        "\
## Problem
Problem statement describing the change.

## Capability Alignment
Capability: Test
Capability Gap: Missing test.
Progress Evidence: Evidence is recorded.

## Requirements
- R1: Deliver the test gate.

## Scope
### In Scope
- Deliver the test gate.

### Out of Scope
- Unrelated features.

## Acceptance Criteria
- AC1: R1 is delivered and verified.

## Verification Inventory
| Requirement | Gate | Oracle | Depends On |
|-------------|------|--------|------------|
| R1 | `cargo test` | Pass | - |

## Reference Context
### Related Specs
| Spec | Relevance |
|------|-----------|
| spec.md | source |

### Spec Plan
| Spec ID | Action | Main Spec Ref |
|---------|--------|---------------|
| wi-draft | update | spec.md |
"
    }

    #[test]
    fn wi_contract_test_gate_measurement_1_complete_change() {
        let root = tempfile::tempdir().unwrap();
        let issue = change(complete_change_body());
        let slug = issue.slug.clone();

        let proj =
            run_test_leaf(root.path(), &issue, &[issue.clone()]).expect("run_test_leaf succeeds");
        let carrier = load(root.path(), &slug).unwrap().expect("carrier loadable");

        assert_eq!(
            carrier.evidence.len(),
            1,
            "must hold exactly 1 EvidenceBinding"
        );
        let binding = &carrier.evidence[0];
        assert_eq!(binding.verifier, "wi-test");
        assert!(binding.passed, "passed must be true for complete change");
        assert_eq!(binding.bound_tuple, carrier.active_digest_tuple());

        assert_eq!(carrier.next.command, format!("aw wi review {slug}"));
        assert_eq!(carrier.next.owner, OwnerVocabulary::Wi);
        assert_eq!(carrier.epoch, 1);
        assert_eq!(carrier.events.len(), 1);
        assert!(!carrier.terminal);

        assert_eq!(
            proj["next"]["command"],
            serde_json::json!(format!("aw wi review {slug}"))
        );
    }

    #[test]
    fn wi_contract_test_gate_measurement_2_failing_dimensions() {
        // 2a: Missing ## Requirements
        {
            let root = tempfile::tempdir().unwrap();
            let no_reqs_body = "\
## Problem
Problem description.

## Scope
### In Scope
- Scope.
### Out of Scope
- None.
";
            let mut issue_no_reqs = change(no_reqs_body);
            issue_no_reqs.slug = "2a".to_string();
            issue_no_reqs.id = Some("2a".to_string());
            let proj_no_reqs =
                run_test_leaf(root.path(), &issue_no_reqs, &[issue_no_reqs.clone()]).unwrap();
            let carrier_no_reqs = load(root.path(), &issue_no_reqs.slug).unwrap().unwrap();
            assert!(valid_persisted_lifecycle(
                &carrier_no_reqs,
                &issue_no_reqs.slug
            ));
            assert!(!proj_no_reqs["wi_revision"].is_null());
            assert_eq!(
                proj_no_reqs["next"]["command"],
                serde_json::json!("aw wi change 2a")
            );
            assert_eq!(proj_no_reqs["next"]["owner"], serde_json::json!("wi"));
            assert_eq!(
                proj_no_reqs["evidence"][0]["passed"],
                serde_json::json!(false)
            );
            assert_eq!(carrier_no_reqs.evidence.len(), 1);
            assert!(!carrier_no_reqs.evidence[0].passed);
            assert!(carrier_no_reqs.evidence[0]
                .summary
                .contains("section_structure"));
        }

        // 2b: Requirement item not matching ^R\d+:
        {
            let root = tempfile::tempdir().unwrap();
            let bad_rid_body = complete_change_body().replace("- R1:", "- InvalidItem:");
            let mut issue_bad_rid = change(&bad_rid_body);
            issue_bad_rid.slug = "2b".to_string();
            issue_bad_rid.id = Some("2b".to_string());
            let proj_bad_rid =
                run_test_leaf(root.path(), &issue_bad_rid, &[issue_bad_rid.clone()]).unwrap();
            let carrier_bad_rid = load(root.path(), &issue_bad_rid.slug).unwrap().unwrap();
            assert!(valid_persisted_lifecycle(
                &carrier_bad_rid,
                &issue_bad_rid.slug
            ));
            assert!(!proj_bad_rid["wi_revision"].is_null());
            assert_eq!(
                proj_bad_rid["next"]["command"],
                serde_json::json!("aw wi change 2b")
            );
            assert_eq!(proj_bad_rid["next"]["owner"], serde_json::json!("wi"));
            assert_eq!(
                proj_bad_rid["evidence"][0]["passed"],
                serde_json::json!(false)
            );
            assert_eq!(carrier_bad_rid.evidence.len(), 1);
            assert!(!carrier_bad_rid.evidence[0].passed);
            assert!(carrier_bad_rid.evidence[0]
                .summary
                .contains("section_structure"));
        }

        // 2c: Body looks_too_large_for_atomic_wi
        {
            let root = tempfile::tempdir().unwrap();
            let large_body = complete_change_body().replace(
                "- Deliver the test gate.",
                "- Rewrite all codebases from scratch across the fleet.",
            );
            let mut issue_large = change(&large_body);
            issue_large.slug = "2c".to_string();
            issue_large.id = Some("2c".to_string());
            let proj_large =
                run_test_leaf(root.path(), &issue_large, &[issue_large.clone()]).unwrap();
            let carrier_large = load(root.path(), &issue_large.slug).unwrap().unwrap();
            assert!(valid_persisted_lifecycle(&carrier_large, &issue_large.slug));
            assert!(!proj_large["wi_revision"].is_null());
            assert_eq!(
                proj_large["next"]["command"],
                serde_json::json!("aw wi change 2c")
            );
            assert_eq!(proj_large["next"]["owner"], serde_json::json!("wi"));
            assert_eq!(
                proj_large["evidence"][0]["passed"],
                serde_json::json!(false)
            );
            assert_eq!(carrier_large.evidence.len(), 1);
            assert!(!carrier_large.evidence[0].passed);
            assert!(carrier_large.evidence[0].summary.contains("boundedness"));
        }

        // 2d: Depends On reference to non-existent WI
        {
            let root = tempfile::tempdir().unwrap();
            let missing_ref_body = complete_change_body().replace(
                "## Requirements",
                "## Requirements\nDepends On: #nonexistent-wi-999\n",
            );
            let mut issue_missing_ref = change(&missing_ref_body);
            issue_missing_ref.slug = "2d".to_string();
            issue_missing_ref.id = Some("2d".to_string());
            issue_missing_ref
                .labels
                .push("depends-on:nonexistent-wi-999".to_string());
            let proj_missing_ref = run_test_leaf(
                root.path(),
                &issue_missing_ref,
                &[issue_missing_ref.clone()],
            )
            .unwrap();
            let carrier_missing_ref = load(root.path(), &issue_missing_ref.slug).unwrap().unwrap();
            assert!(valid_persisted_lifecycle(
                &carrier_missing_ref,
                &issue_missing_ref.slug
            ));
            assert!(!proj_missing_ref["wi_revision"].is_null());
            assert_eq!(
                proj_missing_ref["next"]["command"],
                serde_json::json!("aw wi change 2d")
            );
            assert_eq!(proj_missing_ref["next"]["owner"], serde_json::json!("wi"));
            assert_eq!(
                proj_missing_ref["evidence"][0]["passed"],
                serde_json::json!(false)
            );
            assert_eq!(carrier_missing_ref.evidence.len(), 1);
            assert!(!carrier_missing_ref.evidence[0].passed);
            assert!(carrier_missing_ref.evidence[0]
                .summary
                .contains("references"));
        }
    }

    #[test]
    fn wi_contract_test_gate_measurement_3_uncovered_requirement() {
        let root = tempfile::tempdir().unwrap();
        let partial_cov_body = "\
## Problem
Problem description.

## Capability Alignment
Capability: Test
Capability Gap: Gap.
Progress Evidence: Evidence.

## Requirements
- R1: First requirement.
- R2: Second requirement.

## Scope
### In Scope
- Scope.
### Out of Scope
- None.

## Acceptance Criteria
- AC1: R1 is delivered.

## Verification Inventory
| Requirement | Gate | Oracle | Depends On |
|-------------|------|--------|------------|
| R1 | `cargo test` | Pass | - |

## Reference Context
### Related Specs
| Spec | Relevance |
|------|-----------|
| spec.md | source |

### Spec Plan
| Spec ID | Action | Main Spec Ref |
|---------|--------|---------------|
| wi-draft | update | spec.md |
";
        let issue = change(partial_cov_body);
        let proj = run_test_leaf(root.path(), &issue, &[issue.clone()]).unwrap();
        let carrier = load(root.path(), &issue.slug).unwrap().unwrap();

        assert!(valid_persisted_lifecycle(&carrier, &issue.slug));
        assert!(!proj["wi_revision"].is_null());
        assert_eq!(
            proj["next"]["command"],
            serde_json::json!(format!("aw wi change {}", issue.slug))
        );
        assert_eq!(proj["next"]["owner"], serde_json::json!("wi"));
        assert_eq!(proj["evidence"][0]["passed"], serde_json::json!(false));
        assert_eq!(carrier.evidence.len(), 1);
        assert!(
            !carrier.evidence[0].passed,
            "passed must be false when R2 is uncovered"
        );
        assert!(
            carrier.evidence[0].summary.contains("R2"),
            "summary must name uncovered requirement id R2, got: {}",
            carrier.evidence[0].summary
        );
    }

    #[test]
    fn wi_contract_test_gate_measurement_4_marker_block_churn_preserves_evidence() {
        let root = tempfile::tempdir().unwrap();
        let before_issue = change(complete_change_body());
        let slug = before_issue.slug.clone();

        record_create(root.path(), &before_issue).unwrap();
        let proj_before =
            run_test_leaf(root.path(), &before_issue, &[before_issue.clone()]).unwrap();
        assert!(!proj_before["wi_revision"].is_null());

        let carrier1 = load(root.path(), &slug).unwrap().unwrap();
        assert_eq!(carrier1.evidence.len(), 1);
        assert!(carrier1.evidence[0].passed);

        let churned_body = format!(
            "{}\n\n<!-- aw:loop-state\nversion: 1\n-->\n",
            complete_change_body()
        );
        let churned_issue = change(&churned_body);

        record_update(root.path(), &before_issue, &churned_issue).unwrap();

        let carrier2 = load(root.path(), &slug).unwrap().unwrap();
        assert_eq!(
            carrier2.evidence.len(),
            1,
            "evidence must be preserved across marker churn"
        );
        assert_eq!(carrier2.evidence[0].verifier, "wi-test");
        assert!(
            carrier2.evidence[0].passed,
            "outcome must remain passed: true"
        );
    }

    #[test]
    fn wi_contract_test_gate_measurement_4b_drifted_tracker_row_uses_committed_snapshot() {
        let root = tempfile::tempdir().unwrap();
        let issue = change(complete_change_body());
        let slug = issue.slug.clone();

        record_create(root.path(), &issue).unwrap();
        let proj_before = run_test_leaf(root.path(), &issue, &[issue.clone()]).unwrap();
        assert_eq!(
            proj_before["evidence"][0]["passed"],
            serde_json::json!(true)
        );

        let mut drifted = issue.clone();
        drifted.labels.push("depends-on:no-such-wi-999".to_string());

        let drifted_outcome = validate_canonical_wi_snapshot(
            &CanonicalWiSnapshot::from_issue(&drifted),
            &[drifted.clone()],
        );
        assert!(!drifted_outcome.passed);
        assert!(
            drifted_outcome.summary.contains("references"),
            "expected summary to contain `references`, got: {}",
            drifted_outcome.summary
        );

        let proj_after = run_test_leaf(root.path(), &drifted, &[drifted.clone()]).unwrap();
        assert_eq!(proj_after["evidence"][0]["passed"], serde_json::json!(true));
        assert_eq!(
            proj_after["next"]["command"],
            serde_json::json!(format!("aw wi review {slug}"))
        );
    }

    #[test]
    fn wi_contract_test_gate_measurement_5_problem_prose_change_evicts_evidence() {
        let root = tempfile::tempdir().unwrap();
        let before_issue = change(complete_change_body());
        let slug = before_issue.slug.clone();

        record_create(root.path(), &before_issue).unwrap();
        let proj_before =
            run_test_leaf(root.path(), &before_issue, &[before_issue.clone()]).unwrap();
        assert!(!proj_before["wi_revision"].is_null());

        let carrier1 = load(root.path(), &slug).unwrap().unwrap();
        assert_eq!(carrier1.evidence.len(), 1);
        assert!(carrier1.evidence[0].passed);

        let edited_body = complete_change_body().replace(
            "Problem statement describing the change.",
            "Edited problem statement prose.",
        );
        let edited_issue = change(&edited_body);

        record_update(root.path(), &before_issue, &edited_issue).unwrap();

        let carrier2 = load(root.path(), &slug).unwrap().unwrap();
        assert!(
            carrier2.evidence.is_empty(),
            "active evidence must be evicted after problem prose change"
        );
        assert_eq!(carrier2.invalidations.len(), 1);
        assert!(carrier2.invalidations[0]
            .evicted_evidence_verifiers
            .contains(&"wi-test".to_string()));
    }

    #[test]
    fn wi_contract_test_gate_measurement_6_projection_after_green_test() {
        let root = tempfile::tempdir().unwrap();
        let issue = change(complete_change_body());
        let slug = issue.slug.clone();

        record_create(root.path(), &issue).unwrap();
        let carrier_before = load(root.path(), &slug).unwrap().unwrap();
        let rev_before = carrier_before.active_revisions[&ArtifactKind::Wi].clone();
        let head_before = carrier_before.head_event_id.clone();

        let proj = run_test_leaf(root.path(), &issue, &[issue.clone()]).unwrap();
        let carrier_after = load(root.path(), &slug).unwrap().unwrap();

        assert_eq!(
            proj["next"]["command"],
            serde_json::json!(format!("aw wi review {slug}"))
        );
        assert_eq!(proj["next"]["owner"], serde_json::json!("wi"));

        let rev_after = carrier_after.active_revisions[&ArtifactKind::Wi].clone();
        let head_after = carrier_after.head_event_id.clone();

        assert_eq!(
            rev_before, rev_after,
            "active_revisions[Wi] must be byte-identical"
        );
        assert_eq!(
            head_before, head_after,
            "head_event_id must be byte-identical"
        );
        assert!(!carrier_after.terminal, "terminal must remain false");
    }

    #[test]
    fn wi_contract_test_gate_measurement_7_refusals() {
        let root = tempfile::tempdir().unwrap();

        assert!(ensure_change_id("").is_err());
        assert!(ensure_change_id("   ").is_err());

        let mut spike_issue = change("spike body");
        spike_issue.issue_type = IssueType::Spike;
        spike_issue.slug = "3363-spike".to_string();

        let err_spike = ensure_change_issue(&spike_issue, "test").unwrap_err();
        assert!(err_spike.to_string().contains("has immutable type `spike`"));

        let mut report_issue = change("report body");
        report_issue.issue_type = IssueType::Report;
        report_issue.slug = "3363-report".to_string();

        let err_report = ensure_change_issue(&report_issue, "test").unwrap_err();
        assert!(err_report
            .to_string()
            .contains("has immutable type `report`"));

        assert!(!carrier_path(root.path(), &spike_issue.slug).exists());
        assert!(!carrier_path(root.path(), &report_issue.slug).exists());
    }

    #[test]
    fn wi_contract_review_digest_measurement_1_accepted_payload() {
        let root = tempfile::tempdir().unwrap();
        let issue = change(complete_change_body());
        let slug = issue.slug.clone();

        record_create(root.path(), &issue).unwrap();
        run_test_leaf(root.path(), &issue, &[issue.clone()]).unwrap();

        let evidence_path = root.path().join("evidence.json");
        std::fs::write(
            &evidence_path,
            r#"{"reviewer_kind":"agent","decision":"accepted","summary":"LGTM"}"#,
        )
        .unwrap();

        let proj = run_review_leaf(root.path(), &issue, &evidence_path).unwrap();
        let carrier = load(root.path(), &slug).unwrap().unwrap();

        assert_eq!(carrier.evidence.len(), 2);
        let review_binding = carrier
            .evidence
            .iter()
            .find(|b| b.verifier == "wi-review")
            .unwrap();
        assert!(review_binding.passed);
        assert_eq!(review_binding.summary, "LGTM");
        assert_eq!(review_binding.bound_tuple, carrier.active_digest_tuple());
        let test_binding = carrier
            .evidence
            .iter()
            .find(|b| b.verifier == "wi-test")
            .unwrap();
        assert_eq!(review_binding.bound_tuple, test_binding.bound_tuple);

        assert_eq!(proj["next"]["command"], format!("aw wi commit {slug}"));
    }

    #[test]
    fn wi_contract_review_digest_measurement_2_refusals_missing_failing_stale_test() {
        let issue = change(complete_change_body());
        let slug = issue.slug.clone();

        // 2a: Missing wi-test binding
        let root_missing = tempfile::tempdir().unwrap();
        let evidence_path_a = root_missing.path().join("evidence.json");
        std::fs::write(
            &evidence_path_a,
            r#"{"reviewer_kind":"agent","decision":"accepted","summary":"LGTM"}"#,
        )
        .unwrap();

        record_create(root_missing.path(), &issue).unwrap();
        let err_missing =
            run_review_leaf(root_missing.path(), &issue, &evidence_path_a).unwrap_err();
        assert!(err_missing.to_string().contains("missing"));
        assert!(err_missing.to_string().contains("wi-test"));
        let carrier_missing = load(root_missing.path(), &slug).unwrap().unwrap();
        assert!(carrier_missing
            .evidence
            .iter()
            .all(|b| b.verifier != "wi-review"));

        // 2b: Failing wi-test binding
        let root_failing = tempfile::tempdir().unwrap();
        let evidence_path_b = root_failing.path().join("evidence.json");
        std::fs::write(
            &evidence_path_b,
            r#"{"reviewer_kind":"agent","decision":"accepted","summary":"LGTM"}"#,
        )
        .unwrap();

        let bad_reqs_body = "\
## Problem
Problem statement.

## Scope
### In Scope
- Scope.
";
        let bad_issue = change(bad_reqs_body);
        let bad_slug = bad_issue.slug.clone();
        record_create(root_failing.path(), &bad_issue).unwrap();
        run_test_leaf(root_failing.path(), &bad_issue, &[bad_issue.clone()]).unwrap();
        let err_failing =
            run_review_leaf(root_failing.path(), &bad_issue, &evidence_path_b).unwrap_err();
        assert!(err_failing.to_string().contains("failing"));
        assert!(err_failing.to_string().contains("wi-test"));
        let carrier_failing = load(root_failing.path(), &bad_slug).unwrap().unwrap();
        assert!(carrier_failing
            .evidence
            .iter()
            .all(|b| b.verifier != "wi-review"));

        // 2c: Stale wi-test binding (bound_tuple does not match current active tuple)
        let root_stale = tempfile::tempdir().unwrap();
        let evidence_path_c = root_stale.path().join("evidence.json");
        std::fs::write(
            &evidence_path_c,
            r#"{"reviewer_kind":"agent","decision":"accepted","summary":"LGTM"}"#,
        )
        .unwrap();

        record_create(root_stale.path(), &issue).unwrap();
        run_test_leaf(root_stale.path(), &issue, &[issue.clone()]).unwrap();
        let carrier_with_test = load(root_stale.path(), &slug).unwrap().unwrap();
        let stale_test_binding = carrier_with_test.evidence[0].clone(); // has old tuple

        let edited_body = complete_change_body().replace(
            "Problem statement describing the change.",
            "Edited problem statement prose.",
        );
        let edited_issue = change(&edited_body);
        record_update(root_stale.path(), &issue, &edited_issue).unwrap();

        let mut carrier_after_update = load(root_stale.path(), &slug).unwrap().unwrap();
        carrier_after_update.evidence.push(stale_test_binding); // insert stale test binding
        save(root_stale.path(), &carrier_after_update).unwrap();

        let err_stale =
            run_review_leaf(root_stale.path(), &edited_issue, &evidence_path_c).unwrap_err();
        assert!(err_stale.to_string().contains("stale"));
        assert!(err_stale.to_string().contains("wi-test"));
        let carrier_stale = load(root_stale.path(), &slug).unwrap().unwrap();
        assert!(carrier_stale
            .evidence
            .iter()
            .all(|b| b.verifier != "wi-review"));
    }

    #[test]
    fn wi_contract_review_digest_measurement_3_needs_revision_payload() {
        let root = tempfile::tempdir().unwrap();
        let issue = change(complete_change_body());
        let slug = issue.slug.clone();

        record_create(root.path(), &issue).unwrap();
        run_test_leaf(root.path(), &issue, &[issue.clone()]).unwrap();

        let evidence_path = root.path().join("evidence.json");
        std::fs::write(
            &evidence_path,
            r#"{"reviewer_kind":"agent","decision":"needs_revision","summary":"Fix section"}"#,
        )
        .unwrap();

        let proj = run_review_leaf(root.path(), &issue, &evidence_path).unwrap();

        assert!(!proj["wi_revision"].is_null());
        assert_eq!(
            proj["next"]["command"],
            serde_json::json!(format!("aw wi change {slug}"))
        );
        assert_eq!(proj["next"]["owner"], serde_json::json!("wi"));
        assert_ne!(
            proj["next"]["command"],
            serde_json::json!(format!("aw wi commit {slug}"))
        );

        let review_b = proj["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["verifier"] == "wi-review")
            .unwrap();
        assert_eq!(review_b["passed"], false);

        let reloaded = load(root.path(), &slug).unwrap().unwrap();
        assert!(valid_persisted_lifecycle(&reloaded, &slug));
    }

    #[test]
    fn wi_contract_review_digest_measurement_4_accepted_payload_carrier_invariance() {
        let root = tempfile::tempdir().unwrap();
        let issue = change(complete_change_body());
        let slug = issue.slug.clone();

        record_create(root.path(), &issue).unwrap();
        run_test_leaf(root.path(), &issue, &[issue.clone()]).unwrap();

        let carrier_before = load(root.path(), &slug).unwrap().unwrap();
        let rev_before = carrier_before.active_revisions[&ArtifactKind::Wi].clone();
        let head_before = carrier_before.head_event_id.clone();
        let epoch_before = carrier_before.epoch;
        let terminal_before = carrier_before.terminal;

        let evidence_path = root.path().join("evidence.json");
        std::fs::write(
            &evidence_path,
            r#"{"reviewer_kind":"agent","decision":"accepted","summary":"LGTM"}"#,
        )
        .unwrap();

        let proj = run_review_leaf(root.path(), &issue, &evidence_path).unwrap();

        assert_eq!(
            proj["next"]["command"],
            serde_json::json!(format!("aw wi commit {slug}"))
        );

        let carrier_after = load(root.path(), &slug).unwrap().unwrap();
        let rev_after = carrier_after.active_revisions[&ArtifactKind::Wi].clone();
        let head_after = carrier_after.head_event_id.clone();
        let epoch_after = carrier_after.epoch;
        let terminal_after = carrier_after.terminal;

        assert_eq!(rev_before, rev_after);
        assert_eq!(head_before, head_after);
        assert_eq!(epoch_before, epoch_after);
        assert_eq!(terminal_before, terminal_after);

        assert!(valid_persisted_lifecycle(&carrier_after, &slug));
    }

    #[test]
    fn wi_contract_review_digest_measurement_5_invalid_payload_field_refusals() {
        let root = tempfile::tempdir().unwrap();
        let issue = change(complete_change_body());
        let slug = issue.slug.clone();

        record_create(root.path(), &issue).unwrap();
        run_test_leaf(root.path(), &issue, &[issue.clone()]).unwrap();

        // 5a: missing reviewer_kind
        let path_no_rk = root.path().join("no_rk.json");
        std::fs::write(&path_no_rk, r#"{"decision":"accepted"}"#).unwrap();
        let err_no_rk = run_review_leaf(root.path(), &issue, &path_no_rk).unwrap_err();
        assert!(err_no_rk.to_string().contains("reviewer_kind"));

        // 5b: unknown reviewer_kind
        let path_bad_rk = root.path().join("bad_rk.json");
        std::fs::write(
            &path_bad_rk,
            r#"{"reviewer_kind":"robot","decision":"accepted"}"#,
        )
        .unwrap();
        let err_bad_rk = run_review_leaf(root.path(), &issue, &path_bad_rk).unwrap_err();
        assert!(err_bad_rk.to_string().contains("reviewer_kind"));

        // 5c: unknown decision
        let path_bad_dec = root.path().join("bad_dec.json");
        std::fs::write(
            &path_bad_dec,
            r#"{"reviewer_kind":"human","decision":"approved"}"#,
        )
        .unwrap();
        let err_bad_dec = run_review_leaf(root.path(), &issue, &path_bad_dec).unwrap_err();
        assert!(err_bad_dec.to_string().contains("decision"));

        let carrier = load(root.path(), &slug).unwrap().unwrap();
        assert!(carrier.evidence.iter().all(|b| b.verifier != "wi-review"));
    }

    #[test]
    fn wi_contract_review_digest_measurement_6_problem_prose_change_evicts_review_and_test() {
        let root = tempfile::tempdir().unwrap();
        let before_issue = change(complete_change_body());
        let slug = before_issue.slug.clone();

        record_create(root.path(), &before_issue).unwrap();
        run_test_leaf(root.path(), &before_issue, &[before_issue.clone()]).unwrap();

        let evidence_path = root.path().join("evidence.json");
        std::fs::write(
            &evidence_path,
            r#"{"reviewer_kind":"agent","decision":"accepted","summary":"LGTM"}"#,
        )
        .unwrap();

        run_review_leaf(root.path(), &before_issue, &evidence_path).unwrap();

        let carrier_before = load(root.path(), &slug).unwrap().unwrap();
        assert_eq!(carrier_before.evidence.len(), 2);

        let edited_body = complete_change_body().replace(
            "Problem statement describing the change.",
            "Edited problem statement prose.",
        );
        let edited_issue = change(&edited_body);

        let proj_drift = projection_for_issue(root.path(), &edited_issue);
        assert!(proj_drift["drift"].as_bool().unwrap());
        assert_eq!(
            proj_drift["remediation"][0]["command"],
            serde_json::json!(format!("aw wi change {slug}"))
        );

        record_update(root.path(), &before_issue, &edited_issue).unwrap();

        let carrier_after = load(root.path(), &slug).unwrap().unwrap();
        assert!(carrier_after.evidence.is_empty());
        assert_eq!(carrier_after.invalidations.len(), 1);
        let evicted = &carrier_after.invalidations[0].evicted_evidence_verifiers;
        assert!(evicted.contains(&"wi-review".to_string()));
        assert!(evicted.contains(&"wi-test".to_string()));
    }

    #[test]
    fn wi_contract_review_digest_measurement_7_marker_block_churn_preserves_review() {
        let root = tempfile::tempdir().unwrap();
        let before_issue = change(complete_change_body());
        let slug = before_issue.slug.clone();

        record_create(root.path(), &before_issue).unwrap();
        run_test_leaf(root.path(), &before_issue, &[before_issue.clone()]).unwrap();

        let evidence_path = root.path().join("evidence.json");
        std::fs::write(
            &evidence_path,
            r#"{"reviewer_kind":"agent","decision":"accepted","summary":"LGTM"}"#,
        )
        .unwrap();

        run_review_leaf(root.path(), &before_issue, &evidence_path).unwrap();

        let churned_body = format!(
            "{}\n\n<!-- aw:loop-state\nversion: 1\n-->\n",
            complete_change_body()
        );
        let churned_issue = change(&churned_body);

        record_update(root.path(), &before_issue, &churned_issue).unwrap();

        let carrier_after = load(root.path(), &slug).unwrap().unwrap();
        let review_binding = carrier_after
            .evidence
            .iter()
            .find(|b| b.verifier == "wi-review")
            .expect("wi-review binding must be present");
        assert!(review_binding.passed);
    }
}
