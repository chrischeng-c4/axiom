//! Additional GitLab CI lint rules (GL002, GL005, GL006, GL009-GL012)

use super::gitlab_ci::CiJob;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, Position, Range};
use std::collections::HashSet;

/// GL002: Unknown job keywords
pub(super) fn check_unknown_keywords(jobs: &[CiJob], lines: &[&str]) -> Vec<Diagnostic> {
    const KNOWN_KEYWORDS: &[&str] = &[
        "script",
        "stage",
        "image",
        "services",
        "variables",
        "before_script",
        "after_script",
        "cache",
        "artifacts",
        "environment",
        "rules",
        "only",
        "except",
        "tags",
        "when",
        "allow_failure",
        "retry",
        "timeout",
        "needs",
        "extends",
        "trigger",
        "resource_group",
        "interruptible",
        "parallel",
        "release",
        "secrets",
        "pages",
        "inherit",
        "coverage",
        "dast_configuration",
        "dependencies",
        "id_tokens",
    ];
    let known: HashSet<&str> = KNOWN_KEYWORDS.iter().copied().collect();
    let mut diagnostics = Vec::new();

    for job in jobs {
        for kw in &job.keywords {
            if !known.contains(kw.as_str()) {
                let col = lines.get(job.start_line).map(|l| l.len()).unwrap_or(0);
                diagnostics.push(Diagnostic::warning(
                    Range::new(
                        Position::new(job.start_line as u32, 0),
                        Position::new(job.start_line as u32, col as u32),
                    ),
                    "GL002",
                    DiagnosticCategory::Logic,
                    format!("Job '{}' uses unknown keyword '{}'", job.name, kw),
                ));
            }
        }
    }

    diagnostics
}

/// GL005: `needs` referencing non-existent job
pub(super) fn check_needs_references(jobs: &[CiJob], lines: &[&str]) -> Vec<Diagnostic> {
    let job_names: HashSet<&str> = jobs.iter().map(|j| j.name.as_str()).collect();
    let mut diagnostics = Vec::new();

    for job in jobs {
        for need in &job.needs {
            if !job_names.contains(need.as_str()) {
                let col = lines.get(job.start_line).map(|l| l.len()).unwrap_or(0);
                diagnostics.push(Diagnostic::warning(
                    Range::new(
                        Position::new(job.start_line as u32, 0),
                        Position::new(job.start_line as u32, col as u32),
                    ),
                    "GL005",
                    DiagnosticCategory::Logic,
                    format!("Job '{}' needs '{}' which does not exist", job.name, need),
                ));
            }
        }
    }

    diagnostics
}

