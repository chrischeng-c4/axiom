// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/logic/issues/two-stage-project-plan.md#logic
// HANDWRITE-BEGIN gap="missing-generator:rust:two-stage-project-planner" tracker="#2387" reason="The deterministic planner is a new issue-domain aggregate; the spec defines its projection while the generator does not yet emit Rust planning logic."

//! Canonical two-stage epic/change planning projection.

use super::{build_work_item_graph, issue_key, GraphDiagnostic, GraphNext, Issue, IssueState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const PROJECT_PLAN_SCHEMA: &str = "aw.wi.project-plan.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectPlan {
    pub schema: String,
    pub action: String,
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
    split_into: Vec<String>,
}

/// Build the one authoritative read-only plan for a project's complete issue
/// inventory. Structural graph defects fail closed. Unowned open changes are
/// the one bootstrap exception: the planner groups them by declared capability
/// context and scheduling horizon, proposes explicit epic owners, and leaves
/// that complete mapping for independent review before publication.
pub fn build_project_plan(project: &str, project_label: &str, issues: &[Issue]) -> ProjectPlan {
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
        return with_digest(ProjectPlan {
            schema: PROJECT_PLAN_SCHEMA.to_string(),
            action: "blocked".to_string(),
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
        });
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
                for part in 1..=2 {
                    let proposal_id = format!("proposal:change:{change_id}:part-{part}");
                    let proposal_covers = covers
                        .iter()
                        .filter(|requirement| {
                            requirement_index(requirement)
                                .and_then(|index| seed.requirements.get(index))
                                .map(|(_, horizon)| horizon.as_str())
                                == Some(if deferred { "deferred" } else { "active" })
                        })
                        .enumerate()
                        .filter(|(index, _)| index % 2 == part - 1)
                        .map(|(_, requirement)| requirement.clone())
                        .collect::<Vec<_>>();
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

        let requirements = seed
            .requirements
            .iter()
            .enumerate()
            .map(|(index, (text, horizon))| {
                let id = requirement_id(&seed.id, index);
                let covered_by = coverage.remove(&id).unwrap_or_default();
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

    with_digest(ProjectPlan {
        schema: PROJECT_PLAN_SCHEMA.to_string(),
        action: "done".to_string(),
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
        diagnostics: Vec::new(),
        next: None,
    })
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
    let text = format!("{}\n{}", issue.title, issue.body).to_ascii_lowercase();
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
    let mut in_scope = true;
    let mut requirements = Vec::new();
    let transaction_published = issue.body.contains("<!-- aw:planning-transaction:");
    for line in issue.body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            section = heading.to_ascii_lowercase();
            in_scope = !section.contains("out of scope");
            continue;
        }
        if let Some(heading) = trimmed.strip_prefix("### ") {
            in_scope = !heading.to_ascii_lowercase().contains("out of scope");
            continue;
        }
        let eligible_section = if transaction_published {
            section == "requirements"
        } else {
            matches!(
                section.as_str(),
                "requirements" | "scope" | "acceptance criteria"
            )
        };
        if !eligible_section || !in_scope {
            continue;
        }
        let Some(value) = list_item_value(trimmed) else {
            continue;
        };
        if !real_value(&value) {
            continue;
        }
        let horizon = if deferred_text(&value) {
            "deferred"
        } else {
            "active"
        };
        requirements.push((value, horizon.to_string()));
    }
    if requirements.is_empty() {
        requirements.push((
            issue.title.clone(),
            if is_deferred_issue(issue) {
                "deferred".to_string()
            } else {
                "active".to_string()
            },
        ));
    }
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
    let lower = text.to_ascii_lowercase();
    [
        "deferred",
        "later phase",
        "future phase",
        "follow-up",
        "follow up",
        "phase 2",
        "subsequent",
        "eventually",
        "not now",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
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
