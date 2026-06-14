// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rules-section_format-rs.md#source
// CODEGEN-BEGIN
//! R3h — `SectionFormatRule`: enforce strict format conformance on every spec
//! section annotation.
//!
//! Three invariants per the TD spec
//! `projects/agentic-workflow/tech-design/core/validate/section-format-rule.md`:
//!
//! 1. Annotation-to-body binding — every `<!-- type: X lang: Y -->` annotation
//!    must be immediately followed (within a configurable lookahead window,
//!    default 5 lines) by either a matching-lang fenced block or the canonical
//!    placeholder annotation marker `<!-- score-td-placeholder -->`.
//! 2. Prose vs structural class — prose section types may have a plain markdown
//!    body extending to the next heading; structural section types MUST carry
//!    a fenced block or placeholder.
//! 3. Mermaid Plus frontmatter gate — any section annotated `lang: mermaid`
//!    must have a fence body whose first non-blank line is `---` (the Mermaid
//!    Plus frontmatter delimiter).

use crate::models::section::parse_all_section_annotations;
use crate::spec_alignment::parser as spec_parser;
use crate::validate::{Finding, Rule, RuleId, RuleReport};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// Default lookahead window (in lines) for finding a fenced block or
/// placeholder marker following an annotation.
pub const DEFAULT_LOOKAHEAD: usize = 5;

/// Canonical placeholder marker. Any line within the lookahead window
/// that contains this exact substring causes the section to be accepted
/// regardless of section type or lang.
pub const PLACEHOLDER_MARKER: &str = "<!-- score-td-placeholder -->";

/// `SectionFormatRule` validation rule (unit struct).
///
/// @spec projects/agentic-workflow/tech-design/core/validate/section-format-rule.md#logic
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SectionFormatRule {}

/// @spec projects/agentic-workflow/tech-design/core/validate/section-format-rule.md#logic
impl Rule for SectionFormatRule {
    fn id(&self) -> RuleId {
        RuleId::SectionFormat
    }

    fn check(&self, spec_path: &Path, content: &str, report: &mut RuleReport) {
        let findings = check_section_format(spec_path, content, DEFAULT_LOOKAHEAD);
        for f in findings {
            report.push(f);
        }
    }
}

/// Run the section-format check and return all findings.
///
/// Implements the Logic flowchart from the TD spec:
/// 1. Parse all section annotations.
/// 2. For each annotation, classify as prose or structural.
/// 3. For structural types, require a matching-lang fenced block or
///    placeholder within the lookahead window.
/// 4. For mermaid-lang sections, require the fence body's first non-blank
///    line to be `---`.
///
/// @spec projects/agentic-workflow/tech-design/core/validate/section-format-rule.md#logic
pub fn check_section_format(spec_path: &Path, content: &str, lookahead: usize) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Parse annotations: returns (heading_line_idx, SectionMeta).
    let annotations = parse_all_section_annotations(content);

    for (heading_line_idx, meta) in annotations {
        // Find the actual annotation line (the `<!-- type: ... -->` line).
        // It is the first line at-or-after the heading line containing the
        // annotation comment.
        let annotation_line_idx = match find_annotation_line(&lines, heading_line_idx) {
            Some(i) => i,
            None => {
                // Defensive: parse_all_section_annotations matched but the
                // line scan can't locate it. Skip this annotation rather
                // than emit a misleading finding.
                continue;
            }
        };

        let lang = meta.effective_lang();
        let is_prose = is_prose_section(meta.section_type);

        // Look for a placeholder marker in the lookahead window — placeholder
        // accepts any section type / lang outright (T7).
        let placeholder_present = has_placeholder(&lines, annotation_line_idx + 1, lookahead);

        if placeholder_present {
            continue;
        }

        // For non-prose (structural) types, require a matching-lang fenced
        // block within the lookahead window. Prose types fall through to
        // the mermaid-frontmatter check below.
        let fence_info = find_fence_in_window(&lines, annotation_line_idx + 1, lookahead);

        if !is_prose {
            match &fence_info {
                Some((_open_idx, fence_lang)) if fence_lang_matches(fence_lang, lang) => {
                    // ok — structural with matching fence
                }
                _ => {
                    findings.push(
                        Finding::error(
                            RuleId::SectionFormat,
                            spec_path,
                            format!(
                                "section type '{}' (lang: {}) requires a matching-lang fenced \
                                 block or `<!-- score-td-placeholder -->` within {} lines of \
                                 the annotation",
                                meta.section_type.as_str(),
                                lang,
                                lookahead,
                            ),
                        )
                        .with_line(annotation_line_idx + 1),
                    );
                    // Don't run the mermaid-plus check on a missing fence.
                    continue;
                }
            }
        }

        // Mermaid Plus frontmatter gate: when lang == mermaid, the fence
        // body's first non-blank line must be `---`.
        if lang == "mermaid" {
            if let Some((open_idx, fence_lang)) = fence_info {
                if fence_lang_matches(&fence_lang, "mermaid") {
                    if !mermaid_starts_with_frontmatter(&lines, open_idx + 1) {
                        findings.push(
                            Finding::error(
                                RuleId::SectionFormat,
                                spec_path,
                                format!(
                                    "section type '{}' uses lang: mermaid but fence body does \
                                     not start with `---` (Mermaid Plus frontmatter delimiter)",
                                    meta.section_type.as_str(),
                                ),
                            )
                            .with_line(annotation_line_idx + 1),
                        );
                    }
                }
            }
            // If we got here for a prose section with lang: mermaid and no
            // fence at all, the prose branch already accepted the body —
            // but a `lang: mermaid` annotation on a prose section without
            // any fence is itself nonsensical. We accept it for now since
            // R3 explicitly allows prose types to have no fence; a separate
            // rule could tighten this later.
        }
    }

    findings.extend(check_ui_complexity_budgets(spec_path, content));
    findings
}