/// GL006: Circular `needs` dependencies
pub(super) fn check_circular_needs(jobs: &[CiJob], lines: &[&str]) -> Vec<Diagnostic> {
    use std::collections::HashMap as Map;
    let mut diagnostics = Vec::new();
    let job_names: HashSet<&str> = jobs.iter().map(|j| j.name.as_str()).collect();

    // Build adjacency list
    let mut graph: Map<&str, Vec<&str>> = Map::new();
    for job in jobs {
        let deps: Vec<&str> = job
            .needs
            .iter()
            .filter(|n| job_names.contains(n.as_str()))
            .map(|n| n.as_str())
            .collect();
        graph.insert(job.name.as_str(), deps);
    }

    // DFS cycle detection with path tracking
    let mut visited: HashSet<&str> = HashSet::new();
    let mut rec_stack: HashSet<&str> = HashSet::new();
    let mut path: Vec<&str> = Vec::new();
    let mut reported: HashSet<String> = HashSet::new();

    fn dfs<'a>(
        node: &'a str,
        graph: &Map<&'a str, Vec<&'a str>>,
        visited: &mut HashSet<&'a str>,
        rec_stack: &mut HashSet<&'a str>,
        path: &mut Vec<&'a str>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(neighbors) = graph.get(node) {
            for &next in neighbors {
                if !visited.contains(next) {
                    dfs(next, graph, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(next) {
                    let start = path.iter().position(|&n| n == next).unwrap_or(0);
                    let mut cycle: Vec<String> =
                        path[start..].iter().map(|s| s.to_string()).collect();
                    cycle.push(next.to_string());
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    let mut cycles: Vec<Vec<String>> = Vec::new();
    for job in jobs {
        if !visited.contains(job.name.as_str()) {
            dfs(
                job.name.as_str(),
                &graph,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycles,
            );
        }
    }

    for cycle in &cycles {
        let cycle_str = cycle.join(" -> ");
        if reported.contains(&cycle_str) {
            continue;
        }
        reported.insert(cycle_str.clone());
        let first = &cycle[0];
        let line = jobs
            .iter()
            .find(|j| j.name == *first)
            .map(|j| j.start_line)
            .unwrap_or(0);
        let col = lines.get(line).map(|l| l.len()).unwrap_or(0);
        diagnostics.push(Diagnostic::warning(
            Range::new(
                Position::new(line as u32, 0),
                Position::new(line as u32, col as u32),
            ),
            "GL006",
            DiagnosticCategory::Logic,
            format!("Circular 'needs' dependency: {}", cycle_str),
        ));
    }

    diagnostics
}

/// GL009: Missing timeout on jobs with script
pub(super) fn check_missing_timeout(jobs: &[CiJob], lines: &[&str]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for job in jobs {
        if job.has_script && !job.has_timeout {
            let col = lines.get(job.start_line).map(|l| l.len()).unwrap_or(0);
            diagnostics.push(Diagnostic::warning(
                Range::new(
                    Position::new(job.start_line as u32, 0),
                    Position::new(job.start_line as u32, col as u32),
                ),
                "GL009",
                DiagnosticCategory::Style,
                format!(
                    "Job '{}' has no 'timeout' — consider adding one to prevent stuck pipelines",
                    job.name,
                ),
            ));
        }
    }

    diagnostics
}

/// GL010: allow_failure without when: manual
pub(super) fn check_allow_failure_without_manual(
    jobs: &[CiJob],
    lines: &[&str],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for job in jobs {
        if job.has_allow_failure && !job.has_when_manual {
            let col = lines.get(job.start_line).map(|l| l.len()).unwrap_or(0);
            diagnostics.push(Diagnostic::warning(
                Range::new(
                    Position::new(job.start_line as u32, 0),
                    Position::new(job.start_line as u32, col as u32),
                ),
                "GL010",
                DiagnosticCategory::Logic,
                format!(
                    "Job '{}' has 'allow_failure' without 'when: manual' — failures will be silently ignored",
                    job.name,
                ),
            ));
        }
    }

    diagnostics
}

/// GL011: Unused extends templates (defined but never referenced)
pub(super) fn check_unused_templates(
    jobs: &[CiJob],
    templates: &[String],
    lines: &[&str],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let used_templates: HashSet<&str> = jobs.iter().filter_map(|j| j.extends.as_deref()).collect();

    for tpl in templates {
        if !used_templates.contains(tpl.as_str()) {
            let line_num = lines
                .iter()
                .enumerate()
                .find(|(_, l)| {
                    let trimmed = l.trim();
                    trimmed
                        .strip_suffix(':')
                        .map(|k| k.trim() == tpl)
                        .unwrap_or(false)
                })
                .map(|(i, _)| i)
                .unwrap_or(0);
            let col = lines.get(line_num).map(|l| l.len()).unwrap_or(0);
            diagnostics.push(Diagnostic::warning(
                Range::new(
                    Position::new(line_num as u32, 0),
                    Position::new(line_num as u32, col as u32),
                ),
                "GL011",
                DiagnosticCategory::Logic,
                format!("Template '{}' is defined but never extended", tpl),
            ));
        }
    }

    diagnostics
}

/// GL012: Invalid include references (check local: paths)
pub(super) fn check_invalid_includes(lines: &[&str]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut in_include = false;

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        if trimmed == "include:" && indent == 0 {
            in_include = true;
            continue;
        }

        if in_include {
            if indent == 0 && !trimmed.is_empty() && !trimmed.starts_with('-') {
                in_include = false;
                continue;
            }
            if let Some(rest) = trimmed
                .strip_prefix("- local:")
                .or_else(|| trimmed.strip_prefix("local:"))
            {
                let path = rest.trim().trim_matches('"').trim_matches('\'');
                if !path.starts_with('/') {
                    diagnostics.push(Diagnostic::warning(
                        Range::new(
                            Position::new(line_num as u32, 0),
                            Position::new(line_num as u32, line.len() as u32),
                        ),
                        "GL012",
                        DiagnosticCategory::Logic,
                        format!("Include local path '{}' should start with '/'", path),
                    ));
                }
            }
        }
    }

    diagnostics
}
