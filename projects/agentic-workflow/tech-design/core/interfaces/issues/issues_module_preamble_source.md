---
id: sdd-interfaces-issues-module-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# Issues Module Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/issues/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `backend` | projects/agentic-workflow/src/issues/mod.rs | module | pub | 25 |  |
| `backends` | projects/agentic-workflow/src/issues/mod.rs | module | pub | 26 |  |
| `github_backend` | projects/agentic-workflow/src/issues/mod.rs | function | pub | 238 | github_backend() -> GitHubBackend |
| `labels` | projects/agentic-workflow/src/issues/mod.rs | module | pub | 27 |  |
| `local_backend` | projects/agentic-workflow/src/issues/mod.rs | function | pub | 184 | local_backend(project_root: &Path) -> LocalBackend |
| `make_backend` | projects/agentic-workflow/src/issues/mod.rs | function | pub | 57 | make_backend(     kind: &str,     project_root: &Path,     repo: Option<String>,     host: Option<String>, ) -> Result<Box<dyn IssueBackend>> |
| `next_id` | projects/agentic-workflow/src/issues/mod.rs | module | pub | 28 |  |
| `push_through` | projects/agentic-workflow/src/issues/mod.rs | module | pub | 29 |  |
| `remote_read_cache_backend` | projects/agentic-workflow/src/issues/mod.rs | function | pub | 207 | remote_read_cache_backend(     kind: &str,     repo: Option<&str>,     host: Option<&str>, ) -> LocalBackend |
| `remote_read_cache_dir` | projects/agentic-workflow/src/issues/mod.rs | function | pub | 192 | remote_read_cache_dir(kind: &str, repo: Option<&str>, host: Option<&str>) -> PathBuf |
| `resolve_default_backend` | projects/agentic-workflow/src/issues/mod.rs | function | pub | 87 | resolve_default_backend(     project_root: &Path, ) -> Result<(String, Option<String>, Option<String>)> |
| `slug` | projects/agentic-workflow/src/issues/mod.rs | module | pub | 30 |  |
| `types` | projects/agentic-workflow/src/issues/mod.rs | module | pub | 31 |  |
## Source
<!-- type: source lang: rust -->

```rust
//! Issue artifact store — uniform interface over local files, GitHub, GitLab, Jira.
//!
//! # Architecture
//!
//! - [`Issue`] / [`IssueType`] / [`IssueState`] / [`IssueFilter`] — wire
//!   format (also the local issue `{open,closed}/*.md` frontmatter schema)
//! - [`IssueBackend`] — storage trait implemented by each backend
//! - [`backends::LocalBackend`] — reads/writes issue files under a chosen root
//! - [`backends::GitHubBackend`] — shells out to `gh` CLI (read-only MVP)
//! - [`remote_read_cache_backend`] — ephemeral `/tmp` cache for remote reads
//! - [`make_backend`] — factory that picks a backend from resolved kind + repo + host
//! - [`resolve_default_backend`] — read `.aw/config.toml` and return the
//!   `(kind, repo, host)` triple to feed into `make_backend`.
//!
//! # Agent usage
//!
//! All verbs are exposed via the `aw wi` CLI subcommand with a
//! `--json` flag for machine-parseable output. Agents should invoke the
//! CLI rather than linking this module directly.
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
    description: "Source template owns the issues module documentation preamble."
```
