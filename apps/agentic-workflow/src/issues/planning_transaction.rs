// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/logic/issues/project-plan-transaction.md#logic
// HANDWRITE-BEGIN gap="missing-generator:rust:project-plan-transaction" tracker="#2388" reason="The reviewed issue-platform transaction is a new issue-domain aggregate; the spec defines its deterministic manifest and retry contract while the generator does not emit backend orchestration."

//! Digest-bound, retry-safe publication of one reviewed project plan.

use super::{
    explicit_parent_references, issue_key, Issue, IssueBackend, IssueFilter, IssuePatch, IssueType,
    PlanningStage, ProjectPlan,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const PLANNING_TRANSACTION_SCHEMA: &str = "aw.wi.project-plan-transaction.v2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningTransactionManifest {
    pub version: u8,
    pub schema: String,
    pub kind: String,
    pub root_id: String,
    pub stage: PlanningStage,
    pub project: String,
    pub project_label: String,
    pub plan_digest: String,
    pub tracker_snapshot_digest: String,
    pub issue_snapshots: Vec<PlanningIssueSnapshot>,
    pub mutations: Vec<PlanningMutation>,
    pub apply_command: String,
    pub terminal_next: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningIssueSnapshot {
    pub id: String,
    pub issue_type: String,
    pub title: String,
    pub state: String,
    pub labels: Vec<String>,
    pub body: String,
    pub related: Vec<String>,
    pub implements: Vec<String>,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningMutation {
    pub order: usize,
    pub idempotency_key: String,
    pub action: String,
    pub target: String,
    pub issue_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub add_labels: Vec<String>,
    pub remove_labels: Vec<String>,
    pub reason: String,
    pub stage: PlanningStage,
    pub certainty: String,
    pub evidence: Vec<String>,
    pub decision_source: String,
    pub requires_hitl: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningMutationResult {
    pub order: usize,
    pub idempotency_key: String,
    pub target: String,
    pub resolved_issue: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningTransactionResult {
    pub schema: String,
    pub source_digest: String,
    pub tracker_snapshot_digest: String,
    pub status: String,
    pub no_op: bool,
    pub mutation_count: usize,
    pub applied_count: usize,
    pub reconciled_count: usize,
    pub created_issue_count: usize,
    pub results: Vec<PlanningMutationResult>,
    pub next: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PlanningTransactionCheckpoint {
    schema: String,
    source_digest: String,
    status: String,
    results: Vec<PlanningMutationResult>,
}

/// Build the exact mutation set reviewed by `aw wi plan-review`.
pub fn build_planning_transaction_manifest(
    plan: &ProjectPlan,
    issues: &[Issue],
    apply_command: &str,
    terminal_next: &str,
) -> PlanningTransactionManifest {
    let mut issue_snapshots = issues
        .iter()
        .map(PlanningIssueSnapshot::from_issue)
        .collect::<Vec<_>>();
    issue_snapshots
        .sort_by(|left, right| reference_sort_key(&left.id).cmp(&reference_sort_key(&right.id)));
    let tracker_snapshot_digest = snapshot_digest(&issue_snapshots);
    let issue_by_id = issues
        .iter()
        .map(|issue| (issue_key(issue), issue))
        .collect::<BTreeMap<_, _>>();
    let requirement_text_by_id = plan
        .epics
        .iter()
        .flat_map(|epic| {
            epic.requirements.iter().map(|requirement| {
                (
                    requirement.id.clone(),
                    (requirement.text.clone(), requirement.verification.clone()),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    let mut mutations = Vec::new();

    for proposal in &plan.proposed_epics {
        let target = proposal.id.clone();
        let key = mutation_key(&plan.digest, "create", &target);
        let (capability, provenance) = proposal
            .source_epic
            .strip_prefix("bootstrap:")
            .map(|context| {
                (
                    context.to_string(),
                    format!("Planning Context: `{context}`"),
                )
            })
            .unwrap_or_else(|| {
                (
                    "work-item-planning".to_string(),
                    format!("Source Epic: #{}", proposal.source_epic),
                )
            });
        let body = format!(
            "## Capability Alignment\n\nCapability: {capability}\n{provenance}\nPlanning Horizon: `{}`\n\n## Requirements\n\n{}\n\n## Scope\n\n### In Scope\n- Deliver the reviewed {} horizon.\n\n### Out of Scope\n- Delete or rewrite retained issue history.\n\n## Acceptance Criteria\n\n- This epic owns only the reviewed {} requirements and retained changes.\n\n## Reference Context\n\n- Project plan digest: `{}`\n\n{}\n",
            proposal.horizon,
            proposal
                .requirements
                .iter()
                .map(|requirement| format!("- {requirement}"))
                .collect::<Vec<_>>()
                .join("\n"),
            proposal.horizon,
            proposal.horizon,
            plan.digest,
            transaction_marker(&key),
        );
        mutations.push(PlanningMutation {
            order: 0,
            idempotency_key: key,
            action: "create".to_string(),
            target,
            issue_type: "epic".to_string(),
            title: Some(proposal.title.clone()),
            body: Some(body),
            add_labels: sorted_labels([
                "type:epic".to_string(),
                plan.project_label.clone(),
                format!("priority:{}", proposal.priority),
            ]),
            remove_labels: Vec::new(),
            reason: proposal.reason.clone(),
            stage: plan.stage,
            certainty: "decision".to_string(),
            evidence: vec![format!("source_epic:{}", proposal.source_epic)],
            decision_source: "agent_review_and_human_confirmation".to_string(),
            requires_hitl: true,
        });
    }

    for proposal in &plan.proposed_changes {
        if proposal.covers.is_empty() {
            continue;
        }
        let target = proposal.id.clone();
        let key = mutation_key(&plan.digest, "create", &target);
        let capability = issue_by_id
            .get(&proposal.owner_epic)
            .and_then(|issue| issue_capability(issue))
            .unwrap_or_else(|| format!("inherited from parent epic #{}", proposal.owner_epic));
        let scope = proposal
            .covers
            .iter()
            .filter_map(|requirement| requirement_text_by_id.get(requirement))
            .map(|(requirement, _)| format!("- {requirement}"))
            .collect::<Vec<_>>();
        let scope = if scope.is_empty() {
            format!("- {}", proposal.title)
        } else {
            scope.join("\n")
        };
        let verification = proposal
            .covers
            .iter()
            .filter_map(|requirement| requirement_text_by_id.get(requirement))
            .flat_map(|(_, verification)| verification)
            .map(|verification| {
                format!(
                    "- Gate: `{}`\n  Oracle: {}",
                    verification.gate, verification.oracle
                )
            })
            .collect::<Vec<_>>();
        let verification = if verification.is_empty() {
            "- Gate and oracle must be supplied before publication.".to_string()
        } else {
            verification.join("\n")
        };
        let body = format!(
            "## Capability Alignment\n\nCapability: {capability}\nParent Epic: #{}\n\n## Scope\n\n### In Scope\n{}\n\n### Out of Scope\n- Unrelated requirements outside this reviewed atomic change.\n\n## Acceptance Criteria\n\n- The change closes reviewed requirement coverage: {}.\n\n## Verification\n\n{}\n\n## Reference Context\n\n- Project plan digest: `{}`\n- Planning reason: `{}`\n{}\n\n{}\n",
            proposal.owner_epic,
            scope,
            proposal.covers.join(", "),
            verification,
            plan.digest,
            proposal.reason,
            proposal
                .source_change
                .as_ref()
                .map(|source| format!("- Supersedes reviewed source change: #{source}"))
                .unwrap_or_default(),
            transaction_marker(&key),
        );
        let mut labels = vec![
            "type:change".to_string(),
            plan.project_label.clone(),
            format!("priority:{}", proposal.priority),
            format!("epic:{}", proposal.owner_epic),
        ];
        labels.extend(
            proposal
                .dependencies
                .iter()
                .map(|dependency| format!("depends-on:{dependency}")),
        );
        if let Some(source) = &proposal.source_change {
            labels.push(format!("supersedes:{source}"));
        }
        mutations.push(PlanningMutation {
            order: 0,
            idempotency_key: key,
            action: "create".to_string(),
            target,
            issue_type: "change".to_string(),
            title: Some(proposal.title.clone()),
            body: Some(body),
            add_labels: sorted_labels(labels),
            remove_labels: Vec::new(),
            reason: proposal.reason.clone(),
            stage: plan.stage,
            certainty: "decision".to_string(),
            evidence: proposal.covers.clone(),
            decision_source: "agent_review_and_human_confirmation".to_string(),
            requires_hitl: true,
        });
    }

    for epic in &plan.epics {
        let Some(issue) = issue_by_id.get(&epic.id).copied() else {
            continue;
        };
        let target = epic.id.clone();
        let key = mutation_key(&plan.digest, "update", &target);
        let desired = sorted_labels([
            "type:epic".to_string(),
            plan.project_label.clone(),
            format!("priority:{}", epic.priority),
        ]);
        let (add_labels, remove_labels) = managed_label_delta(issue, &desired, false);
        if add_labels.is_empty() && remove_labels.is_empty() && epic.split_into.is_empty() {
            continue;
        }
        let note = format!(
            "Plan digest `{}` assigns project order {} and priority `{}`.{} The source epic is retained as history.",
            plan.digest,
            epic.order,
            epic.priority,
            if epic.split_into.is_empty() {
                String::new()
            } else {
                format!(" Reviewed sibling split: {}.", epic.split_into.join(", "))
            }
        );
        mutations.push(PlanningMutation {
            order: 0,
            idempotency_key: key.clone(),
            action: "update".to_string(),
            target,
            issue_type: "epic".to_string(),
            title: None,
            body: Some(append_transaction_note(&issue.body, &key, &note)),
            add_labels,
            remove_labels,
            reason: if epic.split_into.is_empty() {
                "canonicalize_epic_priority_and_order".to_string()
            } else {
                "record_reviewed_epic_split".to_string()
            },
            stage: plan.stage,
            certainty: if plan.stage == PlanningStage::Normalize {
                "deterministic"
            } else {
                "inferred"
            }
            .to_string(),
            evidence: vec![format!("issue:{}", epic.id)],
            decision_source: if plan.stage == PlanningStage::Normalize {
                "explicit_metadata"
            } else {
                "human_decision"
            }
            .to_string(),
            requires_hitl: plan.stage == PlanningStage::Reconcile,
        });
    }

    for change in &plan.changes {
        let Some(issue) = issue_by_id.get(&change.id).copied() else {
            continue;
        };
        let target = change.id.clone();
        let key = mutation_key(&plan.digest, "update", &target);
        let mut desired = vec![
            "type:change".to_string(),
            plan.project_label.clone(),
            format!("priority:{}", change.priority),
            format!("epic:{}", change.owner_epic),
        ];
        desired.extend(
            change
                .dependencies
                .iter()
                .map(|dependency| format!("depends-on:{dependency}")),
        );
        if let Some(duplicate) = &change.duplicate_of {
            desired.push(format!("duplicate-of:{duplicate}"));
        }
        desired.extend(
            change
                .replacement_ids
                .iter()
                .map(|replacement| format!("superseded-by:{replacement}")),
        );
        let desired = sorted_labels(desired);
        let (add_labels, remove_labels) = managed_label_delta(issue, &desired, true);
        let reparented = explicit_parent_references(issue)
            .iter()
            .any(|owner| owner != &change.owner_epic);
        let base_body = if reparented {
            canonical_parent_body(&issue.body, &change.owner_epic)
        } else {
            issue.body.clone()
        };
        if add_labels.is_empty()
            && remove_labels.is_empty()
            && change.duplicate_of.is_none()
            && change.replacement_ids.is_empty()
            && !reparented
        {
            continue;
        }
        let note = format!(
            "Plan digest `{}` keeps this issue and records lane `{}`, owner `{}`, duplicate target `{}`, and replacement recommendations `{}`.",
            plan.digest,
            change.lane,
            change.owner_epic,
            change.duplicate_of.as_deref().unwrap_or("none"),
            if change.replacement_ids.is_empty() {
                "none".to_string()
            } else {
                change.replacement_ids.join(", ")
            }
        );
        mutations.push(PlanningMutation {
            order: 0,
            idempotency_key: key.clone(),
            action: "update".to_string(),
            target,
            issue_type: "change".to_string(),
            title: None,
            body: Some(append_transaction_note(&base_body, &key, &note)),
            add_labels,
            remove_labels,
            reason: if change.duplicate_of.is_some() {
                "record_duplicate_recommendation_without_deletion".to_string()
            } else if !change.replacement_ids.is_empty() {
                "record_supersession_recommendation_without_deletion".to_string()
            } else {
                "canonicalize_change_graph_and_priority".to_string()
            },
            stage: plan.stage,
            certainty: if plan.stage == PlanningStage::Normalize {
                "deterministic"
            } else {
                "inferred"
            }
            .to_string(),
            evidence: vec![format!("issue:{}", change.id)],
            decision_source: if plan.stage == PlanningStage::Normalize {
                "explicit_metadata"
            } else {
                "human_decision"
            }
            .to_string(),
            requires_hitl: plan.stage == PlanningStage::Reconcile,
        });
    }

    mutations.retain(|mutation| match plan.stage {
        PlanningStage::Normalize => {
            if mutation.action != "update" || mutation.certainty != "deterministic" {
                return false;
            }
            let Some(issue) = issue_by_id.get(&mutation.target).copied() else {
                return false;
            };
            if mutation.issue_type == "epic" {
                issue.labels.iter().any(|label| {
                    matches!(
                        label.as_str(),
                        "priority:p0" | "priority:p1" | "priority:p2" | "priority:p3"
                    )
                })
            } else {
                let Some(change) = plan
                    .changes
                    .iter()
                    .find(|change| change.id == mutation.target)
                else {
                    return false;
                };
                !change.owner_epic.starts_with("proposal:")
                    && change.priority_source == "explicit"
                    && explicit_parent_references(issue) == vec![change.owner_epic.clone()]
            }
        }
        PlanningStage::Reconcile => {
            mutation.action == "update"
                && !plan.changes.iter().any(|change| {
                    change.id == mutation.target && change.owner_epic.starts_with("proposal:")
                })
        }
        PlanningStage::Atomize => true,
        PlanningStage::Verify => false,
    });

    let dependency_depths = create_dependency_depths(&mutations);
    mutations.sort_by(|left, right| {
        mutation_action_rank(&left.action, &left.issue_type)
            .cmp(&mutation_action_rank(&right.action, &right.issue_type))
            .then_with(|| {
                dependency_depths
                    .get(&left.target)
                    .copied()
                    .unwrap_or_default()
                    .cmp(
                        &dependency_depths
                            .get(&right.target)
                            .copied()
                            .unwrap_or_default(),
                    )
            })
            .then_with(|| reference_sort_key(&left.target).cmp(&reference_sort_key(&right.target)))
    });
    for (index, mutation) in mutations.iter_mut().enumerate() {
        mutation.order = index + 1;
    }

    PlanningTransactionManifest {
        version: 2,
        schema: PLANNING_TRANSACTION_SCHEMA.to_string(),
        kind: "project_plan".to_string(),
        root_id: plan.root_id.clone(),
        stage: plan.stage,
        project: plan.project.clone(),
        project_label: plan.project_label.clone(),
        plan_digest: plan.digest.clone(),
        tracker_snapshot_digest,
        issue_snapshots,
        mutations,
        apply_command: apply_command.to_string(),
        terminal_next: terminal_next.to_string(),
    }
}

fn create_dependency_depths(mutations: &[PlanningMutation]) -> BTreeMap<String, usize> {
    let create_targets = mutations
        .iter()
        .filter(|mutation| mutation.action == "create")
        .map(|mutation| mutation.target.clone())
        .collect::<BTreeSet<_>>();
    let dependencies = mutations
        .iter()
        .filter(|mutation| mutation.action == "create")
        .map(|mutation| {
            let values = mutation
                .add_labels
                .iter()
                .filter_map(|label| label.strip_prefix("depends-on:"))
                .filter(|dependency| create_targets.contains(*dependency))
                .map(str::to_string)
                .collect::<Vec<_>>();
            (mutation.target.clone(), values)
        })
        .collect::<BTreeMap<_, _>>();

    fn depth(
        target: &str,
        dependencies: &BTreeMap<String, Vec<String>>,
        visiting: &mut BTreeSet<String>,
        memo: &mut BTreeMap<String, usize>,
    ) -> usize {
        if let Some(depth) = memo.get(target) {
            return *depth;
        }
        if !visiting.insert(target.to_string()) {
            return 0;
        }
        let value = dependencies
            .get(target)
            .into_iter()
            .flatten()
            .map(|dependency| depth(dependency, dependencies, visiting, memo) + 1)
            .max()
            .unwrap_or_default();
        visiting.remove(target);
        memo.insert(target.to_string(), value);
        value
    }

    let mut memo = BTreeMap::new();
    for target in create_targets {
        depth(&target, &dependencies, &mut BTreeSet::new(), &mut memo);
    }
    memo
}

/// Apply or reconcile the reviewed mutation manifest. Preflight finishes
/// before the first write; every mutation is independently idempotent.
pub async fn apply_planning_transaction(
    backend: &dyn IssueBackend,
    manifest: &PlanningTransactionManifest,
    source_digest: &str,
    checkpoint_path: &Path,
) -> Result<PlanningTransactionResult> {
    if manifest.version != 2
        || manifest.schema != PLANNING_TRANSACTION_SCHEMA
        || manifest.kind != "project_plan"
    {
        anyhow::bail!("unsupported project planning transaction manifest");
    }
    validate_stage_mutations(manifest)?;
    let mut checkpoint = load_checkpoint(checkpoint_path, source_digest)?;
    let mut current = backend.list(&IssueFilter::default()).await?;
    let mut resolutions = reconcile_created_targets(manifest, &current);
    preflight_tracker_snapshot(manifest, &current, &resolutions)?;

    checkpoint.status = "applying".to_string();
    write_checkpoint(checkpoint_path, &checkpoint)?;
    let mut results = Vec::new();
    let mut applied_count = 0usize;
    let mut reconciled_count = 0usize;
    let mut created_issue_count = 0usize;

    for mutation in &manifest.mutations {
        let resolved = resolve_mutation(mutation, &resolutions);
        if let Some(existing) = mutation_already_applied(&resolved, &current, &resolutions) {
            if mutation.action == "create" {
                resolutions.insert(mutation.target.clone(), existing.clone());
            }
            reconciled_count += 1;
            results.push(PlanningMutationResult {
                order: mutation.order,
                idempotency_key: mutation.idempotency_key.clone(),
                target: mutation.target.clone(),
                resolved_issue: existing,
                status: "reconciled".to_string(),
            });
            continue;
        }

        let resolved_issue = match mutation.action.as_str() {
            "create" => {
                let issue = issue_from_create_mutation(&resolved)?;
                let created = backend.create(&issue).await.with_context(|| {
                    format!("planning transaction create `{}` failed", mutation.target)
                })?;
                let id = issue_key(&created);
                resolutions.insert(mutation.target.clone(), id.clone());
                current.push(created);
                created_issue_count += 1;
                id
            }
            "update" => {
                let target = resolve_reference(&mutation.target, &resolutions);
                let updated = backend
                    .update(&target, &patch_from_mutation(&resolved))
                    .await
                    .with_context(|| format!("planning transaction update `{target}` failed"))?;
                replace_current_issue(&mut current, updated.clone());
                issue_key(&updated)
            }
            other => anyhow::bail!("unsupported planning mutation action `{other}`"),
        };
        applied_count += 1;
        let result = PlanningMutationResult {
            order: mutation.order,
            idempotency_key: mutation.idempotency_key.clone(),
            target: mutation.target.clone(),
            resolved_issue,
            status: "applied".to_string(),
        };
        results.push(result.clone());
        checkpoint
            .results
            .retain(|item| item.idempotency_key != result.idempotency_key);
        checkpoint.results.push(result);
        write_checkpoint(checkpoint_path, &checkpoint)?;
    }

    checkpoint.status = "complete".to_string();
    checkpoint.results = results.clone();
    write_checkpoint(checkpoint_path, &checkpoint)?;
    Ok(PlanningTransactionResult {
        schema: PLANNING_TRANSACTION_SCHEMA.to_string(),
        source_digest: source_digest.to_string(),
        tracker_snapshot_digest: manifest.tracker_snapshot_digest.clone(),
        status: "complete".to_string(),
        no_op: applied_count == 0,
        mutation_count: manifest.mutations.len(),
        applied_count,
        reconciled_count,
        created_issue_count,
        results,
        next: manifest.terminal_next.clone(),
    })
}

/// Digest used by the independent project-plan review record. The plan and
/// complete transaction manifest are one authorization unit.
pub fn planning_transaction_source_digest(plan_body: &str, manifest_body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plan_body.as_bytes());
    hasher.update([0]);
    hasher.update(manifest_body.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Keep retry checkpoints scoped to the exact plan/manifest authorization
/// unit. A later iteration of the same stage must not collide with an earlier
/// completed digest.
pub fn planning_transaction_checkpoint_path(
    plan_path: &Path,
    stage: PlanningStage,
    source_digest: &str,
) -> PathBuf {
    plan_path.with_extension(format!(
        "{}.{}.transaction.json",
        stage.as_str(),
        source_digest
    ))
}

/// Read-only admission gate for goal roots consuming a published plan.
///
/// Lifecycle state and non-graph labels may advance after publication. Graph
/// ownership, priority, dependencies, duplicate/supersession relations, issue
/// identity, and the transaction provenance marker must still match the exact
/// completed manifest.
pub fn verify_published_planning_transaction(
    manifest: &PlanningTransactionManifest,
    source_digest: &str,
    checkpoint_path: &Path,
    current: &[Issue],
) -> Result<()> {
    if manifest.version != 2
        || manifest.schema != PLANNING_TRANSACTION_SCHEMA
        || manifest.kind != "project_plan"
    {
        anyhow::bail!("unsupported published project planning transaction manifest");
    }
    let checkpoint = load_checkpoint(checkpoint_path, source_digest)?;
    if checkpoint.status != "complete" {
        anyhow::bail!(
            "project planning transaction is not complete for reviewed digest `{source_digest}`"
        );
    }
    let result_by_key = checkpoint
        .results
        .iter()
        .map(|result| (result.idempotency_key.as_str(), result))
        .collect::<BTreeMap<_, _>>();
    for mutation in &manifest.mutations {
        if !result_by_key.contains_key(mutation.idempotency_key.as_str()) {
            anyhow::bail!(
                "published project plan is incomplete: mutation {} has no completed result",
                mutation.order
            );
        }
    }

    let relevant = current
        .iter()
        .filter(|issue| {
            issue
                .labels
                .iter()
                .any(|label| label == &manifest.project_label)
        })
        .map(|issue| (issue_key(issue), issue))
        .collect::<BTreeMap<_, _>>();
    let mut resolutions = BTreeMap::new();
    for mutation in manifest
        .mutations
        .iter()
        .filter(|mutation| mutation.action == "create")
    {
        let result = result_by_key[mutation.idempotency_key.as_str()];
        let marker = transaction_marker(&mutation.idempotency_key);
        let Some(issue) = relevant.get(&result.resolved_issue).copied() else {
            anyhow::bail!(
                "published project plan is stale: created issue `{}` is missing",
                result.resolved_issue
            );
        };
        if !issue.body.contains(&marker) {
            anyhow::bail!(
                "published project plan is stale: created issue `{}` lost transaction marker",
                result.resolved_issue
            );
        }
        resolutions.insert(mutation.target.clone(), result.resolved_issue.clone());
    }

    let expected_ids = manifest
        .issue_snapshots
        .iter()
        .map(|snapshot| snapshot.id.clone())
        .chain(resolutions.values().cloned())
        .collect::<BTreeSet<_>>();
    if let Some(unexpected) = relevant.keys().find(|id| !expected_ids.contains(*id)) {
        anyhow::bail!(
            "published project plan is stale: project issue `{unexpected}` was not in the reviewed transaction"
        );
    }
    for snapshot in &manifest.issue_snapshots {
        let Some(issue) = relevant.get(&snapshot.id).copied() else {
            anyhow::bail!(
                "published project plan is stale: reviewed issue `{}` is missing",
                snapshot.id
            );
        };
        if manifest
            .mutations
            .iter()
            .any(|mutation| mutation.action == "update" && mutation.target == snapshot.id)
        {
            continue;
        }
        let expected_graph_labels = graph_labels(&snapshot.labels, &manifest.project_label);
        let actual_graph_labels = graph_labels(&issue.labels, &manifest.project_label);
        if expected_graph_labels != actual_graph_labels {
            anyhow::bail!(
                "published project plan is stale: issue `{}` graph labels changed (expected: {}; actual: {})",
                snapshot.id,
                expected_graph_labels.join(", "),
                actual_graph_labels.join(", ")
            );
        }
    }

    let snapshots = manifest
        .issue_snapshots
        .iter()
        .map(|snapshot| (snapshot.id.as_str(), snapshot))
        .collect::<BTreeMap<_, _>>();
    for mutation in &manifest.mutations {
        let resolved = resolve_mutation(mutation, &resolutions);
        let Some(issue) = relevant.get(&resolved.target).copied() else {
            anyhow::bail!(
                "published project plan is stale: mutation target `{}` is missing",
                resolved.target
            );
        };
        if issue.issue_type.as_str() != resolved.issue_type {
            anyhow::bail!(
                "published project plan is stale: issue `{}` changed type",
                resolved.target
            );
        }
        if resolved
            .title
            .as_ref()
            .is_some_and(|title| title != &issue.title)
        {
            anyhow::bail!(
                "published project plan is stale: issue `{}` changed title",
                resolved.target
            );
        }
        if !issue
            .body
            .contains(&transaction_marker(&resolved.idempotency_key))
        {
            anyhow::bail!(
                "published project plan is stale: issue `{}` lost transaction provenance",
                resolved.target
            );
        }

        let expected_labels = if mutation.action == "create" {
            resolved.add_labels.clone()
        } else {
            let snapshot = snapshots.get(mutation.target.as_str()).ok_or_else(|| {
                anyhow::anyhow!(
                    "published project plan is stale: update target `{}` lacks reviewed snapshot",
                    mutation.target
                )
            })?;
            let mut labels = snapshot.labels.clone();
            labels.retain(|label| !resolved.remove_labels.contains(label));
            labels.extend(resolved.add_labels.iter().cloned());
            sorted_labels(labels)
        };
        let expected_graph_labels = graph_labels(&expected_labels, &manifest.project_label);
        let actual_graph_labels = graph_labels(&issue.labels, &manifest.project_label);
        if expected_graph_labels != actual_graph_labels {
            anyhow::bail!(
                "published project plan is stale: issue `{}` graph labels changed (expected: {}; actual: {})",
                resolved.target,
                expected_graph_labels.join(", "),
                actual_graph_labels.join(", ")
            );
        }
    }
    Ok(())
}

pub fn validate_stage_mutations(manifest: &PlanningTransactionManifest) -> Result<()> {
    for mutation in &manifest.mutations {
        if mutation.stage != manifest.stage {
            anyhow::bail!(
                "planning mutation {} stage does not match manifest stage",
                mutation.order
            );
        }
        match manifest.stage {
            PlanningStage::Normalize => {
                if mutation.action != "update"
                    || mutation.certainty != "deterministic"
                    || mutation.requires_hitl
                {
                    anyhow::bail!(
                        "normalize manifest contains non-deterministic or non-update mutation {}",
                        mutation.order
                    );
                }
            }
            PlanningStage::Reconcile => {
                if mutation.action != "update" {
                    anyhow::bail!(
                        "reconcile manifest contains non-update mutation {}",
                        mutation.order
                    );
                }
            }
            PlanningStage::Atomize => {
                if mutation.action == "create" && !mutation.requires_hitl {
                    anyhow::bail!(
                        "atomize create mutation {} lacks human confirmation",
                        mutation.order
                    );
                }
            }
            PlanningStage::Verify => {
                anyhow::bail!("verify stage may not contain tracker mutations");
            }
        }
    }
    Ok(())
}

fn preflight_tracker_snapshot(
    manifest: &PlanningTransactionManifest,
    current: &[Issue],
    resolutions: &BTreeMap<String, String>,
) -> Result<()> {
    let relevant = current
        .iter()
        .filter(|issue| {
            issue
                .labels
                .iter()
                .any(|label| label == &manifest.project_label)
        })
        .map(|issue| (issue_key(issue), issue))
        .collect::<BTreeMap<_, _>>();
    let snapshot_ids = manifest
        .issue_snapshots
        .iter()
        .map(|snapshot| snapshot.id.clone())
        .collect::<BTreeSet<_>>();
    let created_ids = resolutions.values().cloned().collect::<BTreeSet<_>>();
    if let Some(unexpected) = relevant
        .keys()
        .find(|id| !snapshot_ids.contains(*id) && !created_ids.contains(*id))
    {
        anyhow::bail!(
            "tracker drift before planning transaction: new project issue `{unexpected}` was not reviewed"
        );
    }
    for snapshot in &manifest.issue_snapshots {
        let Some(issue) = relevant.get(&snapshot.id).copied() else {
            anyhow::bail!(
                "tracker drift before planning transaction: reviewed issue `{}` is missing",
                snapshot.id
            );
        };
        if snapshot.matches(issue) {
            continue;
        }
        let Some(mutation) = manifest
            .mutations
            .iter()
            .find(|mutation| mutation.action == "update" && mutation.target == snapshot.id)
        else {
            anyhow::bail!(
                "tracker drift before planning transaction: reviewed issue `{}` changed",
                snapshot.id
            );
        };
        let resolved = resolve_mutation(mutation, resolutions);
        if !snapshot.matches_after(issue, &resolved) {
            let fields = snapshot.drift_fields_after(issue, &resolved).join(", ");
            anyhow::bail!(
                "tracker drift before planning transaction: reviewed issue `{}` changed (fields: {})",
                snapshot.id,
                fields
            );
        }
    }
    Ok(())
}

impl PlanningIssueSnapshot {
    fn from_issue(issue: &Issue) -> Self {
        let mut snapshot = Self {
            id: issue_key(issue),
            issue_type: issue.issue_type.as_str().to_string(),
            title: issue.title.clone(),
            state: issue.state.as_str().to_string(),
            labels: sorted_labels(issue.labels.clone()),
            body: canonical_body(&issue.body),
            related: sorted_labels(issue.related.clone()),
            implements: sorted_labels(issue.implements.clone()),
            digest: String::new(),
        };
        snapshot.digest = snapshot_value_digest(&snapshot);
        snapshot
    }

    fn matches(&self, issue: &Issue) -> bool {
        self.digest == Self::from_issue(issue).digest
    }

    fn matches_after(&self, issue: &Issue, mutation: &PlanningMutation) -> bool {
        let mut expected = self.clone();
        apply_snapshot_mutation(&mut expected, mutation);
        expected.digest = snapshot_value_digest(&expected);
        expected.digest == Self::from_issue(issue).digest
    }

    fn drift_fields_after(&self, issue: &Issue, mutation: &PlanningMutation) -> Vec<&'static str> {
        let mut expected = self.clone();
        apply_snapshot_mutation(&mut expected, mutation);
        let actual = Self::from_issue(issue);
        let mut fields = Vec::new();
        if expected.issue_type != actual.issue_type {
            fields.push("type");
        }
        if expected.title != actual.title {
            fields.push("title");
        }
        if expected.state != actual.state {
            fields.push("state");
        }
        if expected.labels != actual.labels {
            fields.push("labels");
        }
        if expected.body != actual.body {
            fields.push("body");
        }
        if expected.related != actual.related {
            fields.push("related");
        }
        if expected.implements != actual.implements {
            fields.push("implements");
        }
        fields
    }
}

fn apply_snapshot_mutation(snapshot: &mut PlanningIssueSnapshot, mutation: &PlanningMutation) {
    if let Some(title) = &mutation.title {
        snapshot.title = title.clone();
    }
    if let Some(body) = &mutation.body {
        snapshot.body = canonical_body(body);
    }
    snapshot
        .labels
        .retain(|label| !mutation.remove_labels.contains(label));
    snapshot.labels.extend(mutation.add_labels.iter().cloned());
    snapshot.labels = sorted_labels(snapshot.labels.clone());
}

fn mutation_already_applied(
    mutation: &PlanningMutation,
    current: &[Issue],
    resolutions: &BTreeMap<String, String>,
) -> Option<String> {
    if mutation.action == "create" {
        return current
            .iter()
            .find(|issue| {
                issue
                    .body
                    .contains(&transaction_marker(&mutation.idempotency_key))
            })
            .map(issue_key);
    }
    let target = resolve_reference(&mutation.target, resolutions);
    let issue = current.iter().find(|issue| issue_key(issue) == target)?;
    mutation_matches_issue(mutation, issue).then_some(target)
}

fn mutation_matches_issue(mutation: &PlanningMutation, issue: &Issue) -> bool {
    mutation
        .title
        .as_ref()
        .is_none_or(|title| title == &issue.title)
        && mutation
            .body
            .as_ref()
            .is_none_or(|body| canonical_body(body) == canonical_body(&issue.body))
        && mutation
            .add_labels
            .iter()
            .all(|label| issue.labels.contains(label))
        && mutation
            .remove_labels
            .iter()
            .all(|label| !issue.labels.contains(label))
}

fn reconcile_created_targets(
    manifest: &PlanningTransactionManifest,
    current: &[Issue],
) -> BTreeMap<String, String> {
    manifest
        .mutations
        .iter()
        .filter(|mutation| mutation.action == "create")
        .filter_map(|mutation| {
            current
                .iter()
                .find(|issue| {
                    issue
                        .body
                        .contains(&transaction_marker(&mutation.idempotency_key))
                })
                .map(|issue| (mutation.target.clone(), issue_key(issue)))
        })
        .collect()
}

fn resolve_mutation(
    mutation: &PlanningMutation,
    resolutions: &BTreeMap<String, String>,
) -> PlanningMutation {
    let mut resolved = mutation.clone();
    resolved.target = resolve_reference(&resolved.target, resolutions);
    resolved.title = resolved
        .title
        .map(|value| resolve_text(&value, resolutions));
    resolved.body = resolved.body.map(|value| resolve_text(&value, resolutions));
    resolved.add_labels = resolved
        .add_labels
        .into_iter()
        .map(|value| resolve_text(&value, resolutions))
        .collect();
    resolved.remove_labels = resolved
        .remove_labels
        .into_iter()
        .map(|value| resolve_text(&value, resolutions))
        .collect();
    resolved
}

fn resolve_text(value: &str, resolutions: &BTreeMap<String, String>) -> String {
    let mut resolved = value.to_string();
    for (symbolic, actual) in resolutions {
        resolved = resolved.replace(symbolic, actual);
    }
    resolved
}

fn resolve_reference(value: &str, resolutions: &BTreeMap<String, String>) -> String {
    resolutions
        .get(value)
        .cloned()
        .unwrap_or_else(|| value.to_string())
}

fn issue_from_create_mutation(mutation: &PlanningMutation) -> Result<Issue> {
    let issue_type = IssueType::parse(&mutation.issue_type).ok_or_else(|| {
        anyhow::anyhow!("unsupported planning issue type `{}`", mutation.issue_type)
    })?;
    let mut issue: Issue = serde_json::from_value(serde_json::json!({
        "type": issue_type.as_str(),
        "title": mutation.title.clone().unwrap_or_default(),
        "state": "open",
    }))?;
    issue.body = mutation.body.clone().unwrap_or_default();
    issue.labels = mutation.add_labels.clone();
    Ok(issue)
}

fn patch_from_mutation(mutation: &PlanningMutation) -> IssuePatch {
    IssuePatch {
        title: mutation.title.clone(),
        body: mutation.body.clone(),
        add_labels: mutation.add_labels.clone(),
        remove_labels: mutation.remove_labels.clone(),
        ..IssuePatch::default()
    }
}

fn replace_current_issue(current: &mut Vec<Issue>, updated: Issue) {
    let id = issue_key(&updated);
    if let Some(index) = current.iter().position(|issue| issue_key(issue) == id) {
        current[index] = updated;
    } else {
        current.push(updated);
    }
}

fn managed_label_delta(
    issue: &Issue,
    desired: &[String],
    include_change_relations: bool,
) -> (Vec<String>, Vec<String>) {
    let desired = desired.iter().cloned().collect::<BTreeSet<_>>();
    let managed = |label: &str| {
        label.starts_with("type:")
            || label.starts_with("priority:")
            || (include_change_relations
                && [
                    "epic:",
                    "parent-epic:",
                    "parent:",
                    "depends-on:",
                    "duplicate-of:",
                    "superseded-by:",
                ]
                .iter()
                .any(|prefix| label.starts_with(prefix)))
    };
    let current = issue.labels.iter().cloned().collect::<BTreeSet<_>>();
    let add = desired.difference(&current).cloned().collect::<Vec<_>>();
    let remove = current
        .iter()
        .filter(|label| managed(label) && !desired.contains(*label))
        .cloned()
        .collect::<Vec<_>>();
    (add, remove)
}

fn graph_labels(labels: &[String], project_label: &str) -> Vec<String> {
    sorted_labels(
        labels
            .iter()
            .filter(|label| {
                label.as_str() == project_label
                    || [
                        "type:",
                        "priority:",
                        "epic:",
                        "parent-epic:",
                        "parent:",
                        "depends-on:",
                        "duplicate-of:",
                        "supersedes:",
                        "superseded-by:",
                    ]
                    .iter()
                    .any(|prefix| label.starts_with(prefix))
            })
            .cloned(),
    )
}

fn append_transaction_note(body: &str, key: &str, note: &str) -> String {
    let marker = transaction_marker(key);
    if body.contains(&marker) {
        return body.to_string();
    }
    format!(
        "{}{}\n\n## Reviewed Planning Transaction\n\n{}\n{}\n",
        body.trim_end(),
        if body.trim().is_empty() { "" } else { "\n\n" },
        note,
        marker,
    )
}

/// Replace every legacy body parent declaration with one canonical value before
/// an ownership update. Keeping the old `Parent:` alongside a new `epic:`
/// label would make the strict graph correctly report two owners.
fn canonical_parent_body(body: &str, owner_epic: &str) -> String {
    let mut output = Vec::new();
    let mut wrote_parent = false;
    for line in body.lines() {
        if is_parent_declaration_line(line) {
            if !wrote_parent {
                output.push(format!("Parent Epic: #{owner_epic}"));
                wrote_parent = true;
            }
        } else {
            output.push(line.to_string());
        }
    }
    if !wrote_parent {
        output.push(format!("Parent Epic: #{owner_epic}"));
    }
    output.join("\n")
}

fn is_parent_declaration_line(line: &str) -> bool {
    let cleaned = line
        .trim()
        .trim_start_matches(|ch: char| matches!(ch, '-' | '*' | ' '))
        .replace("**", "")
        .replace('`', "");
    let lower = cleaned.trim().to_ascii_lowercase();
    ["parent epic:", "parent wi:", "parent:"]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

fn transaction_marker(key: &str) -> String {
    format!("<!-- aw:planning-transaction:{key} -->")
}

fn mutation_key(plan_digest: &str, action: &str, target: &str) -> String {
    format!(
        "sha256:{:x}",
        Sha256::digest(format!("{plan_digest}\0{action}\0{target}").as_bytes())
    )
}

fn mutation_action_rank(action: &str, issue_type: &str) -> u8 {
    match (action, issue_type) {
        ("create", "epic") => 0,
        ("create", _) => 1,
        ("update", "epic") => 2,
        _ => 3,
    }
}

fn sorted_labels(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn issue_capability(issue: &Issue) -> Option<String> {
    issue.body.lines().find_map(|line| {
        let trimmed = line.trim().replace("**", "").replace('`', "");
        let (field, value) = trimmed.split_once(':')?;
        if !field.trim().eq_ignore_ascii_case("capability") {
            return None;
        }
        let value = value.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn canonical_body(body: &str) -> String {
    let normalized = body.replace("\r\n", "\n");
    let mut lines = Vec::new();
    let mut previous_blank = true;
    for line in normalized.lines() {
        let line = line.trim_end();
        let blank = line.is_empty();
        if blank && previous_blank {
            continue;
        }
        lines.push(line);
        previous_blank = blank;
    }
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

fn snapshot_digest(snapshots: &[PlanningIssueSnapshot]) -> String {
    format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(snapshots).expect("snapshot serializes"))
    )
}

fn snapshot_value_digest(snapshot: &PlanningIssueSnapshot) -> String {
    let mut payload = snapshot.clone();
    payload.digest.clear();
    format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(&payload).expect("issue snapshot serializes"))
    )
}

fn load_checkpoint(path: &Path, source_digest: &str) -> Result<PlanningTransactionCheckpoint> {
    if !path.exists() {
        return Ok(PlanningTransactionCheckpoint {
            schema: PLANNING_TRANSACTION_SCHEMA.to_string(),
            source_digest: source_digest.to_string(),
            status: "pending".to_string(),
            results: Vec::new(),
        });
    }
    let checkpoint: PlanningTransactionCheckpoint = serde_json::from_str(
        &std::fs::read_to_string(path)
            .with_context(|| format!("read planning checkpoint {}", path.display()))?,
    )?;
    if checkpoint.source_digest != source_digest {
        anyhow::bail!(
            "planning transaction checkpoint {} belongs to a different reviewed digest",
            path.display()
        );
    }
    Ok(checkpoint)
}

fn write_checkpoint(path: &Path, checkpoint: &PlanningTransactionCheckpoint) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("transaction.tmp");
    std::fs::write(
        &temp,
        format!("{}\n", serde_json::to_string_pretty(checkpoint)?),
    )?;
    std::fs::rename(&temp, path)?;
    Ok(())
}

fn reference_sort_key(value: &str) -> (u8, u64, String) {
    match value.parse::<u64>() {
        Ok(number) => (0, number, String::new()),
        Err(_) => (1, u64::MAX, value.to_ascii_lowercase()),
    }
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct FailingBackend {
        issues: Mutex<Vec<Issue>>,
        fail_after_first_create_write: Mutex<bool>,
    }

    impl FailingBackend {
        fn new(issues: Vec<Issue>, fail_after_first_create_write: bool) -> Self {
            Self {
                issues: Mutex::new(issues),
                fail_after_first_create_write: Mutex::new(fail_after_first_create_write),
            }
        }

        fn snapshot(&self) -> Vec<Issue> {
            self.issues.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl IssueBackend for FailingBackend {
        fn name(&self) -> &'static str {
            "fixture"
        }

        async fn list(&self, _filter: &IssueFilter) -> Result<Vec<Issue>> {
            Ok(self.snapshot())
        }

        async fn get(&self, id: &str) -> Result<Option<Issue>> {
            Ok(self
                .snapshot()
                .into_iter()
                .find(|issue| issue_key(issue) == id))
        }

        async fn write(&self, issue: &Issue) -> Result<()> {
            replace_current_issue(&mut self.issues.lock().unwrap(), issue.clone());
            Ok(())
        }

        async fn create(&self, issue: &Issue) -> Result<Issue> {
            let mut created = issue.clone();
            let sequence = self.issues.lock().unwrap().len() + 1;
            created.slug = format!("created-{sequence}");
            self.issues.lock().unwrap().push(created.clone());
            let mut fail = self.fail_after_first_create_write.lock().unwrap();
            if *fail {
                *fail = false;
                anyhow::bail!("fixture transport failed after remote create");
            }
            Ok(created)
        }

        async fn update(&self, id: &str, patch: &IssuePatch) -> Result<Issue> {
            let mut issues = self.issues.lock().unwrap();
            let issue = issues
                .iter_mut()
                .find(|issue| issue_key(issue) == id)
                .ok_or_else(|| anyhow::anyhow!("missing fixture issue {id}"))?;
            patch.apply(issue);
            Ok(issue.clone())
        }

        async fn close(&self, _id: &str, _reason: Option<&str>) -> Result<()> {
            anyhow::bail!("close is not part of a planning transaction")
        }
    }

    fn epic() -> Issue {
        let mut issue: Issue = serde_json::from_value(serde_json::json!({
            "type": "epic",
            "title": "Reviewed delivery",
            "state": "open",
            "github_id": 42,
            "labels": ["type:epic", "app:demo", "priority:p1"]
        }))
        .unwrap();
        issue.slug = "42".to_string();
        issue.body = "## Requirements\n\n- R1: Publish one reviewed atomic change.\n".to_string();
        issue
    }

    fn manifest(issue: &Issue) -> PlanningTransactionManifest {
        let plan =
            crate::issues::build_project_plan("demo", "app:demo", std::slice::from_ref(issue));
        assert!(plan.valid);
        build_planning_transaction_manifest(
            &plan,
            std::slice::from_ref(issue),
            "aw wi plan-review --evidence-file /tmp/review.json",
            "aw wi graph --project demo --json",
        )
    }

    #[tokio::test]
    async fn proposed_requirement_dependencies_create_in_order_and_resolve_labels() {
        let mut source = epic();
        source.body = "## Requirements\n\n- R1: Publish the native Python EC replacement.\n- R2: Delete the delegated Rust EC wrapper.\n\n## Verification Inventory\n\n| Requirement | Gate | Oracle | Depends On |\n|-------------|------|--------|------------|\n| R1 | `python3 native_ec.py` | Native EC passes. | - |\n| R2 | `test ! -e delegated.rs` | Delegated wrapper is absent. | R1 |\n".to_string();
        let manifest = manifest(&source);
        let creates = manifest
            .mutations
            .iter()
            .filter(|mutation| mutation.action == "create")
            .collect::<Vec<_>>();
        assert_eq!(creates.len(), 2);
        assert!(creates[0].target.ends_with("requirement-1"));
        assert!(creates[1].target.ends_with("requirement-2"));
        assert!(creates[1]
            .add_labels
            .contains(&format!("depends-on:{}", creates[0].target)));

        let backend = FailingBackend::new(vec![source], false);
        let temp = tempfile::tempdir().unwrap();
        let checkpoint = temp.path().join("transaction.json");
        apply_planning_transaction(&backend, &manifest, "review-digest", &checkpoint)
            .await
            .unwrap();

        let published = backend.snapshot();
        let prerequisite = published
            .iter()
            .find(|issue| issue.title.contains("Publish the native Python EC"))
            .map(issue_key)
            .unwrap();
        let dependent = published
            .iter()
            .find(|issue| issue.title.contains("Delete the delegated Rust EC"))
            .unwrap();
        assert!(dependent
            .labels
            .contains(&format!("depends-on:{prerequisite}")));
    }

    #[tokio::test]
    async fn transport_failure_after_create_reconciles_without_duplicate() {
        let source = epic();
        let manifest = manifest(&source);
        assert_eq!(
            manifest
                .mutations
                .iter()
                .filter(|mutation| mutation.action == "create")
                .count(),
            1
        );
        let backend = FailingBackend::new(vec![source], true);
        let temp = tempfile::tempdir().unwrap();
        let checkpoint = temp.path().join("transaction.json");

        let first = apply_planning_transaction(&backend, &manifest, "review-digest", &checkpoint)
            .await
            .unwrap_err();
        assert!(format!("{first:#}").contains("after remote create"));
        assert_eq!(backend.snapshot().len(), 2);

        let resumed = apply_planning_transaction(&backend, &manifest, "review-digest", &checkpoint)
            .await
            .unwrap();
        assert_eq!(backend.snapshot().len(), 2);
        assert_eq!(resumed.reconciled_count, 1);
        assert_eq!(resumed.applied_count, 0);

        let repeated =
            apply_planning_transaction(&backend, &manifest, "review-digest", &checkpoint)
                .await
                .unwrap();
        assert!(repeated.no_op);
        assert_eq!(repeated.applied_count, 0);
        assert_eq!(repeated.reconciled_count, manifest.mutations.len());
        assert_eq!(backend.snapshot().len(), 2);
    }

    #[tokio::test]
    async fn tracker_drift_fails_before_first_mutation_and_names_issue() {
        let source = epic();
        let manifest = manifest(&source);
        let mut drifted = source;
        drifted.title = "Externally changed title".to_string();
        let backend = FailingBackend::new(vec![drifted], false);
        let temp = tempfile::tempdir().unwrap();
        let checkpoint = temp.path().join("transaction.json");

        let error = apply_planning_transaction(&backend, &manifest, "review-digest", &checkpoint)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("reviewed issue `42` changed"));
        assert_eq!(backend.snapshot().len(), 1);
        assert!(!checkpoint.exists());
    }

    #[tokio::test]
    async fn published_verifier_allows_lifecycle_drift_but_rejects_graph_drift() {
        let source = epic();
        let manifest = manifest(&source);
        let backend = FailingBackend::new(vec![source], false);
        let temp = tempfile::tempdir().unwrap();
        let checkpoint = temp.path().join("transaction.json");

        apply_planning_transaction(&backend, &manifest, "review-digest", &checkpoint)
            .await
            .unwrap();
        verify_published_planning_transaction(
            &manifest,
            "review-digest",
            &checkpoint,
            &backend.snapshot(),
        )
        .unwrap();

        let mut lifecycle_advanced = backend.snapshot();
        let epic = lifecycle_advanced
            .iter_mut()
            .find(|issue| issue_key(issue) == "42")
            .unwrap();
        epic.state = crate::issues::IssueState::Closed;
        epic.labels.push("phase:done".to_string());
        epic.body
            .push_str("\nLifecycle evidence advanced after publication.\n");
        verify_published_planning_transaction(
            &manifest,
            "review-digest",
            &checkpoint,
            &lifecycle_advanced,
        )
        .unwrap();

        let epic = lifecycle_advanced
            .iter_mut()
            .find(|issue| issue_key(issue) == "42")
            .unwrap();
        epic.labels.retain(|label| !label.starts_with("priority:"));
        epic.labels.push("priority:p3".to_string());
        let error = verify_published_planning_transaction(
            &manifest,
            "review-digest",
            &checkpoint,
            &lifecycle_advanced,
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("issue `42` graph labels changed"));
    }

    #[test]
    fn checkpoint_path_is_scoped_to_the_exact_stage_digest() {
        let plan_path = Path::new("/tmp/project-plan.json");
        let first =
            planning_transaction_checkpoint_path(plan_path, PlanningStage::Reconcile, "digest-a");
        let second =
            planning_transaction_checkpoint_path(plan_path, PlanningStage::Reconcile, "digest-b");

        assert_ne!(first, second);
        assert!(first.ends_with("project-plan.reconcile.digest-a.transaction.json"));
        assert!(second.ends_with("project-plan.reconcile.digest-b.transaction.json"));
    }
}
