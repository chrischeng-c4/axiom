//! Kubernetes YAML lint checker (source-line analysis)

use super::kubernetes_rules;
use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};

/// Kubernetes manifest checker — uses source-line analysis on YAML
pub struct KubernetesChecker;

impl KubernetesChecker {
    pub fn new() -> Self {
        Self
    }

    /// Known valid apiVersion values
    const KNOWN_API_VERSIONS: &'static [&'static str] = &[
        "v1",
        "apps/v1",
        "batch/v1",
        "batch/v1beta1",
        "networking.k8s.io/v1",
        "networking.k8s.io/v1beta1",
        "rbac.authorization.k8s.io/v1",
        "rbac.authorization.k8s.io/v1beta1",
        "policy/v1",
        "policy/v1beta1",
        "autoscaling/v1",
        "autoscaling/v2",
        "autoscaling/v2beta1",
        "autoscaling/v2beta2",
        "storage.k8s.io/v1",
        "admissionregistration.k8s.io/v1",
        "apiextensions.k8s.io/v1",
        "coordination.k8s.io/v1",
        "certificates.k8s.io/v1",
        "events.k8s.io/v1",
        "scheduling.k8s.io/v1",
        "discovery.k8s.io/v1",
        "flowcontrol.apiserver.k8s.io/v1beta3",
    ];

    /// K8001: Invalid or missing apiVersion/kind
    fn check_api_version(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut found_api_version = false;
        let mut found_kind = false;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if let Some(value) = trimmed.strip_prefix("apiVersion:") {
                found_api_version = true;
                let version = value.trim().trim_matches('"').trim_matches('\'');
                if !version.is_empty()
                    && !Self::KNOWN_API_VERSIONS.contains(&version)
                    && !version.contains('/')
                {
                    diagnostics.push(Diagnostic::warning(
                        Range::new(
                            Position::new(line_num as u32, 0),
                            Position::new(line_num as u32, line.len() as u32),
                        ),
                        "K8001",
                        DiagnosticCategory::Logic,
                        format!("Unknown apiVersion: '{}' — verify this is correct", version),
                    ));
                }
            }

            if trimmed.starts_with("kind:") {
                found_kind = true;
            }
        }

        if !found_api_version {
            diagnostics.push(Diagnostic::error(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                "K8001",
                DiagnosticCategory::Logic,
                "Missing 'apiVersion' field in Kubernetes manifest",
            ));
        }
        if !found_kind {
            diagnostics.push(Diagnostic::error(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                "K8001",
                DiagnosticCategory::Logic,
                "Missing 'kind' field in Kubernetes manifest",
            ));
        }

        diagnostics
    }

    /// K8003: Using `latest` image tag or no tag
    fn check_latest_image(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if let Some(rest) = Self::strip_yaml_key(trimmed, "image") {
                let image = rest.trim_matches('"').trim_matches('\'').trim();
                if image.is_empty() {
                    continue;
                }
                let uses_latest = image.ends_with(":latest");
                let has_tag = image.contains(':') || image.contains('@');
                if uses_latest || !has_tag {
                    let msg = if uses_latest {
                        format!(
                            "Image '{}' uses ':latest' tag — pin to a specific version",
                            image
                        )
                    } else {
                        format!(
                            "Image '{}' has no tag — defaults to ':latest', pin a version",
                            image
                        )
                    };
                    diagnostics.push(Diagnostic::warning(
                        Range::new(
                            Position::new(line_num as u32, 0),
                            Position::new(line_num as u32, line.len() as u32),
                        ),
                        "K8003",
                        DiagnosticCategory::Security,
                        msg,
                    ));
                }
            }
        }

        diagnostics
    }

    /// K8004: Missing resource limits
    fn check_resource_limits(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let has_resources = lines.iter().any(|l| l.trim().starts_with("resources:"));
        let has_limits = lines.iter().any(|l| l.trim().starts_with("limits:"));
        let has_containers = Self::has_pod_containers(lines);

        if has_containers && (!has_resources || !has_limits) {
            vec![Diagnostic::warning(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                "K8004",
                DiagnosticCategory::Logic,
                "Missing resource limits — set resources.limits to prevent unbounded resource usage",
            )]
        } else {
            Vec::new()
        }
    }

    /// K8006: Running as root (missing securityContext.runAsNonRoot)
    fn check_run_as_root(&self, lines: &[&str]) -> Vec<Diagnostic> {
        if !Self::has_pod_containers(lines) {
            return Vec::new();
        }

        let has_security_context = lines
            .iter()
            .any(|l| l.trim().starts_with("securityContext:"));
        let has_run_as_non_root = lines.iter().any(|l| {
            let t = l.trim();
            t.starts_with("runAsNonRoot:") && t.contains("true")
        });

        if !has_security_context || !has_run_as_non_root {
            vec![Diagnostic::new(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                DiagnosticSeverity::Information,
                "K8006",
                DiagnosticCategory::Security,
                "Missing securityContext.runAsNonRoot: true — container may run as root",
            )]
        } else {
            Vec::new()
        }
    }

    /// K8007: Missing namespace in metadata
    fn check_missing_namespace(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut in_metadata = false;
        let mut metadata_indent: Option<usize> = None;
        let mut has_namespace = false;

        for line in lines.iter() {
            let trimmed = line.trim();
            let indent = line.len() - line.trim_start().len();

            if trimmed.starts_with("metadata:") {
                in_metadata = true;
                metadata_indent = Some(indent);
                continue;
            }

            if in_metadata {
                if let Some(mi) = metadata_indent {
                    if indent <= mi && !trimmed.is_empty() {
                        in_metadata = false;
                    } else if trimmed.starts_with("namespace:") {
                        has_namespace = true;
                        break;
                    }
                }
            }
        }

        let has_metadata = lines.iter().any(|l| l.trim().starts_with("metadata:"));

        if has_metadata && !has_namespace {
            vec![Diagnostic::new(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                DiagnosticSeverity::Hint,
                "K8007",
                DiagnosticCategory::Style,
                "Missing 'namespace' in metadata — resource will be created in the default namespace",
            )]
        } else {
            Vec::new()
        }
    }

    /// Helper: strip a YAML key prefix and return the value portion
    fn strip_yaml_key<'a>(line: &'a str, key: &str) -> Option<&'a str> {
        let stripped = line.strip_prefix(key)?;
        let stripped = stripped.strip_prefix(':')?;
        Some(stripped.trim())
    }

    fn has_pod_containers(lines: &[&str]) -> bool {
        let pod_workload = lines.iter().any(|line| {
            matches!(
                Self::strip_yaml_key(line.trim(), "kind")
                    .map(|kind| kind.trim_matches('"').trim_matches('\'')),
                Some(
                    "Pod"
                        | "Deployment"
                        | "StatefulSet"
                        | "DaemonSet"
                        | "ReplicaSet"
                        | "Job"
                        | "CronJob"
                )
            )
        });
        pod_workload
            && lines
                .iter()
                .any(|line| line.trim().starts_with("containers:"))
    }

    /// Quick check: does this look like a Kubernetes manifest?
    pub(super) fn is_k8s_manifest(lines: &[&str]) -> bool {
        let has_api = lines.iter().any(|l| l.trim().starts_with("apiVersion:"));
        let has_kind = lines.iter().any(|l| l.trim().starts_with("kind:"));
        has_api || has_kind
    }
}

