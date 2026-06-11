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
