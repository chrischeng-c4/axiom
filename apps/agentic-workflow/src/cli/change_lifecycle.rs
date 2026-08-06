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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TrackerObservation {
    pub body: String,
    pub head_event_id: Option<String>,
    pub epoch: Option<u64>,
    pub present_event_ids: BTreeSet<String>,
    #[serde(default)]
    pub state: Option<IssueState>,
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
        }
    }

    pub fn empty(body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            head_event_id: None,
            epoch: None,
            present_event_ids: BTreeSet::new(),
            state: Some(IssueState::Open),
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
}

fn carrier_path(project_root: &Path, slug: &str) -> PathBuf {
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

fn canonicalize_wi_body(body: &str) -> String {
    let stripped = strip_aw_marker_blocks(body);
    let normalized = stripped.replace("\r\n", "\n");
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

fn canonical_wi_digest(body: &str) -> String {
    canonical_digest(&canonicalize_wi_body(body))
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
    issue
        .labels
        .iter()
        .find_map(|label| {
            label
                .strip_prefix("app:")
                .or_else(|| label.strip_prefix("lib:"))
                .or_else(|| label.strip_prefix("project:"))
        })
        .unwrap_or("agentic-workflow")
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
        command: format!("aw wi validate {slug}"),
        owner: OwnerVocabulary::Wi,
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
            command: "aw td check".to_string(),
            owner: OwnerVocabulary::Td,
        },
        FailureOwnership::Implementation => NextObligation {
            command: "aw cb check".to_string(),
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

fn transitive_invalidation(
    trigger: &ArtifactRevision,
    current: &BTreeMap<ArtifactKind, Option<ArtifactRevision>>,
    evidence: &[EvidenceBinding],
) -> InvalidationRecord {
    let invalidated_kinds = match trigger.kind {
        ArtifactKind::Wi => vec![ArtifactKind::Ec, ArtifactKind::Td, ArtifactKind::Cb],
        ArtifactKind::Ec => vec![ArtifactKind::Td, ArtifactKind::Cb],
        ArtifactKind::Td => vec![ArtifactKind::Cb],
        ArtifactKind::Cb => Vec::new(),
    };
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

/// Construct the post-backend-success WI creation event and durable record.
pub fn fold_wi_create(slug: &str, body: &str, project: &str) -> ChangeLifecycle {
    let digest = canonical_wi_digest(body);
    let revision = artifact_revision(ArtifactKind::Wi, digest.clone(), Vec::new(), 1);
    let command = crate::cli::run::ec_draft_command(project, slug);
    let mut active_revisions = empty_revisions();
    active_revisions.insert(ArtifactKind::Wi, Some(revision.clone()));
    ChangeLifecycle {
        schema: SCHEMA.to_string(),
        slug: slug.to_string(),
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

fn initial_lifecycle(issue: &Issue) -> ChangeLifecycle {
    fold_wi_create(&issue.slug, &issue.body, project_for(issue))
}

/// Fold a body update against the persisted carrier.  Equal canonical bodies
/// remain event-free so callers can preserve the carrier bytes exactly.
pub fn fold_wi_update(
    prior_lifecycle: &ChangeLifecycle,
    new_body: &str,
    pre_update_body: Option<&str>,
) -> ReducerResult {
    let stored_digest = prior_lifecycle
        .active_revisions
        .get(&ArtifactKind::Wi)
        .and_then(|revision| revision.as_ref())
        .map(|revision| revision.digest.clone());

    if let (Some(observed), Some(stored)) = (pre_update_body, &stored_digest) {
        let observed_digest = canonical_wi_digest(observed);
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

    let new_digest = canonical_wi_digest(new_body);
    let old_digest = stored_digest.or_else(|| pre_update_body.map(canonical_wi_digest));
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
        },
    )
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

fn load(project_root: &Path, slug: &str) -> Result<Option<ChangeLifecycle>> {
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
    replayed.epoch == lifecycle.epoch
        && replayed.head_event_id == lifecycle.head_event_id
        && replayed.active_revisions == lifecycle.active_revisions
        && replayed.iteration == lifecycle.iteration
        && replayed.terminal == lifecycle.terminal
        && replayed.next == lifecycle.next
        && lifecycle.invalidations.len() == replayed.invalidations.len()
        && lifecycle
            .invalidations
            .iter()
            .zip(&replayed.invalidations)
            .all(|(persisted, replayed)| same_invalidation_shape(persisted, replayed))
}

fn save(project_root: &Path, lifecycle: &ChangeLifecycle) -> Result<()> {
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

/// Fold a successful issue creation into the durable carrier.  Legacy
/// loop-state WIs deliberately remain unmigrated and are rendered fail-closed.
pub fn record_create(project_root: &Path, issue: &Issue) -> Result<()> {
    if !issue.issue_type.is_change() || is_legacy_loop_state(issue) {
        return Ok(());
    }
    save(project_root, &initial_lifecycle(issue))
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
    let result = fold_wi_update(&lifecycle, &updated.body, Some(&before.body));
    if !result.accepted {
        return Ok(());
    }
    save(project_root, &result.lifecycle)
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
        let obs_wi_digest = canonical_wi_digest(&observation.body);
        if obs_wi_digest != *active_wi_digest {
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
                drift: false,
                remediation: Vec::new(),
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
    }
}

fn render(lifecycle: &ChangeLifecycle) -> serde_json::Value {
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
            render(&lifecycle)
        }
        Ok(Some(_)) | Ok(None) | Err(_) => zero_head_projection(&issue.slug, OwnerVocabulary::Wi),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issues::{IssueState, IssueType};

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

    fn candidate_tuple(
        lifecycle: &ChangeLifecycle,
        candidate: &ArtifactRevision,
    ) -> ActiveDigestTuple {
        let mut tuple = lifecycle.active_digest_tuple();
        match candidate.kind {
            ArtifactKind::Wi => tuple.wi_digest = Some(candidate.digest.clone()),
            ArtifactKind::Ec => tuple.ec_digest = Some(candidate.digest.clone()),
            ArtifactKind::Td => tuple.td_digest = Some(candidate.digest.clone()),
            ArtifactKind::Cb => tuple.cb_digest = Some(candidate.digest.clone()),
        }
        tuple
    }

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
            },
        );
        assert!(result.accepted, "{:?}", result.rejection_reason);
        result.lifecycle
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
        assert_eq!(projection["ledger"]["head_event_id"], "evt-002");
        assert_eq!(projection["ec_revision"]["id"], ec_revision.id);
        assert_eq!(projection["next"]["owner"], "td");
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
            },
        );
        assert!(!stale.accepted);
        assert_eq!(stale.lifecycle.next.command, "aw wi validate causal");

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
        assert_eq!(non_cb_commit.lifecycle.next.command, "aw cb check");

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
        let proj_4 = render(&cb);
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
        };
        let committed = reduce_event(&cb, commit_event);
        assert!(committed.accepted);
        let terminal_lc = committed.lifecycle;

        // Row 4: cb_commit milestone next.command is "aw wi show causal", next.owner is "cb"
        let terminal_proj = render(&terminal_lc);
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
        };
        let rej = reduce_event(&terminal_lc, stale_event);
        assert!(!rej.accepted);
        let proj_after_rej = render(&rej.lifecycle);
        assert_eq!(proj_after_rej["milestones"], terminal_proj["milestones"]);

        // Row 8: empty evidence on terminal lifecycle: cb_commit milestone still reports aw wi show and owner cb, and evidence []
        let mut emptied_lc = terminal_lc.clone();
        emptied_lc.evidence.clear();
        let emptied_proj = render(&emptied_lc);
        let emptied_ms = emptied_proj["milestones"].as_array().unwrap();
        let emptied_cb_ms = &emptied_ms[4];
        assert_eq!(emptied_cb_ms["next"]["command"], "aw wi show causal");
        assert_eq!(emptied_cb_ms["next"]["owner"], "cb");
        assert_eq!(emptied_cb_ms["evidence"], serde_json::json!([]));
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
}