/// Classify a `SectionType` as prose (free-form markdown body allowed) vs
/// structural (must carry a fenced block or placeholder).
///
/// Prose types per the section-format-rule TD spec: Overview, Doc,
/// Requirements, TestPlan, Scenarios. All remaining variants are structural.
///
/// Sourced from `crate::generate::generators::primitive_registry::is_prose_section`
/// — the registry owns the canonical taxonomy; this re-export keeps the
/// section_format rule's call sites unchanged.
///
/// @spec projects/agentic-workflow/tech-design/core/validate/section-format-rule.md#requirements
pub use crate::generate::generators::primitive_registry::is_prose_section;

/// Find the line index of the annotation comment, scanning at-or-after
/// `start_idx`. Supports legacy `<!-- type: ... -->` and attr-style
/// `<!-- score-section ... -->` forms.
fn find_annotation_line(lines: &[&str], start_idx: usize) -> Option<usize> {
    for (offset, line) in lines.iter().enumerate().skip(start_idx) {
        if crate::models::section::parse_section_annotation_parts(line).is_some() {
            return Some(offset);
        }
    }
    None
}

/// Return `true` if the canonical placeholder marker appears on any line
/// in `lines[start..start+window]` (clamped at the end of `lines`).
fn has_placeholder(lines: &[&str], start: usize, window: usize) -> bool {
    let end = (start + window).min(lines.len());
    if start >= end {
        return false;
    }
    lines[start..end]
        .iter()
        .any(|l| l.contains(PLACEHOLDER_MARKER))
}

/// Locate an opening fenced code block within `lines[start..start+window]`.
/// Returns `(line_idx_of_opening_fence, lang_label)` on the first hit.
/// The opening fence is the first line whose trim starts with at least three
/// backticks or tildes.
fn find_fence_in_window(lines: &[&str], start: usize, window: usize) -> Option<(usize, String)> {
    let end = (start + window).min(lines.len());
    if start >= end {
        return None;
    }
    for (offset, line) in lines.iter().enumerate().take(end).skip(start) {
        let trimmed = line.trim_start();
        let Some(marker) = trimmed.chars().next() else {
            continue;
        };
        if marker != '`' && marker != '~' {
            continue;
        }
        let marker_len = trimmed.chars().take_while(|c| *c == marker).count();
        if marker_len >= 3 {
            // Strip any trailing fence-attribute info; lang label is the
            // first whitespace-separated token after the fence marker.
            let rest = trimmed.get(marker_len..).unwrap_or("");
            let lang = rest.split_whitespace().next().unwrap_or("").to_string();
            return Some((offset, lang));
        }
    }
    None
}

