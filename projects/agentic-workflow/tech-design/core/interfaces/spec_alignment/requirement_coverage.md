---
id: projects-sdd-src-spec-alignment-requirement-coverage-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate."
---

# Standardized projects/agentic-workflow/src/spec_alignment/requirement_coverage.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_alignment/requirement_coverage.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `check` | projects/agentic-workflow/src/spec_alignment/requirement_coverage.rs | function | pub | 23 | check(doc: &SpecDocument) -> (Vec<Violation>, Vec<OrphanRequirementEntry>) |
| `check_with_content` | projects/agentic-workflow/src/spec_alignment/requirement_coverage.rs | function | pub | 34 | check_with_content(     doc: &SpecDocument,     content: &str, ) -> (Vec<Violation>, Vec<OrphanRequirementEntry>) |
| `extract_requirement_ids` | projects/agentic-workflow/src/spec_alignment/requirement_coverage.rs | function | pub | 215 | extract_requirement_ids(spec_path: &str) -> HashMap<String, Option<String>> |
| `extract_requirement_ids_from_content` | projects/agentic-workflow/src/spec_alignment/requirement_coverage.rs | function | pub | 226 | extract_requirement_ids_from_content(content: &str) -> HashMap<String, Option<String>> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/spec_alignment/requirement_coverage.rs -->
```rust
//! Requirement↔Scenario cross-reference.
//!
//! Extracts R{N} IDs from requirements tables and checks that each is referenced
//! by at least one scenario body or test-plan `Covers` column.
//! Emits `orphan_requirement` violations for unreferenced requirements.

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use super::models::{OrphanRequirementEntry, SpecDocument, Violation, ViolationKind};

/// Regex to match requirement IDs: `R` followed by one or more digits, word-boundary.
static REQUIREMENT_ID_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bR(\d+)\b").unwrap());

/// Check requirement↔scenario coverage within a single spec document.
///
/// Returns violations for any R{N} in the Requirements table not referenced by
/// any scenario body or test-plan Covers column.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/requirement_coverage.md#source
pub fn check(doc: &SpecDocument) -> (Vec<Violation>, Vec<OrphanRequirementEntry>) {
    // Read file once and pass content to all extraction functions
    let content = match std::fs::read_to_string(&doc.path) {
        Ok(c) => c,
        Err(_) => return (Vec::new(), Vec::new()),
    };
    check_with_content(doc, &content)
}

/// Check requirement↔scenario coverage using pre-read content (avoids redundant file I/O).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/requirement_coverage.md#source
pub fn check_with_content(
    doc: &SpecDocument,
    content: &str,
) -> (Vec<Violation>, Vec<OrphanRequirementEntry>) {
    let requirements = extract_requirements_from_content(content);
    if requirements.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let scenario_refs = extract_scenario_references_from_content(content);
    let test_plan_refs = extract_test_plan_covers_from_content(content);

    let all_refs: HashSet<String> = scenario_refs.union(&test_plan_refs).cloned().collect();

    let mut violations = Vec::new();
    let mut orphans = Vec::new();

    for (req_id, description) in &requirements {
        if !all_refs.contains(req_id) {
            violations.push(Violation {
                kind: ViolationKind::OrphanRequirement,
                message: format!(
                    "Requirement '{}' in '{}' is not referenced by any scenario or test-plan entry",
                    req_id, doc.path
                ),
                heading: Some("Requirements".to_string()),
                line: None,
                lines: None,
                name: Some(req_id.clone()),
                expected_lang: None,
                field: None,
                details: None,
            });

            orphans.push(OrphanRequirementEntry {
                requirement_id: req_id.clone(),
                spec_path: doc.path.clone(),
                description: description.clone(),
            });
        }
    }

    (violations, orphans)
}

/// Extract R{N} IDs and their descriptions from the Requirements section of content.
fn extract_requirements_from_content(content: &str) -> HashMap<String, Option<String>> {
    let mut reqs = HashMap::new();
    let mut in_requirements = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect entering Requirements section
        if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            in_requirements = trimmed
                .strip_prefix("## ")
                .map(|h| h.trim().to_lowercase() == "requirements")
                .unwrap_or(false);
            continue;
        }

        if !in_requirements {
            continue;
        }

        // Parse table rows: | R{N} | description | ... |
        if trimmed.starts_with('|') && trimmed.contains('|') {
            let cells: Vec<&str> = trimmed.split('|').collect();
            // cells[0] is empty (before first |), cells[1] is ID column
            if cells.len() >= 3 {
                let id_cell = cells[1].trim();
                if let Some(cap) = REQUIREMENT_ID_RE.captures(id_cell) {
                    let req_id = format!("R{}", &cap[1]);
                    let description = if cells.len() >= 4 {
                        Some(cells[2].trim().to_string())
                    } else {
                        None
                    };
                    reqs.insert(req_id, description);
                }
            }
        }
    }

    reqs
}

/// Extract requirement references from scenario sections of content.
fn extract_scenario_references_from_content(content: &str) -> HashSet<String> {
    let mut refs = HashSet::new();
    let mut in_scenarios = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect entering/leaving Scenarios section
        if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            in_scenarios = trimmed
                .strip_prefix("## ")
                .map(|h| h.trim().to_lowercase() == "scenarios")
                .unwrap_or(false);
            continue;
        }

        if !in_scenarios {
            continue;
        }

        // Collect all R{N} references in scenario text
        for cap in REQUIREMENT_ID_RE.captures_iter(trimmed) {
            refs.insert(format!("R{}", &cap[1]));
        }
    }

    refs
}

/// Extract requirement IDs from test-plan `Covers` column of content.
fn extract_test_plan_covers_from_content(content: &str) -> HashSet<String> {
    let mut refs = HashSet::new();
    let mut in_test_plan = false;
    let mut covers_col_idx: Option<usize> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect entering/leaving Test Plan section
        if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            in_test_plan = trimmed
                .strip_prefix("## ")
                .map(|h| h.trim().to_lowercase() == "test plan")
                .unwrap_or(false);
            covers_col_idx = None;
            continue;
        }

        if !in_test_plan {
            continue;
        }

        // Find header row to locate Covers column
        if trimmed.starts_with('|') && covers_col_idx.is_none() {
            let cells: Vec<&str> = trimmed.split('|').collect();
            for (i, cell) in cells.iter().enumerate() {
                if cell.trim().to_lowercase() == "covers" {
                    covers_col_idx = Some(i);
                    break;
                }
            }
            continue;
        }

        // Skip separator row
        if trimmed.starts_with('|') && trimmed.contains("---") {
            continue;
        }

        // Parse data rows — extract R{N} from Covers column
        if let Some(col_idx) = covers_col_idx {
            if trimmed.starts_with('|') {
                let cells: Vec<&str> = trimmed.split('|').collect();
                if cells.len() > col_idx {
                    let covers_cell = cells[col_idx].trim();
                    for cap in REQUIREMENT_ID_RE.captures_iter(covers_cell) {
                        refs.insert(format!("R{}", &cap[1]));
                    }
                }
            }
        }
    }

    refs
}

/// Extract all requirement IDs from a spec file (for use by coverage module).
///
/// Returns a map of requirement_id -> optional description.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/requirement_coverage.md#source
pub fn extract_requirement_ids(spec_path: &str) -> HashMap<String, Option<String>> {
    let content = match std::fs::read_to_string(spec_path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };

    extract_requirement_ids_from_content(&content)
}

/// Extract all requirement IDs from spec content (no file I/O).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/requirement_coverage.md#source
pub fn extract_requirement_ids_from_content(content: &str) -> HashMap<String, Option<String>> {
    extract_requirements_from_content(content)
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_alignment/requirement_coverage.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete requirement coverage alignment module.
```
