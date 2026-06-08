---
id: sdd-spec-plan-section-suggestions-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Spec Plan Section Suggestions

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/spec_plan.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SpecPlanEntry` | projects/agentic-workflow/src/tools/spec_plan.rs | struct | pub | 203 |  |
| `deduplicate_spec_plans` | projects/agentic-workflow/src/tools/spec_plan.rs | function | pub | 262 | deduplicate_spec_plans(change_dir: &Path) -> Result<()> |
| `prepare_specs_from_plan` | projects/agentic-workflow/src/tools/spec_plan.rs | function | pub | 319 | prepare_specs_from_plan(change_dir: &Path, project_root: &Path) -> Result<Vec<String>> |
| `suggest_sections` | projects/agentic-workflow/src/tools/spec_plan.rs | function | pub | 51 | suggest_sections(requirements: &str) -> Vec<String> |
## Source
<!-- type: source lang: rust -->

````rust
/// Map a spec-domain section name to its fill_order priority.
///
/// Based on the Section Fill Order in `.aw/tech-design/sdd/logic/change-spec.md`.
/// Lower number = fill first.
fn section_fill_order(section: &str) -> u8 {
    match section {
        "overview" => 0,
        "db-model" => 1,
        "schema" => 2,
        "state-machine" => 3,
        "logic" | "model" | "prompt" => 4,
        "dependency" => 5,
        "interaction" | "threat-model" | "auth-matrix" => 6,
        "rest-api" | "rpc-api" | "async-api" | "cli" | "grpc" | "graphql" => 7,
        "wireframe" | "component" | "design-token" => 8,
        "config" | "container" | "deploy" | "cloud-resource" | "pipeline" | "observability" => 9,
        "unit-test" | "e2e-test" | "test-fixture" | "perf-test" | "security-test" => 10,
        "changes" => 11,
        _ => 99,
    }
}

/// Keyword-matching rule engine: map requirements text to suggested section types.
///
/// Rules are evaluated case-insensitively against the full requirements text.
/// The `changes` transition manifest is always included.
/// `unit-test` is added when more than 2 keyword-matched sections are found.
/// `interaction`, `logic`, and `dependency` are added when more than 3
/// keyword-matched sections are found.
///
/// Returns deduplicated section names sorted by fill_order priority.
/// @spec projects/agentic-workflow/tech-design/core/tools/spec_plan_entry.md#changes
pub fn suggest_sections(requirements: &str) -> Vec<String> {
    use regex::Regex;

    // Build case-insensitive regex for each keyword pattern and collect matches.
    // Note: Rust's regex crate does not support lookaheads; patterns use (?i) inline flag
    // at the start only and avoid negative lookaheads.
    // For "api": match as a whole word (word boundaries ensure "capital" does not match).
    let keyword_rules: &[(&str, &[&str])] = &[
        // Existing types
        (
            r"(?i)\b(endpoint|route|api|REST|HTTP)\b",
            &["rest-api", "schema"],
        ),
        (r"(?i)\b(rpc|json-rpc|MCP\s+tool)\b", &["rpc-api", "schema"]),
        (
            r"(?i)\b(queue|pubsub|webhook|background|async)\b",
            &["async-api"],
        ),
        (
            r"(?i)\b(database|table|migration|collection)\b",
            &["db-model"],
        ),
        (
            r"(?i)\b(state|phase|lifecycle|transition)\b",
            &["state-machine"],
        ),
        (
            r"(?i)\b(UI|page|component|layout|frontend)\b",
            &["wireframe", "component"],
        ),
        (r"(?i)\b(CLI|command|subcommand|flag)\b", &["cli"]),
        (
            r"(?i)(\b(config|env|settings)\b|\.toml\b|\.env\b)",
            &["config"],
        ),
        (
            r"(?i)\b(token|color|spacing|typography|theme)\b",
            &["design-token"],
        ),
        // Backend API types
        (r"(?i)\b(grpc|protobuf|proto|gRPC)\b", &["grpc", "schema"]),
        (
            r"(?i)\b(graphql|mutation|subscription|SDL)\b",
            &["graphql", "schema"],
        ),
        // QA types
        (r"(?i)\b(e2e|end-to-end|acceptance\s+test)\b", &["e2e-test"]),
        (r"(?i)\b(fixture|test-data|seed\s+data)\b", &["unit-test"]),
        (
            r"(?i)\b(performance|load-test|benchmark|latency)\b",
            &["unit-test"],
        ),
        // Security types
        (
            r"(?i)\b(threat|STRIDE|attack\s+surface)\b",
            &["threat-model"],
        ),
        (
            r"(?i)\b(auth-matrix|RBAC|permission\s+matrix|authorization\s+matrix)\b",
            &["auth-matrix"],
        ),
        (
            r"(?i)\b(security-test|pen-test|penetration|OWASP)\b",
            &["e2e-test"],
        ),
        // SRE types
        (r"(?i)\b(container|docker|Dockerfile|OCI)\b", &["container"]),
        (
            r"(?i)\b(deploy|deployment|kubernetes|k8s|helm)\b",
            &["deploy"],
        ),
        (
            r"(?i)\b(cloud-resource|terraform|pulumi|cloud\s+resource)\b",
            &["cloud-resource"],
        ),
        (
            r"(?i)\b(pipeline|CI/CD|CICD|github\s+actions|gitlab\s+ci)\b",
            &["pipeline"],
        ),
        (
            r"(?i)\b(observability|monitoring|alerting|metrics|tracing|SLO)\b",
            &["observability"],
        ),
        // MLE types
        (
            r"(?i)\b(ML\s+model|machine.learning|training|inference|neural)\b",
            &["model"],
        ),
        // Agent types
        (
            r"(?i)\b(prompt|system.instruction|few.shot|prompt\s+template)\b",
            &["prompt"],
        ),
    ];

    let mut matched: std::collections::LinkedList<&str> = std::collections::LinkedList::new();

    for (pattern, sections) in keyword_rules {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(requirements) {
                for &s in *sections {
                    matched.push_back(s);
                }
            }
        }
    }

    // Collect keyword-matched sections (deduplicated, preserving insertion order).
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut result: Vec<String> = Vec::new();

    for s in &matched {
        if seen.insert(s.to_string()) {
            result.push(s.to_string());
        }
    }

    let keyword_section_count = result.len();

    // Conditional additions based on section count.
    if keyword_section_count > 3 {
        for s in &["interaction", "logic", "dependency"] {
            if seen.insert(s.to_string()) {
                result.push(s.to_string());
            }
        }
    }
    if keyword_section_count > 2 {
        if seen.insert("unit-test".to_string()) {
            result.push("unit-test".to_string());
        }
    }

    // Always-present transition manifest.
    for s in &["changes"] {
        if seen.insert(s.to_string()) {
            result.push(s.to_string());
        }
    }

    // Sort by fill_order priority.
    result.sort_by_key(|s| section_fill_order(s.as_str()));
    result
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/spec_plan.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:standardize-gap-sdd-spec-plan-section-suggestions>"
    description: "Keyword-based section suggestion and fill-order sorting rules for spec_plan entries."
```