/// Return `true` when the fence's declared lang label is acceptable for the
/// annotation's `lang:`. Empty/missing fence-lang matches anything (lenient
/// pass-through for legacy fences whose lang is implicit). Otherwise the
/// labels must match case-insensitively.
fn fence_lang_matches(fence_lang: &str, annotation_lang: &str) -> bool {
    if fence_lang.is_empty() {
        return true;
    }
    fence_lang.eq_ignore_ascii_case(annotation_lang)
}

/// Return `true` when the first non-blank line at-or-after `start_idx` is
/// exactly `---` (the Mermaid Plus frontmatter delimiter).
fn mermaid_starts_with_frontmatter(lines: &[&str], start_idx: usize) -> bool {
    for line in lines.iter().skip(start_idx) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        return trimmed == "---";
    }
    false
}

#[derive(Debug, Deserialize, Default)]
struct UiBudgetConfig {
    #[serde(default)]
    projects: Vec<UiBudgetProject>,
    #[serde(default)]
    ui_profiles: BTreeMap<String, UiBudgetProfile>,
}

#[derive(Debug, Deserialize, Default)]
struct UiBudgetProject {
    #[serde(default)]
    workspaces: Vec<UiBudgetWorkspace>,
}

#[derive(Debug, Deserialize, Default)]
struct UiBudgetWorkspace {
    name: Option<String>,
    ui_profile: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct UiBudgetProfile {
    #[serde(default)]
    task_budgets: BTreeMap<String, u64>,
}

#[derive(Debug, Clone)]
struct UiTaskMetric {
    id: String,
    class_name: String,
    score: u64,
}

const UI_BUDGET_SECTION_TYPES: &[&str] = &[
    "wireframe",
    "component",
    "design-token",
    "config",
    "manifest",
    "unit-test",
    "e2e-test",
];

fn check_ui_complexity_budgets(spec_path: &Path, content: &str) -> Vec<Finding> {
    let path_str = spec_path.to_string_lossy();
    let doc = spec_parser::parse(&path_str, content);
    let mut findings = Vec::new();

    for section in &doc.sections {
        let Some(annotation) = &section.annotation else {
            continue;
        };
        if !UI_BUDGET_SECTION_TYPES.contains(&annotation.section_type.as_str()) {
            continue;
        }
        let Some(workspace_name) = annotation.attributes.get("workspace") else {
            continue;
        };
        let tasks = extract_ui_tasks(section);
        if tasks.is_empty() {
            continue;
        }

        let Some(project_root) = find_score_project_root(spec_path) else {
            findings.push(
                Finding::error(
                    RuleId::SectionFormat,
                    spec_path,
                    format!(
                        "section '{}' targets workspace '{}' but no .aw/config.toml was found",
                        section.heading, workspace_name
                    ),
                )
                .with_line(section.line),
            );
            continue;
        };
        let config_path = project_root.join(".aw/config.toml");
        let config = match std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|raw| toml::from_str::<UiBudgetConfig>(&raw).ok())
        {
            Some(config) => config,
            None => continue,
        };

        let Some(workspace) = find_ui_workspace(&config, workspace_name) else {
            findings.push(
                Finding::error(
                    RuleId::SectionFormat,
                    spec_path,
                    format!(
                        "section '{}' references unknown workspace '{}' for UI complexity budget",
                        section.heading, workspace_name
                    ),
                )
                .with_line(section.line),
            );
            continue;
        };
        let Some(profile_name) = workspace.ui_profile.as_deref() else {
            findings.push(
                Finding::error(
                    RuleId::SectionFormat,
                    spec_path,
                    format!(
                        "workspace '{}' has no ui_profile for UI complexity budget validation",
                        workspace_name
                    ),
                )
                .with_line(section.line),
            );
            continue;
        };
        let Some(profile) = config.ui_profiles.get(profile_name) else {
            findings.push(
                Finding::error(
                    RuleId::SectionFormat,
                    spec_path,
                    format!(
                        "workspace '{}' references missing ui_profile '{}'",
                        workspace_name, profile_name
                    ),
                )
                .with_line(section.line),
            );
            continue;
        };

        for task in tasks {
            let Some(budget) = profile.task_budgets.get(&task.class_name) else {
                findings.push(
                    Finding::error(
                        RuleId::SectionFormat,
                        spec_path,
                        format!(
                            "UI task '{}' uses class '{}' but profile '{}' has no task budget",
                            task.id, task.class_name, profile_name
                        ),
                    )
                    .with_line(section.line),
                );
                continue;
            };
            if task.score > *budget {
                findings.push(
                    Finding::error(
                        RuleId::SectionFormat,
                        spec_path,
                        format!(
                            "UI task '{}' class '{}' complexity score {} exceeds budget {} in profile '{}'",
                            task.id, task.class_name, task.score, budget, profile_name
                        ),
                    )
                    .with_line(section.line),
                );
            }
        }
    }

