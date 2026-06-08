---
id: sdd-spec-store
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Change/context/git/spec-store logic supports TD/CB artifact lifecycle dispatch and review state."
---

# FileSystemSpecStore Type

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_store.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FileSystemSpecStore` | projects/agentic-workflow/src/spec_store.rs | struct | pub | 21 |  |
| `from_config` | projects/agentic-workflow/src/spec_store.rs | function | pub | 47 | from_config(root: PathBuf, scopes: HashMap<String, String>) -> Self |
| `new` | projects/agentic-workflow/src/spec_store.rs | function | pub | 37 | new(root: PathBuf, scopes: HashMap<String, String>) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  FileSystemSpecStore:
    type: object
    required: [root, scopes]
    description: Filesystem-backed spec store implementation.
    properties:
      root:
        type: string
        x-rust-type: "PathBuf"
        description: "Project root (the directory that contains `.aw/tech-design/`)."
      scopes:
        type: object
        x-rust-type: "HashMap<String, String>"
        description: "Scope map: group name → parent subdirectory under `.aw/tech-design/`."
    x-rust-struct:
      derive: [Debug, Clone]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/spec_store.rs -->
````rust
//! Filesystem-backed implementation of [`SpecStore`].
//!
//! [`FileSystemSpecStore`] scans spec files under the `.aw/tech-design/` directory
//! tree, resolves group directories using a configurable scope map (identical
//! in shape to `SddConfig.specs.scopes`), and serves keyword-ranked search
//! results plus direct file reads.

use crate::agents::restructure::{SpecExcerpt, SpecStore};
use crate::services::project_registry::resolve_td_root_from_config;
use crate::shared::workspace;
use agent::error::NovaResult;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Filesystem-backed spec store implementation.
/// @spec projects/agentic-workflow/tech-design/core/logic/spec_store.md#schema
#[derive(Debug, Clone)]
pub struct FileSystemSpecStore {
    /// Project root (the directory that contains `.aw/tech-design/`).
    pub root: PathBuf,
    /// Scope map: group name → parent subdirectory under `.aw/tech-design/`.
    pub scopes: HashMap<String, String>,
}
/// @spec projects/agentic-workflow/tech-design/core/logic/spec_store.md#source
impl FileSystemSpecStore {
    /// Create a new store.
    ///
    /// `root` is the project root — `.aw/tech-design/` is expected at
    /// `{root}/.aw/tech-design/`.
    ///
    /// `scopes` maps spec group names to their parent subdirectory under
    /// `.aw/tech-design/` (e.g. `{ "sdd" → "crates" }`). Pass an empty map
    /// to rely entirely on the fallback probe order.
    pub fn new(root: PathBuf, scopes: HashMap<String, String>) -> Self {
        Self { root, scopes }
    }

    /// Convenience constructor — semantically equivalent to [`new`](Self::new).
    ///
    /// Callers with an `SddConfig` can do:
    /// ```rust,ignore
    /// FileSystemSpecStore::from_config(root, config.specs.scopes.clone())
    /// ```
    pub fn from_config(root: PathBuf, scopes: HashMap<String, String>) -> Self {
        Self::new(root, scopes)
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    fn specs_base(&self) -> PathBuf {
        workspace::tech_design_path(&self.root)
    }

    /// Resolve the directory for a spec group using the scope map, with
    /// classic fallback (`crates/` → `projects/` → root).
    ///
    /// Returns `None` when the group cannot be found by any probe.
    fn resolve_group_dir(&self, group: &str) -> Option<PathBuf> {
        let specs_base = self.specs_base();
        if let Ok(resolved) = resolve_td_root_from_config(&self.root, group) {
            let candidate = PathBuf::from(resolved.root);
            if candidate.exists() {
                return Some(candidate);
            }
            return None;
        }

        if let Some(subdir) = self.scopes.get(group) {
            let candidate = specs_base.join(subdir).join(group);
            if candidate.exists() {
                return Some(candidate);
            }
            // Explicitly configured but absent — no fallback.
            return None;
        }
        // Fallback probe order: crates/ → projects/ → root
        for prefix in &["crates", "projects"] {
            let candidate = specs_base.join(prefix).join(group);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        let root_candidate = specs_base.join(group);
        if root_candidate.exists() {
            return Some(root_candidate);
        }
        None
    }

    /// Collect all `.md` files under a directory tree (non-recursive first
    /// level only — specs are stored flat within each group directory).
    fn collect_spec_files(dir: &Path) -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return vec![];
        };
        entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("md"))
            })
            .map(|e| e.path())
            .collect()
    }

    /// Score the relevance of `content` to `query`.
    ///
    /// Simple keyword-frequency scorer: split query into whitespace-separated
    /// terms, count how many appear (case-insensitively) in the content, and
    /// normalise by the number of terms to obtain a score in `[0, 1]`.
    fn score_relevance(content: &str, query: &str) -> f32 {
        let terms: Vec<String> = query.split_whitespace().map(|t| t.to_lowercase()).collect();
        if terms.is_empty() {
            return 0.0;
        }
        let content_lower = content.to_lowercase();
        let matched = terms
            .iter()
            .filter(|t| content_lower.contains(t.as_str()))
            .count();
        matched as f32 / terms.len() as f32
    }

    /// Return the path of a spec file relative to `{root}/.aw/tech-design/`.
    fn relative_spec_path(&self, abs_path: &Path) -> String {
        let specs_base = self.specs_base();
        if let Ok(rel) = abs_path.strip_prefix(&specs_base) {
            return rel.to_string_lossy().into_owned();
        }
        for (project, td_root) in workspace::project_tech_design_paths(&self.root) {
            if let Ok(rel) = abs_path.strip_prefix(&td_root) {
                return PathBuf::from("projects")
                    .join(project)
                    .join(rel)
                    .to_string_lossy()
                    .into_owned();
            }
        }
        abs_path
            .strip_prefix(&self.root)
            .unwrap_or(abs_path)
            .to_string_lossy()
            .into_owned()
    }

    fn resolve_read_path(&self, path: &str) -> PathBuf {
        if let Some(rest) = path.strip_prefix("projects/") {
            let mut parts = rest.splitn(2, '/');
            if let (Some(project), Some(project_rel)) = (parts.next(), parts.next()) {
                for (name, td_root) in workspace::project_tech_design_paths(&self.root) {
                    if name == project {
                        return td_root.join(project_rel);
                    }
                }
            }
        }

        self.specs_base().join(path)
    }

    /// Collect candidate files for a search.
    ///
    /// When `scopes` is non-empty, restricts the scan to configured group
    /// directories (and their fallback equivalents). Otherwise scans the
    /// entire `.aw/tech-design/` tree.
    fn candidate_files(&self) -> Vec<PathBuf> {
        let specs_base = self.specs_base();
        let configured_roots = workspace::project_tech_design_paths(&self.root);
        if !specs_base.exists() && configured_roots.iter().all(|(_, path)| !path.exists()) {
            return vec![];
        }

        if !self.scopes.is_empty() {
            // Only scan configured (and resolvable) group dirs.
            let mut files = Vec::new();
            for group in self.scopes.keys() {
                if let Some(dir) = self.resolve_group_dir(group) {
                    files.extend(self.walk_specs_tree(&dir));
                }
            }
            files
        } else {
            // Scan entire specs tree with walkdir-style depth-limited traversal.
            let mut files = Vec::new();
            if specs_base.exists() {
                files.extend(self.walk_specs_tree(&specs_base));
            }
            for (_, td_root) in configured_roots {
                if td_root.exists() && td_root != specs_base {
                    files.extend(self.walk_specs_tree(&td_root));
                }
            }
            files
        }
    }

    /// Walk the specs tree collecting all `.md` files up to a reasonable depth.
    fn walk_specs_tree(&self, base: &Path) -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(base) else {
            return vec![];
        };
        let mut files = Vec::new();
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                // Recurse one level into group dirs
                files.extend(Self::collect_spec_files(&path));
                // And one more level for nested subdirs (e.g. logic/, interfaces/)
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub in sub_entries.filter_map(|e| e.ok()) {
                        let sub_path = sub.path();
                        if sub_path.is_dir() {
                            files.extend(Self::collect_spec_files(&sub_path));
                        }
                    }
                }
            } else if path
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("md"))
            {
                files.push(path);
            }
        }
        files
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/spec_store.md#source
impl SpecStore for FileSystemSpecStore {
    /// Return spec excerpts ranked by relevance to `query`.
    ///
    /// Scans all spec files in configured scope directories (or the full
    /// `.aw/tech-design/` tree when no scopes are configured). Each file is scored
    /// by keyword frequency; results with score > 0 are returned sorted
    /// descending by relevance.
    async fn search(&self, query: &str) -> NovaResult<Vec<SpecExcerpt>> {
        let candidates = self.candidate_files();

        let mut results: Vec<SpecExcerpt> = candidates
            .iter()
            .filter_map(|path| {
                let content = std::fs::read_to_string(path).ok()?;
                let score = Self::score_relevance(&content, query);
                if score <= 0.0 {
                    return None;
                }
                // Use up to the first 500 characters as the excerpt snippet.
                let excerpt_len = content.len().min(500);
                let excerpt = content[..excerpt_len].to_string();
                Some(SpecExcerpt {
                    path: self.relative_spec_path(path),
                    content: excerpt,
                    relevance: score,
                })
            })
            .collect();

        // Sort descending by relevance score.
        results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// Return the full text of a spec file identified by its path relative to
    /// `{root}/.aw/tech-design/`.
    ///
    /// The `path` parameter should not start with a `/`.
    async fn read(&self, path: &str) -> NovaResult<String> {
        let full_path = self.resolve_read_path(path);
        std::fs::read_to_string(&full_path).map_err(|e| {
            agent::error::NovaError::FileError(format!("Failed to read spec '{}': {}", path, e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: create a temp spec file and return the root path.
    fn make_spec(root: &Path, rel_path: &str, content: &str) {
        let full = root.join(rel_path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, content).unwrap();
    }

    fn write_config(root: &Path, content: &str) {
        let full = root.join(".aw/config.toml");
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, content).unwrap();
    }

    // TC_fs_store_search — REQ-6: FileSystemSpecStore.search returns ranked excerpts
    #[tokio::test]
    async fn test_search_returns_ranked_results() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().to_path_buf();

        make_spec(
            &root,
            "projects/agentic-workflow/tech-design/core/scope-resolution.md",
            "# Scope Resolution\n\nScope resolution handles group directory lookup for specs.\n",
        );
        make_spec(
            &root,
            "projects/agentic-workflow/tech-design/core/state-machine.md",
            "# State Machine\n\nPhase transitions and state management logic.\n",
        );

        let mut scopes = HashMap::new();
        scopes.insert("sdd".to_string(), "projects".to_string());
        let store = FileSystemSpecStore::new(root, scopes);

        let results = store.search("scope resolution").await.unwrap();

        assert!(
            !results.is_empty(),
            "should find at least one matching spec"
        );
        // Best match must be scope-resolution.md (contains both query terms)
        assert!(
            results[0].path.contains("scope-resolution"),
            "top result should be scope-resolution.md, got: {}",
            results[0].path
        );
        // All results have positive relevance
        for r in &results {
            assert!(r.relevance > 0.0);
        }
        // Results sorted descending by relevance
        for i in 1..results.len() {
            assert!(
                results[i - 1].relevance >= results[i].relevance,
                "results not sorted by relevance"
            );
        }
    }

    // TC_fs_store_read — REQ-6: FileSystemSpecStore.read returns file contents by path
    #[tokio::test]
    async fn test_read_returns_file_content() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().to_path_buf();

        make_spec(
            &root,
            "projects/agentic-workflow/tech-design/core/state-machine.md",
            "# State Machine\n\nTransitions here.\n",
        );

        let mut scopes = HashMap::new();
        scopes.insert("sdd".to_string(), "projects".to_string());
        let store = FileSystemSpecStore::new(root, scopes);

        let content = store.read("projects/agentic-workflow/state-machine.md").await.unwrap();
        assert!(content.contains("State Machine"));
        assert!(content.contains("Transitions here."));
    }

    #[tokio::test]
    async fn test_read_error_for_nonexistent_file() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();

        let store = FileSystemSpecStore::new(root, HashMap::new());
        let result = store.read("nonexistent/file.md").await;
        assert!(
            result.is_err(),
            "reading a nonexistent file should return Err"
        );
    }

    // Empty scopes → scan full specs tree (backward compat)
    #[tokio::test]
    async fn test_search_with_empty_scopes_scans_full_tree() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().to_path_buf();

        // Spec under crates/ (fallback tree)
        make_spec(
            &root,
            ".aw/tech-design/crates/cclab-pg/query.md",
            "# Query Builder\n\nPostgres query builder spec for database.\n",
        );

        // No scopes — should scan entire specs tree
        let store = FileSystemSpecStore::new(root, HashMap::new());
        let results = store.search("postgres query").await.unwrap();

        assert!(
            !results.is_empty(),
            "should find spec when scanning full tree"
        );
        assert!(results[0].path.contains("query"));
    }

    #[tokio::test]
    async fn test_search_uses_configured_global_base() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        write_config(
            &root,
            r#"
[agentic_workflow.tech_design_platform]
path = "docs/td"
"#,
        );
        make_spec(
            &root,
            "docs/td/projects/agentic-workflow/configurable.md",
            "# Configurable\n\nConfigured base path search target.\n",
        );

        let store = FileSystemSpecStore::new(root, HashMap::new());
        let results = store.search("configured target").await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].path, "projects/agentic-workflow/configurable.md");
    }

    #[tokio::test]
    async fn test_search_and_read_use_project_td_path() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        write_config(
            &root,
            r#"
[agentic_workflow.tech_design_platform]
path = ".aw/tech-design"

[[projects]]
name = "agentic-workflow"
path = "projects/agentic-workflow"
td_path = "projects/agentic-workflow/tech-design"
"#,
        );
        make_spec(
            &root,
            "projects/agentic-workflow/tech-design/logic/runtime.md",
            "# Runtime\n\nPer project td path search target.\n",
        );

        let mut scopes = HashMap::new();
        scopes.insert("sdd".to_string(), "projects".to_string());
        let store = FileSystemSpecStore::new(root, scopes);
        let results = store.search("per project target").await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].path, "projects/agentic-workflow/logic/runtime.md");
        let content = store.read("projects/agentic-workflow/logic/runtime.md").await.unwrap();
        assert!(content.contains("Runtime"));
    }

    // from_config is a semantic alias for new
    #[test]
    fn test_from_config_is_alias_for_new() {
        let root = PathBuf::from("/tmp");
        let mut scopes = HashMap::new();
        scopes.insert("sdd".to_string(), "projects".to_string());

        let s1 = FileSystemSpecStore::new(root.clone(), scopes.clone());
        let s2 = FileSystemSpecStore::from_config(root.clone(), scopes.clone());

        assert_eq!(s1.root, s2.root);
        assert_eq!(s1.scopes, s2.scopes);
    }

    #[test]
    fn test_score_relevance_no_match() {
        let score = FileSystemSpecStore::score_relevance(
            "completely unrelated content here",
            "scope resolution config",
        );
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_score_relevance_full_match() {
        let score = FileSystemSpecStore::score_relevance(
            "scope resolution config state machine logic",
            "scope resolution",
        );
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_score_relevance_partial_match() {
        // "scope" matches, "resolution" does not → 0.5
        let score =
            FileSystemSpecStore::score_relevance("only scope appears here", "scope resolution");
        assert_eq!(score, 0.5);
    }

    #[test]
    fn test_score_relevance_empty_query() {
        let score = FileSystemSpecStore::score_relevance("anything", "");
        assert_eq!(score, 0.0);
    }

    // TC_resolve_group_dir: configured group found on disk → returns Some
    #[test]
    fn test_resolve_group_dir_config_hit() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let dir = root.join("projects/agentic-workflow/tech-design/core");
        std::fs::create_dir_all(&dir).unwrap();

        let mut scopes = HashMap::new();
        scopes.insert("sdd".to_string(), "projects".to_string());
        let store = FileSystemSpecStore::new(root, scopes);

        assert_eq!(store.resolve_group_dir("sdd"), Some(dir));
    }

    // Configured group but directory absent → None (no fallback for explicit config)
    #[test]
    fn test_resolve_group_dir_config_miss_no_fallback() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        // Create the crates/ fallback path (should be ignored)
        std::fs::create_dir_all(root.join(".aw/tech-design/crates/cclab-foo")).unwrap();

        let mut scopes = HashMap::new();
        scopes.insert("cclab-foo".to_string(), "custom".to_string()); // configured to "custom"
        let store = FileSystemSpecStore::new(root, scopes);

        // custom/cclab-foo doesn't exist → None; crates/cclab-foo is not consulted
        assert_eq!(store.resolve_group_dir("cclab-foo"), None);
    }

    // No config for group → classic crates/ fallback
    #[test]
    fn test_resolve_group_dir_fallback_crates() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let dir = root.join(".aw/tech-design/crates/my-group");
        std::fs::create_dir_all(&dir).unwrap();

        let store = FileSystemSpecStore::new(root.clone(), HashMap::new());
        assert_eq!(store.resolve_group_dir("my-group"), Some(dir));
    }

    // No config for group → classic projects/ fallback
    #[test]
    fn test_resolve_group_dir_fallback_projects() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let dir = root.join(".aw/tech-design/projects/my-project");
        std::fs::create_dir_all(&dir).unwrap();

        let store = FileSystemSpecStore::new(root.clone(), HashMap::new());
        assert_eq!(store.resolve_group_dir("my-project"), Some(dir));
    }

    // Group not found anywhere → None
    #[test]
    fn test_resolve_group_dir_not_found() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();

        let store = FileSystemSpecStore::new(root, HashMap::new());
        assert_eq!(store.resolve_group_dir("unknown-group"), None);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_store.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete filesystem-backed spec store module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Single struct with PathBuf + HashMap fields.
- [schema] Both fields via x-rust-type.
- [changes] Standard split.
