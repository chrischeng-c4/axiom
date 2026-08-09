// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/logic/issues/two-stage-project-plan.md#logic
// HANDWRITE-BEGIN gap="missing-generator:rust:two-stage-project-planner" tracker="#2387" reason="The deterministic planner is a new issue-domain aggregate; the spec defines its projection while the generator does not yet emit Rust planning logic."

//! Canonical staged epic/change planning projection.

use super::{build_work_item_graph, issue_key, GraphDiagnostic, GraphNext, Issue, IssueState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const PROJECT_PLAN_SCHEMA: &str = "aw.wi.project-plan.v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanningStage {
    Normalize,
    Reconcile,
    Atomize,
    Verify,
}

impl PlanningStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normalize => "normalize",
            Self::Reconcile => "reconcile",
            Self::Atomize => "atomize",
            Self::Verify => "verify",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectPlan {
    pub schema: String,
    pub action: String,
    pub root_id: String,
    pub stage: PlanningStage,
    pub project: String,
    pub project_label: String,
    pub source_graph_digest: String,
    pub digest: String,
    pub valid: bool,
    pub stages: Vec<ProjectPlanStage>,
    pub epic_order: Vec<String>,
    pub epics: Vec<PlannedEpic>,
    pub proposed_epics: Vec<ProposedEpic>,
    pub changes: Vec<PlannedChange>,
    pub proposed_changes: Vec<ProposedChange>,
    pub diagnostics: Vec<GraphDiagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<GraphNext>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectPlanStage {
    pub order: u8,
    pub name: String,
    pub operation: String,
    pub node_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedEpic {
    pub id: String,
    pub title: String,
    pub horizon: String,
    pub priority: String,
    pub order: usize,
    pub requirements: Vec<PlanRequirement>,
    pub change_ids: Vec<String>,
    pub split_into: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedEpic {
    pub id: String,
    pub title: String,
    pub source_epic: String,
    pub horizon: String,
    pub priority: String,
    pub requirements: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanRequirement {
    pub id: String,
    pub text: String,
    pub horizon: String,
    pub owner_epic: String,
    pub covered_by: Vec<String>,
    pub status: String,
    pub verification: Vec<RequirementVerification>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementVerification {
    pub gate: String,
    pub oracle: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedChange {
    pub id: String,
    pub title: String,
    pub owner_epic: String,
    pub priority: String,
    pub priority_source: String,
    pub lane: String,
    pub dependencies: Vec<String>,
    pub covers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_of: Option<String>,
    pub replacement_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedChange {
    pub id: String,
    pub title: String,
    pub owner_epic: String,
    pub priority: String,
    pub priority_source: String,
    pub lane: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub covers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_change: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone)]
struct EpicSeed {
    id: String,
    title: String,
    priority: String,
    horizon: String,
    requirements: Vec<(String, String)>,
    change_ids: Vec<String>,
    closed_change_ids: Vec<String>,
    completed_child_plan_ids: Vec<String>,
    explicit_requirement_coverage: BTreeMap<usize, Vec<String>>,
    requirement_verification: BTreeMap<usize, Vec<RequirementVerification>>,
    requirement_dependencies: BTreeMap<usize, Vec<usize>>,
    split_into: Vec<String>,
}

/// Build the one authoritative read-only plan for a project's complete issue
/// inventory. Structural graph defects fail closed. Unowned open changes are
/// the one bootstrap exception: the planner groups them by declared capability
/// context and scheduling horizon, proposes explicit epic owners, and leaves
/// that complete mapping for independent review before publication.
pub fn build_project_plan(project: &str, project_label: &str, issues: &[Issue]) -> ProjectPlan {
    build_project_plan_for_stage(project, project_label, issues, PlanningStage::Atomize)
}

pub fn build_project_plan_for_stage(
    project: &str,
    project_label: &str,
    issues: &[Issue],
    stage: PlanningStage,
) -> ProjectPlan {
    let mut plan = build_full_project_plan(project, project_label, issues, stage);
    match stage {
        PlanningStage::Normalize => {
            plan.proposed_epics.clear();
            plan.proposed_changes.clear();
            for epic in &mut plan.epics {
                epic.split_into.clear();
            }
            for change in &mut plan.changes {
                change.duplicate_of = None;
                change.replacement_ids.clear();
            }
        }
        PlanningStage::Reconcile => {
            plan.proposed_epics.clear();
            plan.proposed_changes.clear();
            for epic in &mut plan.epics {
                epic.split_into.clear();
            }
            for change in &mut plan.changes {
                change.replacement_ids.clear();
            }
        }
        PlanningStage::Atomize | PlanningStage::Verify => {
            if plan
                .proposed_epics
                .iter()
                .any(|epic| epic.reason == "unowned_change_bootstrap")
            {
                plan.valid = false;
                plan.action = "blocked".to_string();
                plan.proposed_epics
                    .retain(|epic| epic.reason != "unowned_change_bootstrap");
                plan.proposed_changes
                    .retain(|change| !change.owner_epic.starts_with("proposal:epic:bootstrap:"));
                plan.next = Some(GraphNext {
                    command: format!(
                        "aw wi plan --project {project} --stage reconcile --root {} --json",
                        plan.root_id
                    ),
                    reason: "unowned changes require explicit human owner decisions".to_string(),
                });
            }
            if let Some(epic) = plan
                .epics
                .iter()
                .find(|epic| !epic.id.starts_with("proposal:") && epic.requirements.is_empty())
            {
                plan.valid = false;
                plan.action = "blocked".to_string();
                plan.diagnostics.push(GraphDiagnostic {
                    code: "missing_authoritative_requirements".to_string(),
                    issue: epic.id.clone(),
                    related: None,
                    message: format!(
                        "epic `{}` has no authoritative `## Requirements` entries",
                        epic.id
                    ),
                    remediation_target: format!("Requirements on epic {}", epic.id),
                    next_command: format!("aw wi show {}", epic.id),
                });
                plan.next = Some(GraphNext {
                    command: format!("aw wi show {}", epic.id),
                    reason: "atomization requires authoritative Requirements".to_string(),
                });
            }
        }
    }
    with_digest(plan)
}

fn build_full_project_plan(
    project: &str,
    project_label: &str,
    issues: &[Issue],
    stage: PlanningStage,
) -> ProjectPlan {
    let graph = build_work_item_graph(project, project_label, issues);
    let stages = vec![
        ProjectPlanStage {
            order: 1,
            name: "epic_inventory".to_string(),
            operation: "atomize_and_prioritize".to_string(),
            node_type: "epic".to_string(),
        },
        ProjectPlanStage {
            order: 2,
            name: "change_inventory_by_epic".to_string(),
            operation: "reconcile_atomize_and_prioritize".to_string(),
            node_type: "change".to_string(),
        },
    ];
    let fatal_diagnostics = graph
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code != "unowned_change")
        .cloned()
        .collect::<Vec<_>>();
    if !fatal_diagnostics.is_empty() {
        let next = fatal_diagnostics.first().map(|diagnostic| GraphNext {
            command: diagnostic.next_command.clone(),
            reason: diagnostic.message.clone(),
        });
        return ProjectPlan {
            schema: PROJECT_PLAN_SCHEMA.to_string(),
            action: "blocked".to_string(),
            root_id: project_plan_root_id(project, &graph.digest),
            stage,
            project: project.to_string(),
            project_label: project_label.to_string(),
            source_graph_digest: graph.digest,
            digest: String::new(),
            valid: false,
            stages,
            epic_order: Vec::new(),
            epics: Vec::new(),
            proposed_epics: Vec::new(),
            changes: Vec::new(),
            proposed_changes: Vec::new(),
            diagnostics: fatal_diagnostics,
            next,
        };
    }

    let issue_by_id = issues
        .iter()
        .map(|issue| (issue_key(issue), issue))
        .collect::<BTreeMap<_, _>>();
    let active_ids = issues
        .iter()
        .filter(|issue| is_active_plan_issue(issue))
        .map(issue_key)
        .collect::<BTreeSet<_>>();
    let published_split_sources = published_split_sources(issues);

    // Stage 1: decompose mixed-horizon epics before assigning their stable
    // project order. The source epic remains visible for tracker reconciliation.
    let mut proposed_epics = Vec::new();
    let mut seeds = graph
        .epics
        .iter()
        .filter(|epic| active_ids.contains(&epic.id))
        .filter_map(|epic| {
            let source = issue_by_id.get(&epic.id).copied()?;
            let requirements = extract_requirements(source);
            let active = requirements
                .iter()
                .filter(|(_, horizon)| horizon == "active")
                .map(|(text, _)| text.clone())
                .collect::<Vec<_>>();
            let deferred = requirements
                .iter()
                .filter(|(_, horizon)| horizon == "deferred")
                .map(|(text, _)| text.clone())
                .collect::<Vec<_>>();
            let horizon = match (active.is_empty(), deferred.is_empty()) {
                (false, false) => "mixed",
                (true, false) => "deferred",
                _ => "active",
            }
            .to_string();
            if horizon == "mixed" && published_split_sources.contains(&epic.id) {
                return None;
            }
            let priority = epic.priority.clone().unwrap_or_else(|| "p3".to_string());
            let explicit_child_ids = explicit_child_work_item_ids(source);
            let explicit_requirement_coverage = explicit_child_requirement_coverage(source);
            let requirement_verification = explicit_requirement_verification(source);
            let requirement_dependencies = explicit_requirement_dependencies(source);
            let completed_child_plan_ids = if !explicit_child_ids.is_empty()
                && explicit_child_ids.iter().all(|id| {
                    issue_by_id
                        .get(id)
                        .is_some_and(|issue| issue.state == IssueState::Closed)
                }) {
                explicit_child_ids
            } else {
                Vec::new()
            };
            let mut split_into = Vec::new();
            if horizon == "mixed" {
                let active_id = format!("proposal:epic:{}:active", epic.id);
                let deferred_id = format!("proposal:epic:{}:deferred", epic.id);
                split_into.extend([active_id.clone(), deferred_id.clone()]);
                proposed_epics.push(ProposedEpic {
                    id: active_id,
                    title: format!("{} - active", epic.title),
                    source_epic: epic.id.clone(),
                    horizon: "active".to_string(),
                    priority: priority.clone(),
                    requirements: active,
                    reason: "mixed_horizon_split".to_string(),
                });
                proposed_epics.push(ProposedEpic {
                    id: deferred_id,
                    title: format!("{} - deferred", epic.title),
                    source_epic: epic.id.clone(),
                    horizon: "deferred".to_string(),
                    priority: "p3".to_string(),
                    requirements: deferred,
                    reason: "mixed_horizon_split".to_string(),
                });
            }
            Some(EpicSeed {
                id: epic.id.clone(),
                title: epic.title.clone(),
                priority,
                horizon,
                requirements,
                change_ids: epic
                    .children
                    .iter()
                    .filter(|id| active_ids.contains(*id))
                    .cloned()
                    .collect(),
                closed_change_ids: epic
                    .children
                    .iter()
                    .filter(|id| {
                        issue_by_id
                            .get(*id)
                            .is_some_and(|issue| issue.state == IssueState::Closed)
                    })
                    .cloned()
                    .collect(),
                completed_child_plan_ids,
                explicit_requirement_coverage,
                requirement_verification,
                requirement_dependencies,
                split_into,
            })
        })
        .collect::<Vec<_>>();

    // A project migration starts with legacy unowned changes by definition.
    // Group them into deterministic DDD/horizon epic proposals so the plan can
    // authorize one coherent owner-assignment transaction. The graph command
    // itself remains strict; only the reviewed planner owns this bootstrap.
    let mut bootstrap_groups = BTreeMap::<(String, String), (String, Vec<String>)>::new();
    for change in graph
        .changes
        .iter()
        .filter(|change| active_ids.contains(&change.id) && change.parent.is_none())
    {
        let Some(issue) = issue_by_id.get(&change.id).copied() else {
            continue;
        };
        let context = capability_context(issue);
        let context_key = identifier_slug(&context);
        let horizon = if is_deferred_issue(issue) {
            "deferred"
        } else {
            "active"
        }
        .to_string();
        bootstrap_groups
            .entry((context_key, horizon))
            .or_insert_with(|| (context.clone(), Vec::new()))
            .1
            .push(change.id.clone());
    }
    for ((context_key, horizon), (context, mut change_ids)) in bootstrap_groups {
        change_ids.sort_by(|left, right| reference_sort_key(left).cmp(&reference_sort_key(right)));
        let priority = if horizon == "deferred" {
            "p3".to_string()
        } else {
            change_ids
                .iter()
                .filter_map(|id| issue_by_id.get(id).copied())
                .filter_map(explicit_issue_priority)
                .min_by_key(|priority| priority_rank(priority))
                .unwrap_or_else(|| "p2".to_string())
        };
        let proposal_id = format!("proposal:epic:bootstrap:{context_key}:{horizon}");
        let requirements = change_ids
            .iter()
            .filter_map(|id| issue_by_id.get(id).map(|issue| issue.title.clone()))
            .collect::<Vec<_>>();
        proposed_epics.push(ProposedEpic {
            id: proposal_id.clone(),
            title: format!("AW {context} {horizon} backlog"),
            source_epic: format!("bootstrap:{context_key}"),
            horizon: horizon.clone(),
            priority: priority.clone(),
            requirements: requirements.clone(),
            reason: "unowned_change_bootstrap".to_string(),
        });
        seeds.push(EpicSeed {
            id: proposal_id,
            title: format!("AW {context} {horizon} backlog"),
            priority,
            horizon: horizon.clone(),
            requirements: requirements
                .into_iter()
                .map(|requirement| (requirement, horizon.clone()))
                .collect(),
            change_ids,
            closed_change_ids: Vec::new(),
            completed_child_plan_ids: Vec::new(),
            explicit_requirement_coverage: BTreeMap::new(),
            requirement_verification: BTreeMap::new(),
            requirement_dependencies: BTreeMap::new(),
            split_into: Vec::new(),
        });
    }
    seeds.sort_by(|left, right| {
        priority_rank(&left.priority)
            .cmp(&priority_rank(&right.priority))
            .then_with(|| horizon_rank(&left.horizon).cmp(&horizon_rank(&right.horizon)))
            .then_with(|| reference_sort_key(&left.id).cmp(&reference_sort_key(&right.id)))
    });
    proposed_epics.sort_by(|left, right| {
        priority_rank(&left.priority)
            .cmp(&priority_rank(&right.priority))
            .then_with(|| reference_sort_key(&left.id).cmp(&reference_sort_key(&right.id)))
    });

    // Stage 2: reconcile each epic's existing changes before proposing only
    // missing atomic siblings. Duplicate and oversized leaves never satisfy a
    // requirement on their own.
    let graph_changes = graph
        .changes
        .iter()
        .map(|change| (change.id.clone(), change))
        .collect::<BTreeMap<_, _>>();
    let mut changes = Vec::new();
    let mut proposed_changes = Vec::new();
    let mut epics = Vec::new();
    for (order, seed) in seeds.iter().enumerate() {
        let owned = seed
            .change_ids
            .iter()
            .filter_map(|id| {
                Some((
                    id.clone(),
                    issue_by_id.get(id).copied()?,
                    graph_changes.get(id).copied()?,
                ))
            })
            .collect::<Vec<_>>();
        let inferred_duplicates = inferred_duplicate_targets(&owned);
        let mut coverage = BTreeMap::<String, Vec<String>>::new();
        for (index, _) in seed.requirements.iter().enumerate() {
            if !seed.completed_child_plan_ids.is_empty() {
                coverage.insert(
                    requirement_id(&seed.id, index),
                    seed.completed_child_plan_ids.clone(),
                );
            }
        }
        for (index, child_ids) in &seed.explicit_requirement_coverage {
            let requirement = requirement_id(&seed.id, *index);
            for child_id in child_ids {
                let delivered = seed.closed_change_ids.contains(child_id);
                let active_valid = owned.iter().find(|(id, _, _)| id == child_id).is_some_and(
                    |(id, issue, graph_change)| {
                        graph_change.duplicate_of.is_none()
                            && !inferred_duplicates.contains_key(id)
                            && !looks_too_large_for_atomic_wi(issue)
                            && is_structured_change(issue)
                    },
                );
                if delivered || active_valid {
                    coverage
                        .entry(requirement.clone())
                        .or_default()
                        .push(child_id.clone());
                }
            }
        }
        for change_id in &seed.closed_change_ids {
            let Some(issue) = issue_by_id.get(change_id).copied() else {
                continue;
            };
            for (index, (requirement, _)) in seed.requirements.iter().enumerate() {
                if normalized_text(requirement) == normalized_text(&issue.title)
                    || text_covers(issue, requirement)
                {
                    coverage
                        .entry(requirement_id(&seed.id, index))
                        .or_default()
                        .push(change_id.clone());
                }
            }
        }
        for (change_id, issue, graph_change) in &owned {
            let duplicate_of = graph_change
                .duplicate_of
                .clone()
                .or_else(|| inferred_duplicates.get(change_id).cloned());
            let oversized = looks_too_large_for_atomic_wi(issue);
            let structured = is_structured_change(issue);
            let deferred = is_deferred_issue(issue);
            let owner_epic = requirement_owner(seed, if deferred { "deferred" } else { "active" });
            let covers = seed
                .requirements
                .iter()
                .enumerate()
                .filter(|(_, (requirement, _))| text_covers(issue, requirement))
                .map(|(index, _)| requirement_id(&seed.id, index))
                .collect::<Vec<_>>();
            let identity_coverage = seed.requirements.iter().any(|(requirement, _)| {
                normalized_text(requirement) == normalized_text(&issue.title)
            });
            if duplicate_of.is_none() && !oversized && (structured || identity_coverage) {
                for requirement in &covers {
                    let requirement_horizon = requirement_index(requirement)
                        .and_then(|index| seed.requirements.get(index))
                        .map(|(_, horizon)| horizon.as_str());
                    if requirement_horizon == Some(if deferred { "deferred" } else { "active" }) {
                        coverage
                            .entry(requirement.clone())
                            .or_default()
                            .push(change_id.clone());
                    }
                }
            }

            let open_dependencies = graph_change
                .dependencies
                .iter()
                .filter(|dependency| active_ids.contains(*dependency))
                .cloned()
                .collect::<Vec<_>>();
            let lane = if deferred {
                "deferred"
            } else if duplicate_of.is_some() {
                "duplicate"
            } else if oversized {
                "needs_atomize"
            } else if !structured {
                "needs_triage"
            } else if !open_dependencies.is_empty() {
                "blocked_by_dependency"
            } else {
                "ready_now"
            };

            let mut replacement_ids = Vec::new();
            if oversized && duplicate_of.is_none() {
                let eligible_covers = covers
                    .iter()
                    .filter(|requirement| {
                        requirement_index(requirement)
                            .and_then(|index| seed.requirements.get(index))
                            .map(|(_, horizon)| horizon.as_str())
                            == Some(if deferred { "deferred" } else { "active" })
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                for part in 1..=2 {
                    let proposal_id = format!("proposal:change:{change_id}:part-{part}");
                    let proposal_covers = eligible_covers
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| index % 2 == part - 1)
                        .map(|(_, requirement)| requirement.clone())
                        .collect::<Vec<_>>();
                    if proposal_covers.is_empty() {
                        continue;
                    }
                    for requirement in &proposal_covers {
                        coverage
                            .entry(requirement.clone())
                            .or_default()
                            .push(proposal_id.clone());
                    }
                    replacement_ids.push(proposal_id.clone());
                    proposed_changes.push(ProposedChange {
                        id: proposal_id,
                        title: bounded_replacement_title(issue, part),
                        owner_epic: owner_epic.clone(),
                        priority: graph_change
                            .priority
                            .value
                            .clone()
                            .unwrap_or_else(|| seed.priority.clone()),
                        priority_source: "replacement_inherits_source".to_string(),
                        lane: "proposed".to_string(),
                        dependencies: Vec::new(),
                        covers: proposal_covers,
                        source_change: Some(change_id.clone()),
                        reason: "oversized_change_replacement".to_string(),
                    });
                }
            }
            changes.push(PlannedChange {
                id: change_id.clone(),
                title: issue.title.clone(),
                owner_epic,
                priority: graph_change
                    .priority
                    .value
                    .clone()
                    .unwrap_or_else(|| seed.priority.clone()),
                priority_source: graph_change.priority.source.clone(),
                lane: lane.to_string(),
                dependencies: graph_change.dependencies.clone(),
                covers,
                duplicate_of,
                replacement_ids,
            });
        }

        let requirement_coverage = coverage.clone();
        let requirements = seed
            .requirements
            .iter()
            .enumerate()
            .map(|(index, (text, horizon))| {
                let id = requirement_id(&seed.id, index);
                let dependency_ids = seed
                    .requirement_dependencies
                    .get(&index)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|dependency_index| requirement_id(&seed.id, dependency_index))
                    .collect::<Vec<_>>();
                let change_dependencies = seed
                    .requirement_dependencies
                    .get(&index)
                    .into_iter()
                    .flatten()
                    .flat_map(|dependency_index| {
                        let dependency_id = requirement_id(&seed.id, *dependency_index);
                        let mut references = requirement_coverage
                            .get(&dependency_id)
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|reference| {
                                reference.starts_with("proposal:")
                                    || issue_by_id
                                        .get(reference)
                                        .is_some_and(|issue| issue.state != IssueState::Closed)
                            })
                            .collect::<Vec<_>>();
                        if references.is_empty() {
                            references.push(format!(
                                "proposal:change:{}:requirement-{}",
                                seed.id,
                                dependency_index + 1
                            ));
                        }
                        references
                    })
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                let mut covered_by = coverage.remove(&id).unwrap_or_default();
                covered_by.sort_by(|left, right| {
                    reference_sort_key(left).cmp(&reference_sort_key(right))
                });
                covered_by.dedup();
                let owner_epic = requirement_owner(seed, horizon);
                if covered_by.is_empty() {
                    proposed_changes.push(ProposedChange {
                        id: format!("proposal:change:{}:requirement-{}", seed.id, index + 1),
                        title: atomic_title(text),
                        owner_epic: owner_epic.clone(),
                        priority: if horizon == "deferred" {
                            "p3".to_string()
                        } else {
                            seed.priority.clone()
                        },
                        priority_source: "inherited_from_epic".to_string(),
                        lane: if horizon == "deferred" {
                            "deferred"
                        } else {
                            "proposed"
                        }
                        .to_string(),
                        dependencies: change_dependencies,
                        covers: vec![id.clone()],
                        source_change: None,
                        reason: "missing_requirement_coverage".to_string(),
                    });
                }
                PlanRequirement {
                    id,
                    text: text.clone(),
                    horizon: horizon.clone(),
                    owner_epic,
                    status: if covered_by.is_empty() {
                        "gap"
                    } else if covered_by
                        .iter()
                        .any(|reference| reference.starts_with("proposal:"))
                    {
                        "planned"
                    } else {
                        "covered"
                    }
                    .to_string(),
                    covered_by,
                    verification: seed
                        .requirement_verification
                        .get(&index)
                        .cloned()
                        .unwrap_or_default(),
                    dependencies: dependency_ids,
                }
            })
            .collect::<Vec<_>>();
        epics.push(PlannedEpic {
            id: seed.id.clone(),
            title: seed.title.clone(),
            horizon: seed.horizon.clone(),
            priority: seed.priority.clone(),
            order: order + 1,
            requirements,
            change_ids: seed.change_ids.clone(),
            split_into: seed.split_into.clone(),
        });
    }

    changes.sort_by(|left, right| {
        priority_rank(&left.priority)
            .cmp(&priority_rank(&right.priority))
            .then_with(|| lane_rank(&left.lane).cmp(&lane_rank(&right.lane)))
            .then_with(|| reference_sort_key(&left.id).cmp(&reference_sort_key(&right.id)))
    });
    proposed_changes.sort_by(|left, right| {
        priority_rank(&left.priority)
            .cmp(&priority_rank(&right.priority))
            .then_with(|| {
                reference_sort_key(&left.owner_epic).cmp(&reference_sort_key(&right.owner_epic))
            })
            .then_with(|| left.id.cmp(&right.id))
    });
    let mut epic_order_nodes = Vec::new();
    for epic in &epics {
        if epic.split_into.is_empty() {
            epic_order_nodes.push((epic.priority.clone(), epic.horizon.clone(), epic.id.clone()));
        } else {
            epic_order_nodes.extend(
                proposed_epics
                    .iter()
                    .filter(|proposal| proposal.source_epic == epic.id)
                    .map(|proposal| {
                        (
                            proposal.priority.clone(),
                            proposal.horizon.clone(),
                            proposal.id.clone(),
                        )
                    }),
            );
        }
    }
    epic_order_nodes.sort_by(|left, right| {
        priority_rank(&left.0)
            .cmp(&priority_rank(&right.0))
            .then_with(|| horizon_rank(&left.1).cmp(&horizon_rank(&right.1)))
            .then_with(|| reference_sort_key(&left.2).cmp(&reference_sort_key(&right.2)))
    });
    let epic_order = epic_order_nodes.into_iter().map(|(_, _, id)| id).collect();

    ProjectPlan {
        schema: PROJECT_PLAN_SCHEMA.to_string(),
        action: "done".to_string(),
        root_id: project_plan_root_id(project, &graph.digest),
        stage,
        project: project.to_string(),
        project_label: project_label.to_string(),
        source_graph_digest: graph.digest,
        digest: String::new(),
        valid: true,
        stages,
        epic_order,
        epics,
        proposed_epics,
        changes,
        proposed_changes,
        diagnostics: graph.diagnostics,
        next: None,
    }
}

fn project_plan_root_id(project: &str, graph_digest: &str) -> String {
    let suffix = graph_digest
        .trim_start_matches("sha256:")
        .chars()
        .take(12)
        .collect::<String>();
    format!("project-plan:{project}:{suffix}")
}

/// Shared #2142 boundedness classifier used by validation and project planning.
pub fn looks_too_large_for_atomic_wi(issue: &Issue) -> bool {
    const HARD_PHRASES: &[&str] = &[
        "google map",
        "google maps",
        "full platform",
        "complete platform",
        "end-to-end product",
        "rewrite all",
        "rewrite everything",
        "all projects",
        "every project",
        "every crate",
        "across the fleet",
    ];
    const SCALE_WORDS: &[&str] = &["entire", "whole", "everything"];
    const SCOPE_NOUNS: &[&str] = &[
        "project",
        "projects",
        "codebase",
        "codebases",
        "repo",
        "repos",
        "repository",
        "repositories",
        "platform",
        "platforms",
        "system",
        "systems",
        "product",
        "products",
        "application",
        "applications",
        "app",
        "apps",
        "service",
        "services",
        "monorepo",
        "monorepos",
        "ecosystem",
        "ecosystems",
        "organization",
        "organizations",
        "org",
        "orgs",
        "roadmap",
        "roadmaps",
        "stack",
        "stacks",
        "suite",
        "suites",
        "fleet",
        "fleets",
        "company",
        "companies",
        "business",
        "businesses",
    ];
    let body_with_leading_newline = format!("\n{}", issue.body);
    let scope = body_with_leading_newline
        .split("\n## ")
        .find_map(|part| {
            let (heading, content) = part.split_once('\n')?;
            heading
                .trim()
                .eq_ignore_ascii_case("scope")
                .then_some(content)
        })
        .unwrap_or_default();
    let in_scope = scope
        .split("\n### ")
        .find_map(|part| {
            let (heading, content) = part.split_once('\n')?;
            heading
                .trim()
                .eq_ignore_ascii_case("in scope")
                .then_some(content)
        })
        .unwrap_or(scope);
    // A GHAN body states its bound in `## Goal` and has no `## Scope`. Without
    // this fallback the check would silently degrade to title-only and wave
    // through every roadmap-sized GHAN work item.
    let bounded = if in_scope.trim().is_empty() {
        body_with_leading_newline
            .split("\n## ")
            .find_map(|part| {
                let (heading, content) = part.split_once('\n')?;
                heading.trim().eq_ignore_ascii_case("goal").then_some(content)
            })
            .unwrap_or_default()
    } else {
        in_scope
    };
    let text = format!("{}\n{}", issue.title, bounded).to_ascii_lowercase();
    if HARD_PHRASES.iter().any(|phrase| text.contains(phrase)) {
        return true;
    }
    let words = text
        .split(char::is_whitespace)
        .map(|raw| raw.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-'))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    words.iter().enumerate().any(|(index, word)| {
        let from_scratch = *word == "from" && words.get(index + 1) == Some(&"scratch");
        if !SCALE_WORDS.contains(word) && !from_scratch {
            return false;
        }
        let start = index.saturating_sub(2);
        let end = (index + if from_scratch { 5 } else { 4 }).min(words.len());
        words[start..end]
            .iter()
            .any(|candidate| SCOPE_NOUNS.contains(candidate))
    })
}

fn extract_requirements(issue: &Issue) -> Vec<(String, String)> {
    let mut section = String::new();
    let mut raw_requirements = Vec::new();
    let mut current = None::<String>;
    for line in issue.body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            if section == "requirements" {
                if let Some(requirement) = current.take() {
                    raw_requirements.push(requirement);
                }
            }
            section = heading.to_ascii_lowercase();
            continue;
        }
        if trimmed.starts_with("### ") {
            if section == "requirements" {
                if let Some(requirement) = current.take() {
                    raw_requirements.push(requirement);
                }
            }
            continue;
        }
        if section != "requirements" {
            continue;
        }
        if let Some(value) = list_item_value(trimmed) {
            if let Some(requirement) = current.replace(value) {
                raw_requirements.push(requirement);
            }
            continue;
        }
        if !trimmed.is_empty()
            && line
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_whitespace())
        {
            if let Some(requirement) = current.as_mut() {
                requirement.push(' ');
                requirement.push_str(trimmed);
            }
        }
    }
    if let Some(requirement) = current {
        raw_requirements.push(requirement);
    }
    let mut requirements = raw_requirements
        .into_iter()
        .filter(|value| real_value(value))
        .map(|value| {
            let horizon = if deferred_text(&value) {
                "deferred"
            } else {
                "active"
            };
            (value, horizon.to_string())
        })
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    requirements.retain(|(text, horizon)| seen.insert((normalized_text(text), horizon.clone())));
    requirements
}

