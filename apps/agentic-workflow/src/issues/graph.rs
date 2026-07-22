// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/logic/issues/epic-change-graph.md#logic
// HANDWRITE-BEGIN gap="missing-generator:rust:issue-platform-graph" tracker="#2386" reason="The graph projection is a new issue-domain primitive; the spec defines the schema and invariants while the generator does not yet emit Rust domain logic."

//! Deterministic issue-platform projection for the canonical epic/change graph.

use super::{Issue, IssueState, IssueType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const GRAPH_SCHEMA: &str = "aw.wi.graph.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkItemGraph {
    pub schema: String,
    pub action: String,
    pub project: String,
    pub project_label: String,
    pub digest: String,
    pub valid: bool,
    pub epics: Vec<GraphEpic>,
    pub changes: Vec<GraphChange>,
    pub diagnostics: Vec<GraphDiagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<GraphNext>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEpic {
    pub id: String,
    pub title: String,
    pub state: String,
    pub priority: Option<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphChange {
    pub id: String,
    pub title: String,
    pub state: String,
    pub parent: Option<String>,
    pub priority: GraphPriority,
    pub dependencies: Vec<String>,
    pub duplicate_of: Option<String>,
    pub supersedes: Vec<String>,
    pub superseded_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPriority {
    pub value: Option<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherited_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphDiagnostic {
    pub code: String,
    pub issue: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related: Option<String>,
    pub message: String,
    pub remediation_target: String,
    pub next_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNext {
    pub command: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
struct ParentResolution {
    valid: Vec<String>,
}

/// Build a stable, read-only graph for one configured project label.
///
/// The caller deliberately supplies the complete backend inventory rather than
/// a project-filtered slice. That lets validation distinguish a missing parent
/// from an existing epic owned by another project.
pub fn build_work_item_graph(
    project: &str,
    project_label: &str,
    issues: &[Issue],
) -> WorkItemGraph {
    let mut indexed = issues.iter().collect::<Vec<_>>();
    indexed.sort_by_key(|issue| issue_sort_key(issue));

    let aliases = issue_alias_index(&indexed);
    let relevant = indexed
        .iter()
        .copied()
        .filter(|issue| issue.labels.iter().any(|label| label == project_label))
        .collect::<Vec<_>>();

    let mut diagnostics = Vec::new();
    let mut epic_priorities = BTreeMap::new();
    let mut parent_by_change = BTreeMap::new();

    for issue in &relevant {
        let id = issue_key(issue);
        let priorities = priority_labels(issue);
        if priorities.len() > 1 {
            diagnostics.push(diagnostic(
                "multiple_priority_labels",
                &id,
                None,
                format!(
                    "work item `{id}` declares multiple priority labels: {}",
                    priorities.join(", ")
                ),
                format!("priority label on {id}"),
            ));
        }
        if issue.issue_type == IssueType::Epic {
            let priority = (priorities.len() == 1).then(|| priorities[0].clone());
            if issue.state == IssueState::Open && priority.is_none() {
                diagnostics.push(diagnostic(
                    "missing_epic_priority",
                    &id,
                    None,
                    format!("open epic `{id}` must declare exactly one priority:p0..p3 label"),
                    format!("priority label on epic {id}"),
                ));
            }
            epic_priorities.insert(id, priority);
        }
    }

    for issue in relevant
        .iter()
        .copied()
        .filter(|issue| issue.issue_type != IssueType::Epic)
    {
        let id = issue_key(issue);
        let resolution =
            resolve_parents(issue, project_label, &indexed, &aliases, &mut diagnostics);
        let parent = (resolution.valid.len() == 1).then(|| resolution.valid[0].clone());
        if issue.state == IssueState::Open {
            if resolution.valid.is_empty() {
                diagnostics.push(diagnostic(
                    "unowned_change",
                    &id,
                    None,
                    format!("open change `{id}` does not resolve to exactly one owning epic"),
                    format!("epic:<epic-id> label on change {id}"),
                ));
            } else if resolution.valid.len() > 1 {
                diagnostics.push(diagnostic(
                    "multiple_epic_owners",
                    &id,
                    Some(resolution.valid.join(",")),
                    format!(
                        "open change `{id}` resolves to multiple owning epics: {}",
                        resolution.valid.join(", ")
                    ),
                    format!("single epic:<epic-id> label on change {id}"),
                ));
            }
        }
        parent_by_change.insert(id.clone(), parent);
    }

    let mut supersedes = BTreeMap::<String, BTreeSet<String>>::new();
    let mut superseded_by = BTreeMap::<String, BTreeSet<String>>::new();
    for issue in relevant
        .iter()
        .copied()
        .filter(|issue| issue.issue_type != IssueType::Epic)
    {
        let source = issue_key(issue);
        for raw_target in relation_labels(issue, "supersedes:") {
            if let Some(target) = resolve_relation_target(
                &source,
                &raw_target,
                "supersedes",
                project_label,
                &indexed,
                &aliases,
                &mut diagnostics,
            ) {
                supersedes
                    .entry(source.clone())
                    .or_default()
                    .insert(target.clone());
                superseded_by
                    .entry(target)
                    .or_default()
                    .insert(source.clone());
            }
        }
        for raw_replacement in relation_labels(issue, "superseded-by:") {
            if let Some(replacement) = resolve_relation_target(
                &source,
                &raw_replacement,
                "superseded-by",
                project_label,
                &indexed,
                &aliases,
                &mut diagnostics,
            ) {
                superseded_by
                    .entry(source.clone())
                    .or_default()
                    .insert(replacement.clone());
                supersedes
                    .entry(replacement)
                    .or_default()
                    .insert(source.clone());
            }
        }
    }

    for (replacement, originals) in &supersedes {
        for original in originals {
            let replacement_parent = parent_by_change.get(replacement).and_then(Clone::clone);
            let original_parent = parent_by_change.get(original).and_then(Clone::clone);
            if replacement_parent.is_none() || replacement_parent != original_parent {
                diagnostics.push(diagnostic(
                    "supersession_not_sibling",
                    replacement,
                    Some(original.clone()),
                    format!(
                        "replacement `{replacement}` and superseded change `{original}` must be siblings under the same epic"
                    ),
                    format!("shared epic:<epic-id> labels on {replacement} and {original}"),
                ));
            }
        }
    }

    let mut children = BTreeMap::<String, Vec<String>>::new();
    for (change, parent) in &parent_by_change {
        if let Some(parent) = parent {
            children
                .entry(parent.clone())
                .or_default()
                .push(change.clone());
        }
    }
    for values in children.values_mut() {
        values.sort_by(|left, right| reference_sort_key(left).cmp(&reference_sort_key(right)));
    }

    let mut epics = relevant
        .iter()
        .copied()
        .filter(|issue| issue.issue_type == IssueType::Epic)
        .map(|issue| {
            let id = issue_key(issue);
            GraphEpic {
                id: id.clone(),
                title: issue.title.clone(),
                state: issue.state.as_str().to_string(),
                priority: epic_priorities.get(&id).cloned().flatten(),
                children: children.remove(&id).unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();
    epics.sort_by(|left, right| reference_sort_key(&left.id).cmp(&reference_sort_key(&right.id)));

    let mut changes = relevant
        .iter()
        .copied()
        .filter(|issue| issue.issue_type != IssueType::Epic)
        .map(|issue| {
            let id = issue_key(issue);
            let parent = parent_by_change.get(&id).and_then(Clone::clone);
            let explicit_priority = priority_labels(issue);
            let priority = if explicit_priority.len() == 1 {
                GraphPriority {
                    value: Some(explicit_priority[0].clone()),
                    source: "explicit".to_string(),
                    inherited_from: None,
                }
            } else if let Some(parent) = parent.as_ref() {
                match epic_priorities.get(parent).cloned().flatten() {
                    Some(value) => GraphPriority {
                        value: Some(value),
                        source: "inherited".to_string(),
                        inherited_from: Some(parent.clone()),
                    },
                    None => unset_priority(),
                }
            } else {
                unset_priority()
            };

            let mut dependencies = relation_labels(issue, "depends-on:");
            dependencies.extend(body_dependency_references(issue));
            dependencies = resolve_relation_list(&dependencies, &indexed, &aliases);

            let duplicates =
                resolve_relation_list(&relation_labels(issue, "duplicate-of:"), &indexed, &aliases);
            if duplicates.len() > 1 {
                diagnostics.push(diagnostic(
                    "multiple_duplicate_targets",
                    &id,
                    Some(duplicates.join(",")),
                    format!("change `{id}` declares multiple duplicate-of targets"),
                    format!("single duplicate-of:<change-id> label on {id}"),
                ));
            }

            GraphChange {
                id: id.clone(),
                title: issue.title.clone(),
                state: issue.state.as_str().to_string(),
                parent,
                priority,
                dependencies,
                duplicate_of: (duplicates.len() == 1).then(|| duplicates[0].clone()),
                supersedes: supersedes
                    .get(&id)
                    .map(|values| values.iter().cloned().collect())
                    .unwrap_or_default(),
                superseded_by: superseded_by
                    .get(&id)
                    .map(|values| values.iter().cloned().collect())
                    .unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();
    changes.sort_by(|left, right| reference_sort_key(&left.id).cmp(&reference_sort_key(&right.id)));

    diagnostics.sort_by(|left, right| {
        (
            left.code.as_str(),
            reference_sort_key(&left.issue),
            left.related.as_deref().unwrap_or(""),
        )
            .cmp(&(
                right.code.as_str(),
                reference_sort_key(&right.issue),
                right.related.as_deref().unwrap_or(""),
            ))
    });
    diagnostics.dedup();

    let valid = diagnostics.is_empty();
    let next = diagnostics.first().map(|diagnostic| GraphNext {
        command: diagnostic.next_command.clone(),
        reason: diagnostic.message.clone(),
    });
    let mut graph = WorkItemGraph {
        schema: GRAPH_SCHEMA.to_string(),
        action: if valid { "done" } else { "blocked" }.to_string(),
        project: project.to_string(),
        project_label: project_label.to_string(),
        digest: String::new(),
        valid,
        epics,
        changes,
        diagnostics,
        next,
    };
    graph.digest = graph_digest(&graph);
    graph
}

/// True when any canonical or compatibility relation declares `parent` as
/// the issue's epic owner. Used by the existing goal runner while all graph
/// consumers converge on [`build_work_item_graph`].
pub fn issue_declares_parent(issue: &Issue, parent: &Issue) -> bool {
    let parent_aliases = issue_aliases(parent);
    declared_parent_references(issue)
        .into_iter()
        .chain(issue.related.iter().cloned())
        .chain(issue.implements.iter().cloned())
        .map(|reference| normalize_reference(&reference))
        .any(|reference| parent_aliases.contains(&reference))
}

/// Canonical parent ids declared explicitly by machine labels or legacy body
/// fields. Soft `related` / `implements` refs are intentionally excluded from
/// this helper so closed-child rollup does not mistake arbitrary see-also refs
/// for the command target.
pub fn explicit_parent_references(issue: &Issue) -> Vec<String> {
    let mut refs = declared_parent_references(issue)
        .into_iter()
        .map(|reference| normalize_reference(&reference))
        .filter(|reference| !reference.is_empty())
        .collect::<Vec<_>>();
    refs.sort_by(|left, right| reference_sort_key(left).cmp(&reference_sort_key(right)));
    refs.dedup();
    refs
}

pub fn epic_owner_label(epic_id: &str) -> String {
    format!("epic:{}", normalize_reference(epic_id))
}

pub fn dependency_label(change_id: &str) -> String {
    format!("depends-on:{}", normalize_reference(change_id))
}

pub fn duplicate_label(change_id: &str) -> String {
    format!("duplicate-of:{}", normalize_reference(change_id))
}

pub fn supersedes_label(change_id: &str) -> String {
    format!("supersedes:{}", normalize_reference(change_id))
}

pub fn superseded_by_label(change_id: &str) -> String {
    format!("superseded-by:{}", normalize_reference(change_id))
}

fn resolve_parents(
    issue: &Issue,
    project_label: &str,
    indexed: &[&Issue],
    aliases: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<GraphDiagnostic>,
) -> ParentResolution {
    let source = issue_key(issue);
    let explicit = declared_parent_references(issue);
    let mut raw = explicit.clone();
    for reference in issue.related.iter().chain(issue.implements.iter()) {
        if let Some(target) = resolve_issue(reference, indexed, aliases) {
            if target.issue_type == IssueType::Epic
                && target.labels.iter().any(|label| label == project_label)
            {
                raw.push(reference.clone());
            }
        }
    }

    let mut valid = Vec::new();
    for reference in raw {
        let normalized = normalize_reference(&reference);
        let Some(target) = resolve_issue(&normalized, indexed, aliases) else {
            if explicit
                .iter()
                .any(|value| normalize_reference(value) == normalized)
            {
                diagnostics.push(diagnostic(
                    "missing_epic_parent",
                    &source,
                    Some(normalized.clone()),
                    format!("change `{source}` references missing epic parent `{normalized}`"),
                    format!("existing epic target for epic:{normalized} on {source}"),
                ));
            }
            continue;
        };
        let target_id = issue_key(target);
        if target.issue_type != IssueType::Epic {
            diagnostics.push(diagnostic(
                "change_cannot_parent",
                &source,
                Some(target_id.clone()),
                format!("change `{source}` names non-epic `{target_id}` as its parent"),
                format!("epic parent replacing {target_id} on {source}"),
            ));
            continue;
        }
        if !target.labels.iter().any(|label| label == project_label) {
            diagnostics.push(diagnostic(
                "cross_project_epic_parent",
                &source,
                Some(target_id.clone()),
                format!(
                    "change `{source}` belongs to `{project_label}` but parent epic `{target_id}` does not"
                ),
                format!("same-project epic parent for {source}"),
            ));
            continue;
        }
        valid.push(target_id);
    }
    valid.sort_by(|left, right| reference_sort_key(left).cmp(&reference_sort_key(right)));
    valid.dedup();
    ParentResolution { valid }
}

fn resolve_relation_target(
    source: &str,
    raw_target: &str,
    relation: &str,
    project_label: &str,
    indexed: &[&Issue],
    aliases: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<GraphDiagnostic>,
) -> Option<String> {
    let normalized = normalize_reference(raw_target);
    let Some(target) = resolve_issue(&normalized, indexed, aliases) else {
        diagnostics.push(diagnostic(
            "missing_relation_target",
            source,
            Some(normalized.clone()),
            format!("change `{source}` has {relation} relation to missing `{normalized}`"),
            format!("existing change target for {relation}:{normalized} on {source}"),
        ));
        return None;
    };
    let target_id = issue_key(target);
    if target.issue_type == IssueType::Epic {
        diagnostics.push(diagnostic(
            "relation_target_not_change",
            source,
            Some(target_id.clone()),
            format!("change `{source}` has {relation} relation to epic `{target_id}`"),
            format!("change target replacing {relation}:{target_id} on {source}"),
        ));
        return None;
    }
    if !target.labels.iter().any(|label| label == project_label) {
        diagnostics.push(diagnostic(
            "cross_project_relation_target",
            source,
            Some(target_id.clone()),
            format!(
                "change `{source}` belongs to `{project_label}` but {relation} target `{target_id}` does not"
            ),
            format!("same-project change target for {relation} on {source}"),
        ));
        return None;
    }
    Some(target_id)
}

fn declared_parent_references(issue: &Issue) -> Vec<String> {
    let mut refs = Vec::new();
    for label in &issue.labels {
        for prefix in ["epic:", "parent-epic:", "parent:"] {
            if let Some(reference) = label.strip_prefix(prefix) {
                refs.push(reference.trim().to_string());
                break;
            }
        }
    }
    for line in issue.body.lines() {
        let cleaned = line
            .trim()
            .trim_start_matches(|ch: char| matches!(ch, '-' | '*' | ' '))
            .replace("**", "")
            .replace('`', "");
        let trimmed = cleaned.trim();
        let lower = trimmed.to_ascii_lowercase();
        for prefix in ["parent epic:", "parent wi:", "parent:"] {
            if lower.starts_with(prefix) {
                refs.push(trimmed[prefix.len()..].trim().trim_matches('`').to_string());
                break;
            }
        }
    }
    refs
}

fn body_dependency_references(issue: &Issue) -> Vec<String> {
    let mut refs = Vec::new();
    for line in issue.body.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("depends on")
            || lower.contains("dependency")
            || lower.contains("dependencies")
            || lower.contains("blocked by")
            || lower.contains("requires #")
        {
            refs.extend(hash_references(line));
        }
    }
    refs
}

fn hash_references(value: &str) -> Vec<String> {
    let bytes = value.as_bytes();
    let mut refs = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'#' {
            index += 1;
            continue;
        }
        index += 1;
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if start < index {
            refs.push(value[start..index].to_string());
        }
    }
    refs
}

fn relation_labels(issue: &Issue, prefix: &str) -> Vec<String> {
    issue
        .labels
        .iter()
        .filter_map(|label| label.strip_prefix(prefix))
        .map(normalize_reference)
        .filter(|reference| !reference.is_empty())
        .collect()
}

fn resolve_relation_list(
    references: &[String],
    indexed: &[&Issue],
    aliases: &BTreeMap<String, usize>,
) -> Vec<String> {
    let mut resolved = references
        .iter()
        .map(|reference| normalize_reference(reference))
        .map(|reference| {
            aliases
                .get(&reference)
                .and_then(|index| indexed.get(*index).copied())
                .map(issue_key)
                .unwrap_or(reference)
        })
        .filter(|reference| !reference.is_empty())
        .collect::<Vec<_>>();
    resolved.sort_by(|left, right| reference_sort_key(left).cmp(&reference_sort_key(right)));
    resolved.dedup();
    resolved
}

fn resolve_issue<'a>(
    reference: &str,
    indexed: &[&'a Issue],
    aliases: &BTreeMap<String, usize>,
) -> Option<&'a Issue> {
    aliases
        .get(&normalize_reference(reference))
        .and_then(|index| indexed.get(*index).copied())
}

fn issue_alias_index(indexed: &[&Issue]) -> BTreeMap<String, usize> {
    let mut aliases = BTreeMap::new();
    for (index, issue) in indexed.iter().enumerate() {
        for alias in issue_aliases(issue) {
            aliases.entry(alias).or_insert(index);
        }
    }
    aliases
}

fn issue_aliases(issue: &Issue) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    aliases.insert(issue_key(issue));
    if !issue.slug.is_empty() {
        aliases.insert(normalize_reference(&issue.slug));
    }
    if let Some(id) = &issue.id {
        aliases.insert(normalize_reference(id));
    }
    aliases
}

pub fn issue_key(issue: &Issue) -> String {
    issue
        .github_id
        .or(issue.gitlab_id)
        .map(|id| id.to_string())
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| normalize_reference(&issue.slug))
}

fn normalize_reference(value: &str) -> String {
    let value = value
        .trim()
        .trim_matches('`')
        .trim_start_matches('#')
        .trim_end_matches(|ch: char| matches!(ch, '.' | ',' | ';' | ')' | ']'))
        .trim();
    if let Some(fragment) = value.rsplit('/').next() {
        if !fragment.is_empty() && fragment.chars().all(|ch| ch.is_ascii_digit()) {
            return fragment.to_string();
        }
    }
    value.to_string()
}

fn priority_labels(issue: &Issue) -> Vec<String> {
    let mut priorities = issue
        .labels
        .iter()
        .filter_map(|label| label.strip_prefix("priority:"))
        .map(|value| value.to_ascii_lowercase())
        .filter(|value| matches!(value.as_str(), "p0" | "p1" | "p2" | "p3"))
        .collect::<Vec<_>>();
    priorities.sort();
    priorities.dedup();
    priorities
}

fn unset_priority() -> GraphPriority {
    GraphPriority {
        value: None,
        source: "unset".to_string(),
        inherited_from: None,
    }
}

fn issue_sort_key(issue: &Issue) -> (u8, u64, String) {
    let id = issue_key(issue);
    let (kind, number, text) = reference_sort_key(&id);
    (kind, number, text)
}

fn reference_sort_key(value: &str) -> (u8, u64, String) {
    match value.parse::<u64>() {
        Ok(number) => (0, number, String::new()),
        Err(_) => (1, u64::MAX, value.to_ascii_lowercase()),
    }
}

fn diagnostic(
    code: &str,
    issue: &str,
    related: Option<String>,
    message: String,
    remediation_target: String,
) -> GraphDiagnostic {
    GraphDiagnostic {
        code: code.to_string(),
        issue: issue.to_string(),
        related,
        message,
        remediation_target,
        next_command: format!("aw wi show {issue}"),
    }
}

fn graph_digest(graph: &WorkItemGraph) -> String {
    let mut payload = graph.clone();
    payload.digest.clear();
    let bytes = serde_json::to_vec(&payload).expect("work-item graph is serializable");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn issue(id: u64, issue_type: &str, state: &str, labels: &[&str], body: &str) -> Issue {
        let mut issue: Issue = serde_json::from_value(json!({
            "type": issue_type,
            "title": format!("fixture {id}"),
            "state": state,
        }))
        .unwrap();
        issue.github_id = Some(id);
        issue.slug = id.to_string();
        issue.labels = labels.iter().map(|label| label.to_string()).collect();
        issue.body = body.to_string();
        issue
    }

    #[test]
    fn canonical_and_legacy_parent_forms_normalize_to_the_same_graph() {
        let epic = issue(
            10,
            "epic",
            "open",
            &["type:epic", "app:demo", "priority:p1"],
            "",
        );
        let canonical = issue(
            11,
            "change",
            "open",
            &["type:change", "app:demo", "epic:10"],
            "",
        );
        let legacy = issue(
            11,
            "change",
            "open",
            &["type:change", "app:demo"],
            "- **Parent:** `#10`.",
        );

        let canonical_graph = build_work_item_graph("demo", "app:demo", &[epic.clone(), canonical]);
        let legacy_graph = build_work_item_graph("demo", "app:demo", &[legacy, epic]);

        assert_eq!(canonical_graph, legacy_graph);
        assert!(canonical_graph.valid);
        assert_eq!(canonical_graph.epics[0].children, vec!["11"]);
        assert_eq!(canonical_graph.changes[0].parent.as_deref(), Some("10"));
        assert_eq!(canonical_graph.changes[0].priority.source, "inherited");
        assert_eq!(
            canonical_graph.changes[0].priority.value.as_deref(),
            Some("p1")
        );
    }

    #[test]
    fn explicit_priority_overrides_inheritance_and_supersession_is_sibling_only() {
        let graph = build_work_item_graph(
            "demo",
            "app:demo",
            &[
                issue(
                    1,
                    "epic",
                    "open",
                    &["type:epic", "app:demo", "priority:p2"],
                    "",
                ),
                issue(
                    2,
                    "change",
                    "closed",
                    &["type:change", "app:demo", "epic:1", "superseded-by:3"],
                    "",
                ),
                issue(
                    3,
                    "change",
                    "open",
                    &[
                        "type:change",
                        "app:demo",
                        "epic:1",
                        "priority:p0",
                        "depends-on:2",
                        "supersedes:2",
                    ],
                    "",
                ),
            ],
        );

        assert!(graph.valid, "{:#?}", graph.diagnostics);
        assert_eq!(graph.epics[0].children, vec!["2", "3"]);
        assert_eq!(graph.changes[0].superseded_by, vec!["3"]);
        assert_eq!(graph.changes[1].supersedes, vec!["2"]);
        assert_eq!(graph.changes[1].dependencies, vec!["2"]);
        assert_eq!(graph.changes[1].priority.source, "explicit");
        assert_eq!(graph.changes[1].priority.value.as_deref(), Some("p0"));
    }

    #[test]
    fn invalid_ownership_reports_exact_remediation_targets() {
        let graph = build_work_item_graph(
            "demo",
            "app:demo",
            &[
                issue(
                    1,
                    "epic",
                    "open",
                    &["type:epic", "app:demo", "priority:p0"],
                    "",
                ),
                issue(
                    7,
                    "epic",
                    "open",
                    &["type:epic", "app:demo", "priority:p1"],
                    "",
                ),
                issue(
                    9,
                    "epic",
                    "open",
                    &["type:epic", "app:other", "priority:p0"],
                    "",
                ),
                issue(2, "change", "open", &["type:change", "app:demo"], ""),
                issue(
                    3,
                    "change",
                    "open",
                    &["type:change", "app:demo", "epic:404"],
                    "",
                ),
                issue(
                    4,
                    "change",
                    "open",
                    &["type:change", "app:demo", "epic:9"],
                    "",
                ),
                issue(
                    5,
                    "change",
                    "open",
                    &["type:change", "app:demo", "epic:2"],
                    "",
                ),
                issue(
                    6,
                    "change",
                    "open",
                    &["type:change", "app:demo", "epic:1", "parent:7"],
                    "",
                ),
            ],
        );

        assert!(!graph.valid);
        let codes = graph
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<BTreeSet<_>>();
        for expected in [
            "unowned_change",
            "missing_epic_parent",
            "cross_project_epic_parent",
            "change_cannot_parent",
            "multiple_epic_owners",
        ] {
            assert!(
                codes.contains(expected),
                "missing {expected}: {:#?}",
                graph.diagnostics
            );
        }
        assert!(graph.diagnostics.iter().all(|diagnostic| {
            diagnostic.next_command == format!("aw wi show {}", diagnostic.issue)
                && !diagnostic.remediation_target.is_empty()
        }));
    }

    #[test]
    fn graph_digest_is_stable_across_inventory_order_and_label_helpers_are_canonical() {
        let epic = issue(
            1,
            "epic",
            "open",
            &["type:epic", "app:demo", "priority:p3"],
            "",
        );
        let change = issue(
            2,
            "change",
            "open",
            &["type:change", "app:demo", "epic:1"],
            "",
        );
        let first = build_work_item_graph("demo", "app:demo", &[epic.clone(), change.clone()]);
        let second = build_work_item_graph("demo", "app:demo", &[change, epic]);
        assert_eq!(first.digest, second.digest);
        assert_eq!(first, second);
        assert_eq!(epic_owner_label("#1"), "epic:1");
        assert_eq!(dependency_label("#2"), "depends-on:2");
        assert_eq!(duplicate_label("#2"), "duplicate-of:2");
        assert_eq!(supersedes_label("#2"), "supersedes:2");
        assert_eq!(superseded_by_label("#3"), "superseded-by:3");
    }
}

// HANDWRITE-END