    findings
}

fn find_score_project_root(spec_path: &Path) -> Option<std::path::PathBuf> {
    let abs = if spec_path.is_absolute() {
        spec_path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(spec_path)
    };
    let mut dir = abs.parent()?;
    loop {
        if dir.join(".aw/config.toml").is_file() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

fn find_ui_workspace<'a>(
    config: &'a UiBudgetConfig,
    workspace_name: &str,
) -> Option<&'a UiBudgetWorkspace> {
    config
        .projects
        .iter()
        .flat_map(|project| project.workspaces.iter())
        .find(|workspace| workspace.name.as_deref() == Some(workspace_name))
}

fn extract_ui_tasks(section: &crate::spec_alignment::models::SpecSection) -> Vec<UiTaskMetric> {
    let mut out = Vec::new();
    for block in &section.code_blocks {
        if block.lang != "yaml" {
            continue;
        }
        let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&block.content) else {
            continue;
        };
        collect_tasks_from_value(&value, &mut out);
    }
    out
}

fn collect_tasks_from_value(value: &serde_yaml::Value, out: &mut Vec<UiTaskMetric>) {
    collect_tasks_from_sequence(value.get("tasks"), out);
    if let Some(complexity) = value.get("complexity") {
        collect_tasks_from_sequence(complexity.get("tasks"), out);
    }
}

