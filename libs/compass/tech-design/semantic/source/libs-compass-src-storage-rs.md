---
id: libs-compass-src-storage-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/storage.rs`.
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

# Standardized libs/compass/src/storage.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/storage.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `resolve_lens_storage` | libs/compass/src/storage.rs | function | pub | 12 | pub fn resolve_lens_storage(project_root: &Path) -> std::io::Result<PathBuf> { |
| `resolve_module_index` | libs/compass/src/storage.rs | function | pub | 20 | pub fn resolve_module_index(project_root: &Path, module_name: &str) -> std::io::Result<PathBuf> { |
| `resolve_pid_file` | libs/compass/src/storage.rs | function | pub | 28 | pub fn resolve_pid_file(project_root: &Path) -> std::io::Result<PathBuf> { |
| `resolve_socket_path` | libs/compass/src/storage.rs | function | pub | 36 | pub fn resolve_socket_path(project_root: &Path) -> std::io::Result<PathBuf> { |
| `resolve_cache_dir` | libs/compass/src/storage.rs | function | pub | 44 | pub fn resolve_cache_dir(project_root: &Path) -> std::io::Result<PathBuf> { |
| `resolve_scope_cache_dir` | libs/compass/src/storage.rs | function | pub | 52 | pub fn resolve_scope_cache_dir(project_root: &Path, scope_id: &str) -> std::io::Result<PathBuf> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Persistent index storage path resolution.
//!
//! Resolves the persistent storage directory for Lens code indexes at
//! `{project_dir}/cclab/.index/`. Indexes are stored locally within each
//! project for portability and easy cleanup.

use std::path::{Path, PathBuf};

/// Resolve the persistent Lens storage root for a project.
///
/// Returns `{project_root}/cclab/.index/`.
pub fn resolve_lens_storage(project_root: &Path) -> std::io::Result<PathBuf> {
    let canonical = project_root.canonicalize()?;
    Ok(canonical.join("cclab").join(".index"))
}

/// Resolve a module-specific index path within the Lens storage directory.
///
/// Returns `{project_root}/cclab/.index/{module_name}.idx`
pub fn resolve_module_index(project_root: &Path, module_name: &str) -> std::io::Result<PathBuf> {
    let root = resolve_lens_storage(project_root)?;
    Ok(root.join(format!("{}.idx", module_name)))
}

/// Resolve the PID file path for the daemon.
///
/// Returns `{project_root}/cclab/.index/daemon.pid`
pub fn resolve_pid_file(project_root: &Path) -> std::io::Result<PathBuf> {
    let root = resolve_lens_storage(project_root)?;
    Ok(root.join("daemon.pid"))
}

/// Resolve the socket path for the daemon.
///
/// Returns `{project_root}/cclab/.index/daemon.sock`
pub fn resolve_socket_path(project_root: &Path) -> std::io::Result<PathBuf> {
    let root = resolve_lens_storage(project_root)?;
    Ok(root.join("daemon.sock"))
}

/// Resolve the persistent AST index cache directory.
///
/// Returns `{project_root}/cclab/.index/cache/`
pub fn resolve_cache_dir(project_root: &Path) -> std::io::Result<PathBuf> {
    let root = resolve_lens_storage(project_root)?;
    Ok(root.join("cache"))
}

/// Resolve per-scope cache directory (#1127).
///
/// Returns `{project_root}/cclab/.index/scopes/{scope_id}/cache/`
pub fn resolve_scope_cache_dir(project_root: &Path, scope_id: &str) -> std::io::Result<PathBuf> {
    let root = resolve_lens_storage(project_root)?;
    Ok(root.join("scopes").join(scope_id).join("cache"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_lens_storage() {
        let temp = TempDir::new().unwrap();
        let path = resolve_lens_storage(temp.path()).unwrap();

        assert!(path.to_string_lossy().ends_with("cclab/.index"));
        // Use canonical path for comparison to handle macOS /var -> /private/var symlinks
        let canonical_temp = temp.path().canonicalize().unwrap();
        assert!(path.starts_with(&canonical_temp));
    }

    #[test]
    fn test_resolve_same_path_gives_same_result() {
        let temp = TempDir::new().unwrap();
        let path1 = resolve_lens_storage(temp.path()).unwrap();
        let path2 = resolve_lens_storage(temp.path()).unwrap();
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_resolve_different_paths_give_different_results() {
        let temp1 = TempDir::new().unwrap();
        let temp2 = TempDir::new().unwrap();
        let path1 = resolve_lens_storage(temp1.path()).unwrap();
        let path2 = resolve_lens_storage(temp2.path()).unwrap();
        assert_ne!(path1, path2);
    }

    #[test]
    fn test_resolve_module_index() {
        let temp = TempDir::new().unwrap();
        let path = resolve_module_index(temp.path(), "backend").unwrap();

        assert!(path.to_string_lossy().ends_with("cclab/.index/backend.idx"));
    }

    #[test]
    fn test_resolve_pid_file() {
        let temp = TempDir::new().unwrap();
        let path = resolve_pid_file(temp.path()).unwrap();

        assert!(path.to_string_lossy().ends_with("cclab/.index/daemon.pid"));
    }

    #[test]
    fn test_resolve_socket_path() {
        let temp = TempDir::new().unwrap();
        let path = resolve_socket_path(temp.path()).unwrap();

        assert!(path.to_string_lossy().ends_with("cclab/.index/daemon.sock"));
    }

    #[test]
    fn test_resolve_cache_dir() {
        let temp = TempDir::new().unwrap();
        let path = resolve_cache_dir(temp.path()).unwrap();

        assert!(path.to_string_lossy().ends_with("cclab/.index/cache"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/storage.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/storage.rs` captured during libs codegen standardization.
```