fn capability_context(issue: &Issue) -> String {
    for line in issue.body.lines() {
        let cleaned = line
            .trim()
            .trim_start_matches(|ch: char| matches!(ch, '-' | '*' | ' '))
            .replace("**", "")
            .replace('`', "");
        let trimmed = cleaned.trim();
        let lower = trimmed.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("capability:") {
            let offset = trimmed.len() - value.len();
            let context = trimmed[offset..].trim().trim_end_matches('.').to_string();
            if real_value(&context) {
                return context;
            }
        }
    }
    "unclassified".to_string()
}

fn explicit_issue_priority(issue: &Issue) -> Option<String> {
    issue
        .labels
        .iter()
        .filter_map(|label| label.strip_prefix("priority:"))
        .map(str::to_ascii_lowercase)
        .filter(|priority| matches!(priority.as_str(), "p0" | "p1" | "p2" | "p3"))
        .min_by_key(|priority| priority_rank(priority))
}

fn identifier_slug(value: &str) -> String {
    let slug = normalized_text(value).replace(' ', "-");
    if slug.is_empty() {
        "unclassified".to_string()
    } else {
        slug
    }
}

fn list_item_value(line: &str) -> Option<String> {
    let raw = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| {
            let (prefix, rest) = line.split_once(". ")?;
            prefix.chars().all(|ch| ch.is_ascii_digit()).then_some(rest)
        })?;
    let value = raw.trim().trim_matches('`').replace("**", "");
    let value = value
        .split_once(':')
        .filter(|(prefix, _)| {
            let prefix = prefix.trim();
            prefix.len() <= 8 && prefix.chars().any(|ch| ch.is_ascii_digit())
        })
        .map(|(_, rest)| rest.trim().to_string())
        .unwrap_or(value);
    Some(value)
}

