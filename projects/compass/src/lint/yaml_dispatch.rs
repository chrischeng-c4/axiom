//! YAML/JSON dispatcher — routes to the appropriate sub-checker based on content

use super::asyncapi::AsyncApiChecker;
use super::openapi::OpenApiChecker;
use super::openrpc::OpenRpcChecker;
use super::{Checker, GitlabCiChecker, KubernetesChecker};
use crate::checker::LintConfig;
use crate::diagnostic::Diagnostic;
use crate::syntax::{Language, ParsedFile};

/// Composite checker that dispatches YAML/JSON files to the appropriate sub-checker.
///
/// Routing priority:
/// 1. OpenAPI 3.x  — `openapi: 3.` in first 10 lines
/// 2. AsyncAPI     — `asyncapi:` in first 10 lines
/// 3. OpenRPC      — `"openrpc"` and `"methods"` in source
/// 4. Kubernetes   — `apiVersion:` and `kind:` at line start
/// 5. GitLab CI    — fallback
pub struct YamlDispatcher {
    k8s: KubernetesChecker,
    gitlab: GitlabCiChecker,
    openapi: OpenApiChecker,
    asyncapi: AsyncApiChecker,
    openrpc: OpenRpcChecker,
}

impl YamlDispatcher {
    pub fn new() -> Self {
        Self {
            k8s: KubernetesChecker::new(),
            gitlab: GitlabCiChecker::new(),
            openapi: OpenApiChecker::new(),
            asyncapi: AsyncApiChecker::new(),
            openrpc: OpenRpcChecker::new(),
        }
    }

    /// Determine whether the source looks like a Kubernetes manifest.
    ///
    /// A file is treated as K8s when it has at least one line starting with
    /// `apiVersion:` AND at least one line starting with `kind:`.
    fn is_kubernetes(source: &str) -> bool {
        let mut has_api_version = false;
        let mut has_kind = false;

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("apiVersion:") {
                has_api_version = true;
            }
            if trimmed.starts_with("kind:") {
                has_kind = true;
            }
            if has_api_version && has_kind {
                return true;
            }
        }

        false
    }
}

impl Default for YamlDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for YamlDispatcher {
    fn language(&self) -> Language {
        Language::Yaml
    }

    fn check(&self, file: &ParsedFile, config: &LintConfig) -> Vec<Diagnostic> {
        if OpenApiChecker::is_openapi(&file.source) {
            self.openapi.check(file, config)
        } else if AsyncApiChecker::is_asyncapi(&file.source) {
            self.asyncapi.check(file, config)
        } else if OpenRpcChecker::is_openrpc(&file.source) {
            self.openrpc.check(file, config)
        } else if Self::is_kubernetes(&file.source) {
            self.k8s.check(file, config)
        } else {
            self.gitlab.check(file, config)
        }
    }

    fn available_rules(&self) -> Vec<&'static str> {
        let mut rules = self.k8s.available_rules();
        rules.extend(self.gitlab.available_rules());
        rules.extend(self.openapi.available_rules());
        rules.extend(self.asyncapi.available_rules());
        rules.extend(self.openrpc.available_rules());
        rules
    }
}