fn collect_tasks_from_sequence(value: Option<&serde_yaml::Value>, out: &mut Vec<UiTaskMetric>) {
    let Some(tasks) = value.and_then(|v| v.as_sequence()) else {
        return;
    };
    for task in tasks {
        let Some(id) = task.get("id").and_then(|v| v.as_str()).map(str::to_string) else {
            continue;
        };
        let Some(class_name) = task
            .get("class")
            .and_then(|v| v.as_str())
            .map(str::to_string)
        else {
            continue;
        };
        let score = task
            .get("metrics")
            .and_then(|v| v.as_mapping())
            .map(|metrics| {
                metrics
                    .values()
                    .filter_map(|v| v.as_i64())
                    .filter(|n| *n > 0)
                    .map(|n| n as u64)
                    .sum()
            })
            .unwrap_or(0);
        out.push(UiTaskMetric {
            id,
            class_name,
            score,
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::models::SectionType;

    use super::*;
    use std::path::PathBuf;

    fn run(content: &str) -> RuleReport {
        let mut r = RuleReport::new();
        SectionFormatRule::default().check(&PathBuf::from("test.md"), content, &mut r);
        r
    }

    fn run_with_config(content: &str, config: &str) -> RuleReport {
        let tmp = tempfile::tempdir().unwrap();
        let score_dir = tmp.path().join(".aw");
        std::fs::create_dir_all(&score_dir).unwrap();
        std::fs::write(score_dir.join("config.toml"), config).unwrap();
        let spec = tmp
            .path()
            .join("projects/agentic-workflow/tech-design/surface/specs/demo.md");
        let mut r = RuleReport::new();
        SectionFormatRule::default().check(&spec, content, &mut r);
        r
    }

    /// T1 prose_pass — overview section with plain markdown body is accepted
    /// without a fenced block.
    #[test]
    fn prose_pass_overview_with_plain_markdown_body() {
        let spec = "## Overview\n\
                    <!-- type: overview lang: markdown -->\n\
                    \n\
                    Some prose. No fence at all.\n";
        let report = run(spec);
        assert!(
            report.is_empty(),
            "expected no findings for prose with plain body, got: {:?}",
            report.findings,
        );
    }

    /// T2 prose_no_fence_ok — overview section followed immediately by another
    /// heading (empty body) is accepted.
    #[test]
    fn prose_no_fence_ok_empty_body() {
        let spec = "## Overview\n\
                    <!-- type: overview lang: markdown -->\n\
                    \n\
                    ## Next Heading\n\
                    <!-- type: doc lang: markdown -->\n\
                    \n\
                    body of doc\n";
        let report = run(spec);
        assert!(
            report.is_empty(),
            "expected no findings, got: {:?}",
            report.findings,
        );
    }

    /// T3 structural_pass — schema section with matching yaml fenced block is
    /// accepted.
    #[test]
    fn structural_pass_schema_with_yaml_fence() {
        let spec = "## Schema\n\
                    <!-- type: schema lang: yaml -->\n\
                    \n\
                    ```yaml\n\
                    foo: bar\n\
                    ```\n";
        let report = run(spec);
        assert!(
            report.is_empty(),
            "expected no findings for schema+yaml fence, got: {:?}",
            report.findings,
        );
    }

    #[test]
    fn structural_pass_rust_source_unit_with_long_fence() {
        let spec = "## Source\n\
                    <!-- type: rust-source-unit lang: rust -->\n\
                    \n\
                    ````rust\n\
                    /// ```ignore\n\
                    /// nested doc fence\n\
                    /// ```\n\
                    pub fn demo() {}\n\
                    ````\n";
        let report = run(spec);
        assert!(
            report.is_empty(),
            "expected no findings for long rust fence, got: {:?}",
            report.findings,
        );
    }

    /// T4 structural_fail — schema section with plain markdown body and no
    /// fence is hard-rejected.
    #[test]
    fn structural_fail_schema_with_plain_body() {
        let spec = "## Schema\n\
                    <!-- type: schema lang: yaml -->\n\
                    \n\
                    just some prose, no fence here either.\n";
        let report = run(spec);
        assert_eq!(report.findings.len(), 1);
        let f = &report.findings[0];
        assert_eq!(f.rule, RuleId::SectionFormat);
        assert!(f.message.contains("schema"));
        assert!(f.message.contains("fenced block") || f.message.contains("placeholder"));
    }

    /// T5 mermaid_plus_pass — logic section with mermaid fence whose body
    /// starts with `---` is accepted.
    #[test]
    fn mermaid_plus_pass_logic_with_frontmatter() {
        let spec = "## Logic\n\
                    <!-- type: logic lang: mermaid -->\n\
                    \n\
                    ```mermaid\n\
                    ---\n\
                    id: my-flow\n\
                    ---\n\
                    flowchart TD\n\
                    a --> b\n\
                    ```\n";
        let report = run(spec);
        assert!(
            report.is_empty(),
            "expected no findings, got: {:?}",
            report.findings,
        );
    }

    /// T6 mermaid_plus_fail — logic section with mermaid fence missing
    /// frontmatter is hard-rejected.
    #[test]
    fn mermaid_plus_fail_logic_without_frontmatter() {
        let spec = "## Logic\n\
                    <!-- type: logic lang: mermaid -->\n\
                    \n\
                    ```mermaid\n\
                    flowchart TD\n\
                    a --> b\n\
                    ```\n";
        let report = run(spec);
        assert_eq!(report.findings.len(), 1);
        let f = &report.findings[0];
        assert_eq!(f.rule, RuleId::SectionFormat);
        assert!(f.message.contains("Mermaid Plus") || f.message.contains("---"));
    }

    /// T7 placeholder_accepted — any structural section followed by the
    /// canonical placeholder marker is accepted regardless of type or lang.
    #[test]
    fn placeholder_accepted_for_structural_section() {
        let spec = "## Schema\n\
                    <!-- type: schema lang: yaml -->\n\
                    <!-- score-td-placeholder -->\n";
        let report = run(spec);
        assert!(
            report.is_empty(),
            "expected placeholder to accept the section, got: {:?}",
            report.findings,
        );
    }

    /// Placeholder also accepts a mermaid-lang section (no frontmatter check).
    #[test]
    fn placeholder_accepted_for_mermaid_section() {
        let spec = "## Logic\n\
                    <!-- type: logic lang: mermaid -->\n\
                    \n\
                    <!-- score-td-placeholder -->\n";
        let report = run(spec);
        assert!(report.is_empty(), "got: {:?}", report.findings);
    }

    #[test]
    fn attr_style_wireframe_within_budget_passes() {
        let config = r#"
[[projects]]
name = "cue"

[[projects.workspaces]]
name = "cue-artifact-studio"
ui_profile = "owner-frontoffice"

[ui_profiles.owner-frontoffice.task_budgets]
intake = 8
"#;
        let spec = r#"## Wireframe
<!-- score-section type="wireframe" lang="yaml" workspace="cue-artifact-studio" surface="studio" role="owner" -->

```yaml
tasks:
  - id: prompt_to_workitem
    class: intake
    metrics:
      visible_actions: 3
      required_decisions: 2
```
"#;
        assert!(run_with_config(spec, config).is_empty());
    }

    #[test]
    fn attr_style_wireframe_over_budget_fails() {
        let config = r#"
[[projects]]
name = "cue"

[[projects.workspaces]]
name = "cue-artifact-studio"
ui_profile = "owner-frontoffice"

[ui_profiles.owner-frontoffice.task_budgets]
intake = 4
"#;
        let spec = r#"## Wireframe
<!-- score-section type="wireframe" lang="yaml" workspace="cue-artifact-studio" -->

```yaml
tasks:
  - id: prompt_to_workitem
    class: intake
    metrics:
      visible_actions: 3
      required_decisions: 2
```
"#;
        let report = run_with_config(spec, config);
        assert!(report
            .findings
            .iter()
            .any(|f| f.message.contains("exceeds budget")));
    }

    #[test]
    fn attr_style_unknown_workspace_fails_when_tasks_present() {
        let config = r#"
[[projects]]
name = "cue"

[[projects.workspaces]]
name = "other"
ui_profile = "owner-frontoffice"

[ui_profiles.owner-frontoffice.task_budgets]
intake = 8
"#;
        let spec = r#"## Wireframe
<!-- score-section type="wireframe" lang="yaml" workspace="cue-artifact-studio" -->

```yaml
tasks:
  - id: prompt_to_workitem
    class: intake
    metrics:
      visible_actions: 1
```
"#;
        let report = run_with_config(spec, config);
        assert!(report
            .findings
            .iter()
            .any(|f| f.message.contains("unknown workspace")));
    }

    /// Multiple sections: one valid, one invalid → exactly one finding.
    #[test]
    fn mixed_sections_report_only_invalid() {
        let spec = "## Schema\n\
                    <!-- type: schema lang: yaml -->\n\
                    \n\
                    ```yaml\n\
                    a: 1\n\
                    ```\n\
                    \n\
                    ## Bad Schema\n\
                    <!-- type: schema lang: yaml -->\n\
                    \n\
                    no fence and no placeholder here\n";
        let report = run(spec);
        assert_eq!(report.findings.len(), 1);
    }

    /// Lookahead is bounded — a fence farther than the window away should
    /// be considered missing.
    #[test]
    fn fence_outside_lookahead_window_is_rejected() {
        // 6 blank lines between annotation and the fence (default window = 5).
        let spec = "## Schema\n\
                    <!-- type: schema lang: yaml -->\n\
                    \n\
                    \n\
                    \n\
                    \n\
                    \n\
                    \n\
                    ```yaml\n\
                    a: 1\n\
                    ```\n";
        let report = run(spec);
        assert_eq!(
            report.findings.len(),
            1,
            "expected fence outside lookahead to be rejected"
        );
    }

    #[test]
    fn is_prose_section_classification() {
        assert!(is_prose_section(SectionType::Overview));
        assert!(is_prose_section(SectionType::Doc));
        assert!(is_prose_section(SectionType::Requirements));
        assert!(is_prose_section(SectionType::Scenarios));
        assert!(!is_prose_section(SectionType::UnitTest));
        assert!(!is_prose_section(SectionType::E2eTest));
        assert!(!is_prose_section(SectionType::Schema));
        assert!(!is_prose_section(SectionType::Changes));
        assert!(!is_prose_section(SectionType::Logic));
        assert!(!is_prose_section(SectionType::StateMachine));
        assert!(!is_prose_section(SectionType::Cli));
    }
}

// CODEGEN-END