fn explicit_child_work_item_ids(issue: &Issue) -> Vec<String> {
    let mut in_child_work_items = false;
    let mut ids = BTreeSet::new();
    for line in issue.body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            in_child_work_items = heading.trim().eq_ignore_ascii_case("child work items");
            continue;
        }
        if !in_child_work_items {
            continue;
        }
        for token in trimmed.split(|ch: char| ch != '#' && !ch.is_ascii_digit()) {
            let Some(id) = token.strip_prefix('#') else {
                continue;
            };
            if !id.is_empty() && id.chars().all(|ch| ch.is_ascii_digit()) {
                ids.insert(id.to_string());
            }
        }
    }
    ids.into_iter().collect()
}

fn explicit_child_requirement_coverage(issue: &Issue) -> BTreeMap<usize, Vec<String>> {
    let mut in_child_work_items = false;
    let mut coverage = BTreeMap::<usize, BTreeSet<String>>::new();
    for line in issue.body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            in_child_work_items = heading.trim().eq_ignore_ascii_case("child work items");
            continue;
        }
        if !in_child_work_items || !trimmed.starts_with('|') {
            continue;
        }
        let cells = trimmed
            .split('|')
            .map(str::trim)
            .filter(|cell| !cell.is_empty())
            .collect::<Vec<_>>();
        let Some(issue_cell) = cells.first() else {
            continue;
        };
        let issue_ids = numeric_references(issue_cell, '#');
        if issue_ids.is_empty() {
            continue;
        }
        for cell in cells.iter().skip(1) {
            for requirement in numeric_references(cell, 'r') {
                let Some(index) = requirement
                    .parse::<usize>()
                    .ok()
                    .and_then(|n| n.checked_sub(1))
                else {
                    continue;
                };
                coverage
                    .entry(index)
                    .or_default()
                    .extend(issue_ids.iter().cloned());
            }
        }
    }
    coverage
        .into_iter()
        .map(|(index, ids)| (index, ids.into_iter().collect()))
        .collect()
}

