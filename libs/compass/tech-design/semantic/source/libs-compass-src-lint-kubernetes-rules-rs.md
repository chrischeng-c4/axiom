---
id: libs-compass-src-lint-kubernetes-rules-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/kubernetes_rules.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/lint/kubernetes_rules.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/kubernetes_rules.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `check_required_fields` | libs/compass/src/lint/kubernetes_rules.rs | function | pub | 7 | pub(super) fn check_required_fields(lines: &[&str]) -> Vec<Diagnostic> { |
| `check_missing_probes` | libs/compass/src/lint/kubernetes_rules.rs | function | pub | 40 | pub(super) fn check_missing_probes(lines: &[&str]) -> Vec<Diagnostic> { |
| `check_deprecated_api_versions` | libs/compass/src/lint/kubernetes_rules.rs | function | pub | 88 | pub(super) fn check_deprecated_api_versions(lines: &[&str]) -> Vec<Diagnostic> { |
| `check_duplicate_resources` | libs/compass/src/lint/kubernetes_rules.rs | function | pub | 116 | pub(super) fn check_duplicate_resources(lines: &[&str]) -> Vec<Diagnostic> { |
| `check_missing_labels` | libs/compass/src/lint/kubernetes_rules.rs | function | pub | 207 | pub(super) fn check_missing_labels(lines: &[&str]) -> Vec<Diagnostic> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Additional Kubernetes lint rules (K8002, K8005, K8008, K8009, K8010)

use crate::diagnostic::{Diagnostic, DiagnosticCategory, Position, Range};
use std::collections::HashMap;

/// K8002: Missing required fields (Deployment without spec.template)
pub(super) fn check_required_fields(lines: &[&str]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut kind_value: Option<String> = None;

    for line in lines.iter() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("kind:") {
            kind_value = Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
            break;
        }
    }

    if let Some(ref kind) = kind_value {
        if kind == "Deployment" || kind == "StatefulSet" || kind == "DaemonSet" {
            let has_template = lines.iter().any(|l| l.trim().starts_with("template:"));
            if !has_template {
                diagnostics.push(Diagnostic::warning(
                    Range::new(Position::new(0, 0), Position::new(0, 1)),
                    "K8002",
                    DiagnosticCategory::Logic,
                    format!(
                        "{} is missing 'spec.template' — this is a required field",
                        kind
                    ),
                ));
            }
        }
    }

    diagnostics
}

/// K8005: Missing liveness/readiness probes
pub(super) fn check_missing_probes(lines: &[&str]) -> Vec<Diagnostic> {
    let has_containers = lines.iter().any(|l| {
        let t = l.trim();
        t.starts_with("containers:") || t.starts_with("- name:")
    });
    if !has_containers {
        return Vec::new();
    }

    let has_liveness = lines.iter().any(|l| l.trim().starts_with("livenessProbe:"));
    let has_readiness = lines
        .iter()
        .any(|l| l.trim().starts_with("readinessProbe:"));

    let mut diagnostics = Vec::new();
    if !has_liveness {
        diagnostics.push(Diagnostic::warning(
            Range::new(Position::new(0, 0), Position::new(0, 1)),
            "K8005",
            DiagnosticCategory::Logic,
            "Missing 'livenessProbe' — Kubernetes cannot detect if the container is deadlocked",
        ));
    }
    if !has_readiness {
        diagnostics.push(Diagnostic::warning(
            Range::new(Position::new(0, 0), Position::new(0, 1)),
            "K8005",
            DiagnosticCategory::Logic,
            "Missing 'readinessProbe' — traffic may be sent to containers that are not ready",
        ));
    }

    diagnostics
}

/// Deprecated apiVersion values
const DEPRECATED_API_VERSIONS: &[&str] = &[
    "extensions/v1beta1",
    "apps/v1beta1",
    "apps/v1beta2",
    "networking.k8s.io/v1beta1",
    "rbac.authorization.k8s.io/v1beta1",
    "scheduling.k8s.io/v1beta1",
    "policy/v1beta1",
    "autoscaling/v2beta1",
];

/// K8008: Deprecated API versions
pub(super) fn check_deprecated_api_versions(lines: &[&str]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("apiVersion:") {
            let version = value.trim().trim_matches('"').trim_matches('\'');
            if DEPRECATED_API_VERSIONS.contains(&version) {
                diagnostics.push(Diagnostic::warning(
                    Range::new(
                        Position::new(line_num as u32, 0),
                        Position::new(line_num as u32, line.len() as u32),
                    ),
                    "K8008",
                    DiagnosticCategory::Logic,
                    format!(
                        "Deprecated apiVersion '{}' — migrate to the stable API version",
                        version,
                    ),
                ));
            }
        }
    }

    diagnostics
}

/// K8009: Duplicate resource names (same name + kind pair)
pub(super) fn check_duplicate_resources(lines: &[&str]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<(String, String), usize> = HashMap::new();

    let mut current_kind: Option<String> = None;
    let mut current_name: Option<String> = None;
    let mut doc_start_line: usize = 0;
    let mut in_metadata = false;
    let mut metadata_indent: Option<usize> = None;

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        if trimmed == "---" {
            if let (Some(k), Some(n)) = (current_kind.take(), current_name.take()) {
                let key = (k, n);
                if let Some(prev_line) = seen.get(&key) {
                    diagnostics.push(Diagnostic::warning(
                        Range::new(
                            Position::new(doc_start_line as u32, 0),
                            Position::new(doc_start_line as u32, 1),
                        ),
                        "K8009",
                        DiagnosticCategory::Logic,
                        format!(
                            "Duplicate resource {}/{} (first seen at line {})",
                            key.0,
                            key.1,
                            prev_line + 1,
                        ),
                    ));
                } else {
                    seen.insert(key, doc_start_line);
                }
            }
            doc_start_line = line_num;
            in_metadata = false;
            metadata_indent = None;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("kind:") {
            if indent == 0 {
                current_kind = Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
            }
        }

        if trimmed == "metadata:" && indent == 0 {
            in_metadata = true;
            metadata_indent = Some(indent);
            continue;
        }

        if in_metadata {
            if let Some(mi) = metadata_indent {
                if indent <= mi && !trimmed.is_empty() {
                    in_metadata = false;
                } else if let Some(rest) = trimmed.strip_prefix("name:") {
                    current_name =
                        Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
                }
            }
        }
    }

    // Finalize last document
    if let (Some(k), Some(n)) = (current_kind, current_name) {
        let key = (k, n);
        if let Some(prev_line) = seen.get(&key) {
            diagnostics.push(Diagnostic::warning(
                Range::new(
                    Position::new(doc_start_line as u32, 0),
                    Position::new(doc_start_line as u32, 1),
                ),
                "K8009",
                DiagnosticCategory::Logic,
                format!(
                    "Duplicate resource {}/{} (first seen at line {})",
                    key.0,
                    key.1,
                    prev_line + 1,
                ),
            ));
        }
    }

    diagnostics
}

/// K8010: Missing labels in metadata
pub(super) fn check_missing_labels(lines: &[&str]) -> Vec<Diagnostic> {
    let mut in_metadata = false;
    let mut metadata_indent: Option<usize> = None;
    let mut has_labels = false;

    for line in lines.iter() {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        if trimmed.starts_with("metadata:") && indent == 0 {
            in_metadata = true;
            metadata_indent = Some(indent);
            continue;
        }

        if in_metadata {
            if let Some(mi) = metadata_indent {
                if indent <= mi && !trimmed.is_empty() {
                    in_metadata = false;
                } else if trimmed.starts_with("labels:") {
                    has_labels = true;
                    break;
                }
            }
        }
    }

    let has_metadata = lines.iter().any(|l| l.trim().starts_with("metadata:"));

    if has_metadata && !has_labels {
        vec![Diagnostic::warning(
            Range::new(Position::new(0, 0), Position::new(0, 1)),
            "K8010",
            DiagnosticCategory::Style,
            "Missing 'labels' in metadata — labels enable filtering, selecting, and organizing resources",
        )]
    } else {
        Vec::new()
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/kubernetes_rules.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/kubernetes_rules.rs` captured during libs codegen standardization.
```