impl Default for KubernetesChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for KubernetesChecker {
    fn language(&self) -> Language {
        Language::Yaml
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let lines: Vec<&str> = file.source.lines().collect();

        if !Self::is_k8s_manifest(&lines) {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();

        diagnostics.extend(self.check_api_version(&lines));
        diagnostics.extend(self.check_latest_image(&lines));
        diagnostics.extend(self.check_resource_limits(&lines));
        diagnostics.extend(self.check_run_as_root(&lines));
        diagnostics.extend(self.check_missing_namespace(&lines));
        // Delegated rules (kubernetes_rules module)
        diagnostics.extend(kubernetes_rules::check_required_fields(&lines));
        diagnostics.extend(kubernetes_rules::check_missing_probes(&lines));
        diagnostics.extend(kubernetes_rules::check_deprecated_api_versions(&lines));
        diagnostics.extend(kubernetes_rules::check_duplicate_resources(&lines));
        diagnostics.extend(kubernetes_rules::check_missing_labels(&lines));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "K8001", // Invalid/missing apiVersion or kind
            "K8003", // Using latest image tag
            "K8004", // Missing resource limits
            "K8006", // Running as root
            "K8007", // Missing namespace
            "K8002", // Missing required fields
            "K8005", // Missing liveness/readiness probes
            "K8008", // Deprecated API versions
            "K8009", // Duplicate resource names
            "K8010", // Missing labels
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::KubernetesChecker;

    #[test]
    fn pod_workload_with_containers_needs_runtime_security_checks() {
        let lines = [
            "apiVersion: apps/v1",
            "kind: StatefulSet",
            "spec:",
            "  template:",
            "    spec:",
            "      containers:",
            "        - name: relay",
        ];

        assert!(KubernetesChecker::has_pod_containers(&lines));
    }

    #[test]
    fn service_port_name_is_not_a_container() {
        let lines = [
            "apiVersion: v1",
            "kind: Service",
            "spec:",
            "  ports:",
            "    - name: http",
            "      port: 8080",
        ];

        assert!(!KubernetesChecker::has_pod_containers(&lines));
    }
}