fn explicit_requirement_verification(
    issue: &Issue,
) -> BTreeMap<usize, Vec<RequirementVerification>> {
    let mut in_inventory = false;
    let mut verification = BTreeMap::<usize, Vec<RequirementVerification>>::new();
    for line in issue.body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            in_inventory = heading
                .trim()
                .eq_ignore_ascii_case("verification inventory");
            continue;
        }
        if !in_inventory || !trimmed.starts_with('|') {
            continue;
        }
        let cells = trimmed
            .split('|')
            .map(str::trim)
            .filter(|cell| !cell.is_empty())
            .collect::<Vec<_>>();
        if cells.len() < 3 {
            continue;
        }
        let gate = cells[1].trim_matches('`').trim().to_string();
        let oracle = cells[2].trim().to_string();
        if !real_value(&gate) || !real_value(&oracle) || gate.chars().all(|ch| ch == '-') {
            continue;
        }
        for requirement in numeric_references(cells[0], 'r') {
            let Some(index) = requirement
                .parse::<usize>()
                .ok()
                .and_then(|number| number.checked_sub(1))
            else {
                continue;
            };
            verification
                .entry(index)
                .or_default()
                .push(RequirementVerification {
                    gate: gate.clone(),
                    oracle: oracle.clone(),
                });
        }
    }
    verification
}

