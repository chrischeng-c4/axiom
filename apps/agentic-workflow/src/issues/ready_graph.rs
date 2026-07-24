// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/logic/issues/reviewed-graph-goal-selection.md#logic
// HANDWRITE-BEGIN gap="missing-generator:rust:reviewed-graph-goal-selection" tracker="#2389" reason="The ready-leaf selector is issue-domain policy over the published epic/change graph; the generator does not emit cross-root scheduling aggregates."

//! Deterministic ready-leaf selection shared by epic and backlog goal roots.

use super::{GraphChange, GraphDiagnostic, WorkItemGraph};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadyChangeLeaf {
    pub id: String,
    pub epic: String,
    pub epic_priority: String,
    pub change_priority: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessBlocker {
    pub code: String,
    pub epic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<String>,
    pub message: String,
    pub next_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadyGraphSelection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<ReadyChangeLeaf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_epic: Option<String>,
    pub blockers: Vec<ReadinessBlocker>,
    pub open_change_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadyGraphError {
    pub code: String,
    pub issue: String,
    pub message: String,
    pub next_command: String,
}

impl fmt::Display for ReadyGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ReadyGraphError {}

/// Select one executable change from a valid published graph.
///
/// Epic priority chooses project direction. Only after an epic is chosen does
/// explicit-or-inherited change priority order its ready children. A blocked
/// epic never hides a ready leaf or terminal rollup in the next epic.
pub fn select_ready_change_leaf(
    graph: &WorkItemGraph,
    epic_scope: Option<&str>,
    excluded_changes: &BTreeSet<String>,
) -> Result<ReadyGraphSelection, ReadyGraphError> {
    if !graph.valid {
        return Err(graph
            .diagnostics
            .first()
            .map(error_from_diagnostic)
            .unwrap_or_else(|| ReadyGraphError {
                code: "invalid_graph".to_string(),
                issue: graph.project.clone(),
                message: format!(
                    "published work-item graph for `{}` is invalid",
                    graph.project
                ),
                next_command: format!("aw wi graph --project {} --json", graph.project),
            }));
    }

    let changes = graph
        .changes
        .iter()
        .map(|change| (change.id.as_str(), change))
        .collect::<BTreeMap<_, _>>();
    for change in graph
        .changes
        .iter()
        .filter(|change| change.state != "closed")
    {
        for dependency in &change.dependencies {
            if !changes.contains_key(dependency.as_str()) {
                return Err(ReadyGraphError {
                    code: "unresolved_dependency".to_string(),
                    issue: change.id.clone(),
                    message: format!(
                        "open change `{}` depends on unresolved change `{dependency}`",
                        change.id
                    ),
                    next_command: format!("aw wi show {}", change.id),
                });
            }
        }
    }

    if let Some(scope) = epic_scope {
        if !graph.epics.iter().any(|epic| epic.id == scope) {
            return Err(ReadyGraphError {
                code: "missing_epic_scope".to_string(),
                issue: scope.to_string(),
                message: format!("epic `{scope}` is absent from the reviewed project graph"),
                next_command: format!("aw wi show {scope}"),
            });
        }
    }

    let mut epics = graph
        .epics
        .iter()
        .filter(|epic| epic_scope.is_none_or(|scope| epic.id == scope))
        .collect::<Vec<_>>();
    epics.sort_by(|left, right| {
        (
            priority_rank(left.priority.as_deref()),
            reference_sort_key(&left.id),
        )
            .cmp(&(
                priority_rank(right.priority.as_deref()),
                reference_sort_key(&right.id),
            ))
    });

    let mut blockers = Vec::new();
    let mut open_change_count = 0usize;
    for epic in epics {
        let children = epic
            .children
            .iter()
            .filter_map(|id| changes.get(id.as_str()).copied())
            .collect::<Vec<_>>();
        let open = children
            .iter()
            .copied()
            .filter(|change| change.state != "closed")
            .collect::<Vec<_>>();

        if epic.state == "closed" && !open.is_empty() {
            return Err(ReadyGraphError {
                code: "closed_epic_has_open_change".to_string(),
                issue: epic.id.clone(),
                message: format!(
                    "closed epic `{}` still owns {} open change(s)",
                    epic.id,
                    open.len()
                ),
                next_command: format!("aw wi show {}", epic.id),
            });
        }
        if epic.state == "closed" {
            continue;
        }
        if children.is_empty() {
            blockers.push(ReadinessBlocker {
                code: "childless_epic".to_string(),
                epic: epic.id.clone(),
                change: None,
                message: format!(
                    "open epic `{}` has no reviewed change children in the published graph",
                    epic.id
                ),
                next_command: format!("aw wi plan --project {}", graph.project),
            });
            continue;
        }
        if open.is_empty() {
            return Ok(ReadyGraphSelection {
                selected: None,
                terminal_epic: Some(epic.id.clone()),
                blockers,
                open_change_count,
            });
        }
        open_change_count += open.len();

        let mut ready = Vec::new();
        for change in open {
            if excluded_changes.contains(&change.id) {
                blockers.push(blocker(
                    "parked_change",
                    epic.id.as_str(),
                    change,
                    format!("change `{}` is parked by the backlog root", change.id),
                    format!("aw goal wi {}", change.id),
                ));
                continue;
            }
            if change.duplicate_of.is_some() || !change.superseded_by.is_empty() {
                blockers.push(blocker(
                    "retained_history",
                    epic.id.as_str(),
                    change,
                    format!(
                        "change `{}` is retained duplicate/superseded history and is not executable",
                        change.id
                    ),
                    format!("aw wi show {}", change.id),
                ));
                continue;
            }
            let open_dependencies = change
                .dependencies
                .iter()
                .filter(|dependency| {
                    changes
                        .get(dependency.as_str())
                        .is_some_and(|dependency| dependency.state != "closed")
                })
                .cloned()
                .collect::<Vec<_>>();
            if !open_dependencies.is_empty() {
                blockers.push(blocker(
                    "dependency_blocked",
                    epic.id.as_str(),
                    change,
                    format!(
                        "change `{}` is blocked by open dependencies: {}",
                        change.id,
                        open_dependencies.join(", ")
                    ),
                    format!("aw goal wi {}", open_dependencies[0]),
                ));
                continue;
            }
            ready.push(change);
        }
        ready.sort_by(|left, right| {
            (
                priority_rank(left.priority.value.as_deref()),
                reference_sort_key(&left.id),
            )
                .cmp(&(
                    priority_rank(right.priority.value.as_deref()),
                    reference_sort_key(&right.id),
                ))
        });
        if let Some(change) = ready.first() {
            return Ok(ReadyGraphSelection {
                selected: Some(ReadyChangeLeaf {
                    id: change.id.clone(),
                    epic: epic.id.clone(),
                    epic_priority: epic.priority.clone().unwrap_or_else(|| "unset".to_string()),
                    change_priority: change
                        .priority
                        .value
                        .clone()
                        .unwrap_or_else(|| "unset".to_string()),
                }),
                terminal_epic: None,
                blockers,
                open_change_count,
            });
        }
    }

    Ok(ReadyGraphSelection {
        selected: None,
        terminal_epic: None,
        blockers,
        open_change_count,
    })
}

fn blocker(
    code: &str,
    epic: &str,
    change: &GraphChange,
    message: String,
    next_command: String,
) -> ReadinessBlocker {
    ReadinessBlocker {
        code: code.to_string(),
        epic: epic.to_string(),
        change: Some(change.id.clone()),
        message,
        next_command,
    }
}

fn error_from_diagnostic(diagnostic: &GraphDiagnostic) -> ReadyGraphError {
    ReadyGraphError {
        code: diagnostic.code.clone(),
        issue: diagnostic.issue.clone(),
        message: diagnostic.message.clone(),
        next_command: diagnostic.next_command.clone(),
    }
}

fn priority_rank(priority: Option<&str>) -> u8 {
    match priority {
        Some("p0") => 0,
        Some("p1") => 1,
        Some("p2") => 2,
        Some("p3") => 3,
        _ => 4,
    }
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
    use crate::issues::{build_work_item_graph, Issue};

    fn issue(id: u64, issue_type: &str, state: &str, labels: &[&str]) -> Issue {
        let mut issue: Issue = serde_json::from_value(serde_json::json!({
            "type": issue_type,
            "title": format!("fixture {id}"),
            "state": state,
        }))
        .unwrap();
        issue.github_id = Some(id);
        issue.slug = id.to_string();
        issue.labels = labels.iter().map(|label| label.to_string()).collect();
        issue
    }

    #[test]
    fn epic_priority_chooses_direction_then_change_priority_chooses_leaf() {
        let graph = build_work_item_graph(
            "demo",
            "app:demo",
            &[
                issue(
                    10,
                    "epic",
                    "open",
                    &["app:demo", "type:epic", "priority:p1"],
                ),
                issue(
                    20,
                    "epic",
                    "open",
                    &["app:demo", "type:epic", "priority:p0"],
                ),
                issue(
                    11,
                    "change",
                    "open",
                    &["app:demo", "type:change", "epic:10", "priority:p0"],
                ),
                issue(
                    21,
                    "change",
                    "open",
                    &["app:demo", "type:change", "epic:20", "priority:p2"],
                ),
                issue(
                    22,
                    "change",
                    "open",
                    &["app:demo", "type:change", "epic:20", "priority:p1"],
                ),
            ],
        );
        let selection = select_ready_change_leaf(&graph, None, &BTreeSet::new()).unwrap();
        assert_eq!(selection.selected.unwrap().id, "22");
    }

    #[test]
    fn blocked_high_priority_leaf_does_not_hide_next_ready_leaf() {
        let graph = build_work_item_graph(
            "demo",
            "app:demo",
            &[
                issue(
                    10,
                    "epic",
                    "open",
                    &["app:demo", "type:epic", "priority:p0"],
                ),
                issue(
                    11,
                    "change",
                    "open",
                    &[
                        "app:demo",
                        "type:change",
                        "epic:10",
                        "priority:p0",
                        "depends-on:12",
                    ],
                ),
                issue(
                    12,
                    "change",
                    "open",
                    &["app:demo", "type:change", "epic:10", "priority:p1"],
                ),
            ],
        );
        let selection = select_ready_change_leaf(&graph, None, &BTreeSet::new()).unwrap();
        assert_eq!(selection.selected.unwrap().id, "12");
        assert_eq!(selection.blockers[0].code, "dependency_blocked");
    }

    #[test]
    fn all_closed_children_select_terminal_epic_rollup() {
        let graph = build_work_item_graph(
            "demo",
            "app:demo",
            &[
                issue(
                    10,
                    "epic",
                    "open",
                    &["app:demo", "type:epic", "priority:p0"],
                ),
                issue(
                    11,
                    "change",
                    "closed",
                    &["app:demo", "type:change", "epic:10"],
                ),
            ],
        );
        let selection = select_ready_change_leaf(&graph, Some("10"), &BTreeSet::new()).unwrap();
        assert_eq!(selection.terminal_epic.as_deref(), Some("10"));
        assert!(selection.selected.is_none());
    }

    #[test]
    fn unresolved_dependency_fails_closed_with_issue_remediation() {
        let graph = build_work_item_graph(
            "demo",
            "app:demo",
            &[
                issue(
                    10,
                    "epic",
                    "open",
                    &["app:demo", "type:epic", "priority:p0"],
                ),
                issue(
                    11,
                    "change",
                    "open",
                    &["app:demo", "type:change", "epic:10", "depends-on:404"],
                ),
            ],
        );
        let error = select_ready_change_leaf(&graph, None, &BTreeSet::new()).unwrap_err();
        assert_eq!(error.code, "unresolved_dependency");
        assert_eq!(error.next_command, "aw wi show 11");
    }
}
