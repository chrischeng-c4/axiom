//! Durable causal read model for Change work items (#3347).
//!
//! The tracker issue remains the WI source of truth.  This module stores only
//! the additive, workspace-local lifecycle carrier used by `aw wi show`.

use crate::issues::Issue;
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
    let digest = canonical_digest(body);
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
    let new_digest = canonical_digest(new_body);
    let old_digest = prior_lifecycle
        .active_revisions
        .get(&ArtifactKind::Wi)
        .and_then(|revision| revision.as_ref())
        .map(|revision| revision.digest.clone())
        .or_else(|| pre_update_body.map(canonical_digest));
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
    })
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
}