pub(crate) fn validate_requirement_verification_inventory(issue: &Issue) -> Vec<String> {
    let requirements = extract_requirements(issue);
    if requirements.is_empty() {
        return Vec::new();
    }
    let verification = explicit_requirement_verification(issue);
    let mut errors = requirements
        .iter()
        .enumerate()
        .filter_map(|(index, _)| {
            (!verification.contains_key(&index)).then(|| {
                format!(
                    "verification: ## Verification Inventory must map R{} to a runnable Gate and observable Oracle",
                    index + 1
                )
            })
        })
        .collect::<Vec<_>>();
    errors.extend(validate_requirement_dependencies(issue, requirements.len()));
    errors
}

fn explicit_requirement_dependencies(issue: &Issue) -> BTreeMap<usize, Vec<usize>> {
    let mut in_inventory = false;
    let mut dependencies = BTreeMap::<usize, BTreeSet<usize>>::new();
    for line in issue.body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            in_inventory = heading
                .trim()
                .eq_ignore_ascii_case("verification inventory");
            continue;
        }
        if !in_inventory || !trimmed.starts_with('|') {
            continue;
        }
        let cells = trimmed
            .split('|')
            .map(str::trim)
            .filter(|cell| !cell.is_empty())
            .collect::<Vec<_>>();
        if cells.len() < 4 {
            continue;
        }
        let requirements = numeric_references(cells[0], 'r');
        let dependency_refs = numeric_references(cells[3], 'r');
        for requirement in requirements {
            let Some(index) = requirement
                .parse::<usize>()
                .ok()
                .and_then(|number| number.checked_sub(1))
            else {
                continue;
            };
            for dependency in &dependency_refs {
                let Some(dependency_index) = dependency
                    .parse::<usize>()
                    .ok()
                    .and_then(|number| number.checked_sub(1))
                else {
                    continue;
                };
                dependencies
                    .entry(index)
                    .or_default()
                    .insert(dependency_index);
            }
        }
    }
    dependencies
        .into_iter()
        .map(|(index, values)| (index, values.into_iter().collect()))
        .collect()
}

