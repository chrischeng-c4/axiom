//! Durable causal read model for Change work items (#3347).
//!
//! The tracker issue remains the WI source of truth.  This module stores only
//! the additive, workspace-local lifecycle carrier used by `aw wi show`.

use crate::issues::Issue;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
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
    pub evidence: Vec<serde_json::Value>,
    #[serde(default)]
    pub invalidations: Vec<serde_json::Value>,
    pub iteration: u64,
    pub terminal: bool,
    pub next: NextObligation,
}

fn carrier_path(project_root: &Path, slug: &str) -> PathBuf {
    project_root
        .join(".aw")
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

fn initial_lifecycle(issue: &Issue) -> ChangeLifecycle {
    let digest = canonical_digest(&issue.body);
    let revision = ArtifactRevision {
        id: revision_id(ArtifactKind::Wi, &digest, &[]),
        kind: ArtifactKind::Wi,
        digest,
        parents: Vec::new(),
        iteration: 1,
    };
    let command = crate::cli::run::ec_draft_command(project_for(issue), &issue.slug);
    let mut active_revisions = empty_revisions();
    active_revisions.insert(ArtifactKind::Wi, Some(revision.clone()));
    ChangeLifecycle {
        schema: SCHEMA.to_string(),
        slug: issue.slug.clone(),
        epoch: 1,
        head_event_id: Some("evt-001".to_string()),
        active_revisions,
        events: vec![LifecycleEvent {
            event_id: "evt-001".to_string(),
            predecessor_id: None,
            kind: LifecycleEventKind::WiCreate,
            candidate_revision: revision,
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

fn valid_persisted_lifecycle(lifecycle: &ChangeLifecycle, requested_slug: &str) -> bool {
    if lifecycle.schema != SCHEMA || lifecycle.slug != requested_slug {
        return false;
    }
    if lifecycle.events.is_empty() {
        return lifecycle.head_event_id.is_none();
    }
    if lifecycle.events[0].predecessor_id.is_some()
        || lifecycle.head_event_id.as_deref()
            != lifecycle.events.last().map(|event| event.event_id.as_str())
    {
        return false;
    }
    if lifecycle
        .events
        .windows(2)
        .any(|events| events[1].predecessor_id.as_deref() != Some(events[0].event_id.as_str()))
    {
        return false;
    }
    let latest_wi_event = lifecycle.events.iter().rev().find(|event| {
        matches!(
            event.kind,
            LifecycleEventKind::WiCreate | LifecycleEventKind::WiChange
        )
    });
    match latest_wi_event {
        Some(event) => {
            lifecycle
                .active_revisions
                .get(&ArtifactKind::Wi)
                .and_then(|revision| revision.as_ref())
                == Some(&event.candidate_revision)
        }
        None => true,
    }
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
    let Some(mut lifecycle) = load(project_root, &updated.slug)? else {
        return Ok(());
    };
    let old_digest = lifecycle
        .active_revisions
        .get(&ArtifactKind::Wi)
        .and_then(|revision| revision.as_ref())
        .map(|revision| revision.digest.clone())
        .unwrap_or_else(|| canonical_digest(&before.body));
    let new_digest = canonical_digest(&updated.body);
    if old_digest == new_digest {
        return Ok(());
    }
    let iteration = lifecycle.iteration + 1;
    let revision = ArtifactRevision {
        id: revision_id(ArtifactKind::Wi, &new_digest, &[]),
        kind: ArtifactKind::Wi,
        digest: new_digest,
        parents: Vec::new(),
        iteration,
    };
    let event_id = format!("evt-{:03}", lifecycle.events.len() + 1);
    let command = format!("aw wi validate {}", updated.slug);
    lifecycle.events.push(LifecycleEvent {
        event_id: event_id.clone(),
        predecessor_id: lifecycle.head_event_id.clone(),
        kind: LifecycleEventKind::WiChange,
        candidate_revision: revision.clone(),
        next_command: command.clone(),
        next_owner: OwnerVocabulary::Wi,
    });
    lifecycle.epoch += 1;
    lifecycle.head_event_id = Some(event_id);
    lifecycle.iteration = iteration;
    lifecycle.terminal = false;
    lifecycle.next = NextObligation {
        command,
        owner: OwnerVocabulary::Wi,
    };
    lifecycle
        .active_revisions
        .insert(ArtifactKind::Wi, Some(revision));
    for kind in [ArtifactKind::Ec, ArtifactKind::Td, ArtifactKind::Cb] {
        lifecycle.active_revisions.insert(kind, None);
    }
    lifecycle.evidence.clear();
    save(project_root, &lifecycle)
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
        let ec_revision = ArtifactRevision {
            id: revision_id(ArtifactKind::Ec, &ec_digest, &[]),
            kind: ArtifactKind::Ec,
            digest: ec_digest,
            parents: Vec::new(),
            iteration: 1,
        };
        lifecycle.events.push(LifecycleEvent {
            event_id: "evt-002".to_string(),
            predecessor_id: Some("evt-001".to_string()),
            kind: LifecycleEventKind::EcChange,
            candidate_revision: ec_revision.clone(),
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