fn validate_requirement_dependencies(issue: &Issue, requirement_count: usize) -> Vec<String> {
    let dependencies = explicit_requirement_dependencies(issue);
    let mut errors = Vec::new();
    for (requirement, dependency_ids) in &dependencies {
        for dependency in dependency_ids {
            if *dependency >= requirement_count {
                errors.push(format!(
                    "verification: R{} depends on unknown R{}",
                    requirement + 1,
                    dependency + 1
                ));
            } else if dependency == requirement {
                errors.push(format!(
                    "verification: R{} cannot depend on itself",
                    requirement + 1
                ));
            }
        }
    }

    fn visit(
        node: usize,
        dependencies: &BTreeMap<usize, Vec<usize>>,
        visiting: &mut BTreeSet<usize>,
        visited: &mut BTreeSet<usize>,
    ) -> bool {
        if visited.contains(&node) {
            return false;
        }
        if !visiting.insert(node) {
            return true;
        }
        let cycle = dependencies
            .get(&node)
            .into_iter()
            .flatten()
            .any(|dependency| visit(*dependency, dependencies, visiting, visited));
        visiting.remove(&node);
        visited.insert(node);
        cycle
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    if (0..requirement_count).any(|node| visit(node, &dependencies, &mut visiting, &mut visited)) {
        errors.push("verification: Requirement dependencies must be acyclic".to_string());
    }
    errors
}

fn numeric_references(text: &str, prefix: char) -> Vec<String> {
    let canonical_prefix = prefix.to_ascii_lowercase();
    let chars = text.chars().collect::<Vec<_>>();
    let mut values = BTreeSet::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index].to_ascii_lowercase() != canonical_prefix {
            index += 1;
            continue;
        }
        let start = index + 1;
        let mut end = start;
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }
        if end > start {
            values.insert(chars[start..end].iter().collect::<String>());
            index = end;
        } else {
            index += 1;
        }
    }
    values.into_iter().collect()
}

fn real_value(value: &str) -> bool {
    !matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "tbd" | "todo" | "(fill)" | "(replace-this)" | "none"
    )
}

fn is_structured_change(issue: &Issue) -> bool {
    [
        "## Capability Alignment",
        "## Scope",
        "## Acceptance Criteria",
        "## Reference Context",
    ]
    .iter()
    .all(|heading| issue.body.contains(heading))
        && issue.body.lines().any(|line| {
            let trimmed = line.trim();
            (trimmed.starts_with("- ") || trimmed.starts_with("* "))
                && real_value(trimmed.trim_start_matches(['-', '*', ' ']))
        })
}

fn is_deferred_issue(issue: &Issue) -> bool {
    issue.labels.iter().any(|label| {
        matches!(
            label.to_ascii_lowercase().as_str(),
            "deferred" | "status:deferred" | "priority:deferred"
        )
    }) || deferred_text(&format!("{}\n{}", issue.title, issue.body))
}

/// A local backend records newly published plan leaves as drafts until their
/// ordinary WI validation promotes them. They are already tracker-backed plan
/// records, however, so the next read-only plan must reconcile them rather
/// than propose a duplicate. Unrelated authoring drafts remain outside the
/// active project-plan inventory.
fn is_active_plan_issue(issue: &Issue) -> bool {
    issue.state == IssueState::Open
        || (issue.state == IssueState::Draft
            && issue.body.contains("<!-- aw:planning-transaction:"))
}

/// A completed mixed-horizon split is represented by two transaction-marked
/// sibling epics. The original epic remains as history, but must not be
/// replanned as a second split on every unchanged inventory read.
fn published_split_sources(issues: &[Issue]) -> BTreeSet<String> {
    let mut horizons_by_source = BTreeMap::<String, BTreeSet<String>>::new();
    for issue in issues
        .iter()
        .filter(|issue| issue.issue_type.as_str() == "epic")
    {
        let Some((source, horizon)) = published_split_source_and_horizon(issue) else {
            continue;
        };
        horizons_by_source
            .entry(source)
            .or_default()
            .insert(horizon);
    }
    horizons_by_source
        .into_iter()
        .filter_map(|(source, horizons)| {
            (horizons.contains("active") && horizons.contains("deferred")).then_some(source)
        })
        .collect()
}

fn published_split_source_and_horizon(issue: &Issue) -> Option<(String, String)> {
    if !issue.body.contains("<!-- aw:planning-transaction:") {
        return None;
    }
    let mut source = None;
    let mut horizon = None;
    for line in issue.body.lines() {
        let cleaned = line
            .trim()
            .trim_start_matches(|ch: char| matches!(ch, '-' | '*' | ' '))
            .replace("**", "")
            .replace('`', "");
        let trimmed = cleaned.trim();
        let lower = trimmed.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("source epic:") {
            let offset = trimmed.len() - value.len();
            let value = trimmed[offset..]
                .trim()
                .trim_start_matches('#')
                .trim_end_matches(|ch: char| matches!(ch, '.' | ',' | ';' | ')' | ']'))
                .trim();
            if !value.is_empty() {
                source = Some(value.to_string());
            }
        }
        if let Some(value) = lower.strip_prefix("planning horizon:") {
            let offset = trimmed.len() - value.len();
            let value = trimmed[offset..]
                .trim()
                .trim_end_matches(|ch: char| matches!(ch, '.' | ',' | ';' | ')' | ']'))
                .trim()
                .to_ascii_lowercase();
            if matches!(value.as_str(), "active" | "deferred") {
                horizon = Some(value);
            }
        }
    }
    Some((source?, horizon?))
}

fn deferred_text(text: &str) -> bool {
    text.lines().any(|line| {
        let cleaned = line
            .trim()
            .trim_start_matches(|ch: char| matches!(ch, '-' | '*' | ' '))
            .replace("**", "")
            .replace('`', "");
        let mut lower = cleaned.trim().to_ascii_lowercase();
        if let Some((prefix, rest)) = lower.split_once(':') {
            let prefix = prefix.trim();
            if prefix.len() <= 8
                && prefix.chars().any(|ch| ch.is_ascii_digit())
                && prefix.chars().all(|ch| ch.is_ascii_alphanumeric())
            {
                lower = rest.trim().to_string();
            }
        }
        [
            "deferred:",
            "[deferred]",
            "later phase:",
            "future phase:",
            "follow-up:",
            "follow up:",
            "phase 2:",
            "subsequent phase:",
            "eventually:",
            "not now:",
        ]
        .iter()
        .any(|marker| lower.starts_with(marker))
    })
}

fn inferred_duplicate_targets(
    owned: &[(String, &Issue, &super::GraphChange)],
) -> BTreeMap<String, String> {
    let mut by_title = BTreeMap::<String, Vec<String>>::new();
    for (id, issue, _) in owned {
        by_title
            .entry(normalized_text(&issue.title))
            .or_default()
            .push(id.clone());
    }
    let mut duplicates = BTreeMap::new();
    for ids in by_title.values_mut() {
        ids.sort_by(|left, right| reference_sort_key(left).cmp(&reference_sort_key(right)));
        if let Some(canonical) = ids.first().cloned() {
            for duplicate in ids.iter().skip(1) {
                duplicates.insert(duplicate.clone(), canonical.clone());
            }
        }
    }
    duplicates
}

fn text_covers(issue: &Issue, requirement: &str) -> bool {
    let requirement = significant_tokens(requirement);
    if requirement.is_empty() {
        return false;
    }
    let haystack = significant_tokens(&format!("{} {}", issue.title, issue.body));
    let overlap = requirement.intersection(&haystack).count();
    overlap >= 2 && overlap * 3 >= requirement.len() * 2
}

fn significant_tokens(text: &str) -> BTreeSet<String> {
    const STOP: &[&str] = &[
        "the", "and", "for", "with", "that", "this", "from", "into", "under", "all", "one",
        "should", "must", "will", "then", "when", "project", "change", "epic",
    ];
    text.split(|ch: char| !ch.is_alphanumeric())
        .map(str::to_ascii_lowercase)
        .filter(|word| word.len() >= 3 && !STOP.contains(&word.as_str()))
        .collect()
}

fn normalized_text(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if ch.is_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn requirement_id(epic_id: &str, index: usize) -> String {
    format!("{epic_id}:requirement-{}", index + 1)
}

fn requirement_index(requirement_id: &str) -> Option<usize> {
    requirement_id
        .rsplit_once(":requirement-")?
        .1
        .parse::<usize>()
        .ok()?
        .checked_sub(1)
}

fn requirement_owner(seed: &EpicSeed, horizon: &str) -> String {
    if seed.horizon == "mixed" {
        format!("proposal:epic:{}:{horizon}", seed.id)
    } else {
        seed.id.clone()
    }
}

fn atomic_title(requirement: &str) -> String {
    let compact = requirement.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= 96 {
        compact
    } else {
        compact.chars().take(93).collect::<String>() + "..."
    }
}

fn bounded_replacement_title(issue: &Issue, part: usize) -> String {
    match part {
        1 => format!("{} - bounded behavior", issue.title),
        _ => format!("{} - bounded integration", issue.title),
    }
}

fn priority_rank(priority: &str) -> u8 {
    match priority {
        "p0" => 0,
        "p1" => 1,
        "p2" => 2,
        _ => 3,
    }
}

fn horizon_rank(horizon: &str) -> u8 {
    match horizon {
        "active" => 0,
        "mixed" => 1,
        _ => 2,
    }
}

fn lane_rank(lane: &str) -> u8 {
    match lane {
        "ready_now" => 0,
        "blocked_by_dependency" => 1,
        "needs_triage" => 2,
        "needs_atomize" => 3,
        "duplicate" => 4,
        _ => 5,
    }
}

fn reference_sort_key(value: &str) -> (u8, u64, String) {
    match value.parse::<u64>() {
        Ok(number) => (0, number, String::new()),
        Err(_) => (1, u64::MAX, value.to_ascii_lowercase()),
    }
}

fn with_digest(mut plan: ProjectPlan) -> ProjectPlan {
    plan.digest.clear();
    let bytes = serde_json::to_vec(&plan).expect("project plan serializes");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    plan.digest = format!("{:x}", hasher.finalize());
    plan
}

// HANDWRITE-END
