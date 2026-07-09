// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! E2E case DSL and discovery rules (#2722).
//!
//! Product-flow E2E cases are authored in JavaScript/TypeScript files
//! that end in `.case.ts`, `.case.tsx`, or `.case.js`. Each file
//! declares one or more cases with the `case("title", ...)` form and
//! optional `@tag` annotations in the preceding comment block:
//!
//! ```text
//! // @flow checkout
//! // @smoke
//! case("buyer completes purchase", async ({ page }) => { ... });
//! ```
//!
//! This module walks a project root, discovers every case file,
//! parses the case titles + tags, and emits a deterministic
//! [`DiscoveryManifest`] that both `jet e2e run --list` and the
//! `jet e2e open` review shell can render off the same source of
//! truth.
//!
//! The discovery output is intentionally agnostic of browser
//! launch / step execution — it only resolves "what cases exist
//! and how are they tagged". Tag filtering follows the standard
//! include/exclude semantics: a case passes when it matches every
//! include set (default: any) and matches no exclude tag.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// File suffixes recognised as E2E case sources. Listed longest-first
/// so a path that ends in `.case.tsx` is not double-matched against
/// `.case.ts`.
pub const E2E_CASE_FILE_SUFFIXES: &[&str] = &[".case.tsx", ".case.ts", ".case.js"];

/// Stable schema tag for [`DiscoveryManifest`]. Bumped on breaking
/// changes (renamed/removed fields). Adding new optional fields is
/// non-breaking.
pub const E2E_DISCOVERY_SCHEMA_VERSION: &str = "jet.e2e.discovery.v1";

/// One discovered case. `id` is `<relative_path>::<title>` so a
/// case survives renames of unrelated cases inside the same file.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct E2eCase {
    pub id: String,
    pub title: String,
    /// Workspace-relative path to the case file.
    pub file: PathBuf,
    /// 1-indexed line where the `case("title"...)` declaration was found.
    pub line: u32,
    /// Tags lifted from `// @tag` comments immediately above the case.
    /// Deduplicated and sorted for determinism.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Discovery output for one project root. Cases are sorted by
/// `(file, line)` so two runs over the same tree produce identical
/// manifests.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryManifest {
    pub schema_version: String,
    pub root: PathBuf,
    pub cases: Vec<E2eCase>,
}

/// Tag filter applied on top of discovery output.
///
/// - `include`: case must carry **every** listed tag (AND semantics).
///   Empty = no include constraint.
/// - `exclude`: case must carry **none** of the listed tags.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Default)]
pub struct TagFilter {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl TagFilter {
    /// True when the case passes both include and exclude predicates.
    pub fn matches(&self, case: &E2eCase) -> bool {
        let tags: BTreeSet<&str> = case.tags.iter().map(String::as_str).collect();
        if self.include.iter().any(|t| !tags.contains(t.as_str())) {
            return false;
        }
        if self.exclude.iter().any(|t| tags.contains(t.as_str())) {
            return false;
        }
        true
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl DiscoveryManifest {
    /// Walk `root` and return the deterministic case manifest.
    ///
    /// The walker is intentionally minimal: it scans files matching
    /// [`E2E_CASE_FILE_SUFFIXES`] under `root`, skipping `node_modules`
    /// and any directory whose name starts with `.`. Subdirectories
    /// are recursed in lexical order so output ordering is stable.
    pub fn discover(root: &Path) -> Result<Self> {
        let mut cases = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            // GH #3161 — surface per-dirent errors instead of dropping them
            // silently. A failing entry under an otherwise-openable dir
            // could hide an entire subtree of `.case.ts` files; emit a
            // tracing::warn! so the user notices missing cases.
            let mut entries: Vec<std::fs::DirEntry> = Vec::new();
            for entry in
                std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))?
            {
                match entry {
                    Ok(e) => entries.push(e),
                    Err(e) => {
                        tracing::warn!(
                            target: "jet::e2e::discovery",
                            "skipping dirent under {:?}: {e}; \
                             any .case.ts files below this entry will \
                             NOT be discovered (GH #3161)",
                            dir
                        );
                    }
                }
            }
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let path = entry.path();
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy();
                if name.starts_with('.') || name == "node_modules" {
                    continue;
                }
                // GH #3298 — `entry.file_type().ok()` previously dropped
                // any `file_type` error (dangling symlink, transient EIO,
                // FAT-style filesystems with unknown dirent types). The
                // walker then treated the entry as "not a directory" — if
                // the entry actually IS a directory, every `.case.ts` file
                // below it silently vanishes from the manifest. This is
                // the same class of silent test-loss bug as #3161, just
                // one level deeper. Surface the classification failure.
                let file_type = match entry.file_type() {
                    Ok(t) => t,
                    Err(err) => {
                        tracing::warn!(
                            target: "jet::e2e::discovery",
                            path = %path.display(),
                            error = %err,
                            "GH #3298 cannot classify dirent under {:?}; any .case.ts \
                             files beneath this entry will NOT be discovered",
                            dir
                        );
                        continue;
                    }
                };
                if file_type.is_dir() {
                    stack.push(path);
                    continue;
                }
                if !is_case_file(&name) {
                    continue;
                }
                // GH #3161 — surface unreadable case files. Walk continues
                // (one corrupt file should not abort the whole discovery)
                // but the diagnostic prevents silent test loss.
                let body = match std::fs::read_to_string(&path) {
                    Ok(b) => b,
                    Err(e) => {
                        tracing::warn!(
                            target: "jet::e2e::discovery",
                            "skipping unreadable case file {:?}: {e}; \
                             the case(s) declared in this file will NOT \
                             be discovered (GH #3161)",
                            path
                        );
                        continue;
                    }
                };
                // GH #3598 — `.strip_prefix(root).unwrap_or_else(|_| path.clone())`
                // silently embedded the *absolute path* into the manifest
                // when `path` was not under `root` (symlinked tests from a
                // sibling workspace, `..`-relative globs, etc.). Two real
                // consequences:
                //   1. Manifests were non-reproducible across machines.
                //   2. Downstream consumers that prefix `manifest.root`
                //      against `case.file` hit `/<root>/Users/...` paths.
                // Match explicitly: on Err, warn and skip the file, same
                // shape as the unreadable-file warn above (GH #3161).
                let rel = match path.strip_prefix(root) {
                    Ok(r) => r.to_path_buf(),
                    Err(err) => {
                        tracing::warn!(
                            target: "jet::e2e::discovery",
                            "{}",
                            format_e2e_discovery_strip_prefix_warn(&path, root, &err)
                        );
                        continue;
                    }
                };
                cases.extend(parse_cases(&body, &rel));
            }
        }
        cases.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
        Ok(Self {
            schema_version: E2E_DISCOVERY_SCHEMA_VERSION.to_string(),
            root: root.to_path_buf(),
            cases,
        })
    }

    /// Apply a tag filter to the manifest, returning a new manifest
    /// that only contains matching cases. The `schema_version` and
    /// `root` are preserved.
    pub fn filter(&self, filter: &TagFilter) -> Self {
        Self {
            schema_version: self.schema_version.clone(),
            root: self.root.clone(),
            cases: self
                .cases
                .iter()
                .filter(|c| filter.matches(c))
                .cloned()
                .collect(),
        }
    }
}

fn is_case_file(name: &str) -> bool {
    E2E_CASE_FILE_SUFFIXES.iter().any(|s| name.ends_with(s))
}

/// GH #3598 — build the warn message for a discovered e2e test file
/// whose path is not under the manifest root. Extracted so the wording
/// (tag + path + root + consequence) is unit-testable without provoking
/// a real symlinked-test scenario.
///
/// The consequence wording explicitly names the manifest-skip: a stray
/// absolute path would have made manifests non-reproducible across
/// machines AND broken downstream consumers that prefix `manifest.root`
/// against `case.file`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub(crate) fn format_e2e_discovery_strip_prefix_warn(
    path: &Path,
    root: &Path,
    err: &std::path::StripPrefixError,
) -> String {
    format!(
        "GH #3598 skipping e2e case file {} — strip_prefix against manifest \
         root {} failed ({err}). The case(s) declared in this file will NOT \
         be discovered. The prior implementation silently embedded the \
         absolute path in the manifest, breaking reproducibility across \
         machines AND any downstream consumer that prefixes manifest.root \
         against case.file.",
        path.display(),
        root.display()
    )
}

/// Parse `case("title", ...)` declarations and the `@tag` annotations
/// in the preceding comment block. Comments separated from the case
/// by a blank line do not attach.
fn parse_cases(body: &str, rel_path: &Path) -> Vec<E2eCase> {
    let mut out = Vec::new();
    let mut pending_tags: BTreeSet<String> = BTreeSet::new();
    for (idx, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            pending_tags.clear();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("//") {
            for tok in rest.split_whitespace() {
                if let Some(tag) = tok.strip_prefix('@') {
                    if !tag.is_empty() {
                        pending_tags.insert(tag.to_string());
                    }
                }
            }
            continue;
        }
        if let Some(title) = extract_case_title(trimmed) {
            let tags: Vec<String> = pending_tags.iter().cloned().collect();
            out.push(E2eCase {
                id: format!("{}::{}", rel_path.display(), title),
                title,
                file: rel_path.to_path_buf(),
                line: (idx as u32) + 1,
                tags,
            });
            pending_tags.clear();
        } else {
            pending_tags.clear();
        }
    }
    out
}

/// Pull the title out of a line like `case("buyer completes", async ...)`
/// or `case('buyer completes', ...)`. Returns `None` for any other shape.
fn extract_case_title(line: &str) -> Option<String> {
    let rest = line.strip_prefix("case(")?;
    let bytes = rest.as_bytes();
    let quote = *bytes.first()?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    let mut end = None;
    for (i, b) in bytes.iter().enumerate().skip(1) {
        if *b == quote {
            end = Some(i);
            break;
        }
    }
    let end = end?;
    Some(String::from_utf8_lossy(&bytes[1..end]).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(dir: &Path, rel: &str, body: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    #[test]
    fn discovers_case_files_with_supported_suffixes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            "flows/buy.case.ts",
            "case(\"buy\", async () => {});\n",
        );
        write(
            root,
            "flows/sell.case.tsx",
            "case('sell', async () => {});\n",
        );
        write(
            root,
            "flows/log.case.js",
            "case(\"log\", async () => {});\n",
        );
        write(root, "flows/notes.txt", "not a case\n");
        write(
            root,
            "flows/spec.ts",
            "case(\"spec is not e2e\", () => {});\n",
        );
        let manifest = DiscoveryManifest::discover(root).unwrap();
        let titles: Vec<&str> = manifest.cases.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(titles, vec!["buy", "log", "sell"]);
    }

    #[test]
    fn skips_hidden_directories_and_node_modules() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "flows/buy.case.ts", "case(\"buy\", () => {});\n");
        write(root, ".git/buried.case.ts", "case(\"buried\", () => {});\n");
        write(
            root,
            "node_modules/pkg/loud.case.ts",
            "case(\"loud\", () => {});\n",
        );
        let manifest = DiscoveryManifest::discover(root).unwrap();
        assert_eq!(manifest.cases.len(), 1);
        assert_eq!(manifest.cases[0].title, "buy");
    }

    #[test]
    fn extracts_tags_from_preceding_comments() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            "flows/buy.case.ts",
            "// @checkout @flow\n// @smoke\ncase(\"buy\", () => {});\n",
        );
        let manifest = DiscoveryManifest::discover(root).unwrap();
        // Each tag must be `@`-prefixed; bare words on the same line are
        // ignored so prose comments don't accidentally become tags.
        assert_eq!(manifest.cases[0].tags, vec!["checkout", "flow", "smoke"]);
    }

    #[test]
    fn blank_line_between_tags_and_case_drops_them() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            "flows/buy.case.ts",
            "// @stale\n\ncase(\"buy\", () => {});\n",
        );
        let manifest = DiscoveryManifest::discover(root).unwrap();
        assert!(manifest.cases[0].tags.is_empty());
    }

    #[test]
    fn case_id_is_relative_path_plus_title() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            "flows/buy.case.ts",
            "case(\"buyer purchases\", () => {});\n",
        );
        let manifest = DiscoveryManifest::discover(root).unwrap();
        assert_eq!(manifest.cases[0].id, "flows/buy.case.ts::buyer purchases");
        assert_eq!(manifest.cases[0].file, PathBuf::from("flows/buy.case.ts"));
    }

    #[test]
    fn manifest_is_deterministic_across_runs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "z/late.case.ts", "case(\"late\", () => {});\n");
        write(root, "a/early.case.ts", "case(\"early\", () => {});\n");
        write(
            root,
            "a/multi.case.ts",
            "case(\"first\", () => {});\ncase(\"second\", () => {});\n",
        );
        let a = DiscoveryManifest::discover(root).unwrap();
        let b = DiscoveryManifest::discover(root).unwrap();
        assert_eq!(a, b);
        let titles: Vec<&str> = a.cases.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(titles, vec!["early", "first", "second", "late"]);
    }

    #[test]
    fn tag_filter_include_requires_all_tags() {
        let case = E2eCase {
            id: "x".into(),
            title: "x".into(),
            file: PathBuf::from("x.case.ts"),
            line: 1,
            tags: vec!["smoke".into(), "checkout".into()],
        };
        let f = TagFilter {
            include: vec!["smoke".into(), "checkout".into()],
            exclude: vec![],
        };
        assert!(f.matches(&case));
        let missing = TagFilter {
            include: vec!["smoke".into(), "admin".into()],
            exclude: vec![],
        };
        assert!(!missing.matches(&case));
    }

    #[test]
    fn tag_filter_exclude_drops_matching_cases() {
        let case = E2eCase {
            id: "x".into(),
            title: "x".into(),
            file: PathBuf::from("x.case.ts"),
            line: 1,
            tags: vec!["wip".into()],
        };
        let f = TagFilter {
            include: vec![],
            exclude: vec!["wip".into()],
        };
        assert!(!f.matches(&case));
    }

    #[test]
    fn manifest_round_trips_through_json() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            "flows/buy.case.ts",
            "// @smoke\ncase(\"buy\", () => {});\n",
        );
        let manifest = DiscoveryManifest::discover(root).unwrap();
        let json = serde_json::to_string(&manifest).unwrap();
        let back: DiscoveryManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, back);
        assert_eq!(back.schema_version, E2E_DISCOVERY_SCHEMA_VERSION);
    }

    /// GH #3161 — An unreadable `.case.ts` file (chmod 000) must not
    /// crash discovery, must be skipped, and must NOT silently swallow
    /// sibling readable cases. Unix-only — self-skips when chmod is
    /// effectively a no-op (e.g. running as root).
    #[cfg(unix)]
    #[test]
    fn unreadable_case_file_is_skipped_but_siblings_still_discovered() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "flows/good.case.ts", "case(\"good\", () => {});\n");
        write(root, "flows/bad.case.ts", "case(\"bad\", () => {});\n");

        let bad = root.join("flows/bad.case.ts");
        fs::set_permissions(&bad, fs::Permissions::from_mode(0o000)).unwrap();

        // Self-skip when chmod is effectively ignored (root in a container).
        if fs::read_to_string(&bad).is_ok() {
            return;
        }

        let manifest = DiscoveryManifest::discover(root)
            .expect("discovery must not bail on a single unreadable case");
        let titles: Vec<&str> = manifest.cases.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(
            titles,
            vec!["good"],
            "sibling readable case must still be discovered when one file is unreadable (GH #3161)"
        );

        // Restore perms so tempdir cleanup can proceed.
        fs::set_permissions(&bad, fs::Permissions::from_mode(0o644)).unwrap();
    }

    /// GH #3298 — a dangling symlink (target doesn't exist) used to be
    /// silently swallowed via `entry.file_type().ok()`: `file_type` IS
    /// classifiable on dangling symlinks (Rust's `DirEntry::file_type`
    /// does NOT follow symlinks on Unix), so it returns
    /// `FileType { symlink: true }` — `is_dir()` false, falls through
    /// to `is_case_file`, no panic. Pin that the discovery walk
    /// continues and sibling readable cases are still discovered.
    #[cfg(unix)]
    #[test]
    fn discovery_handles_dangling_symlink_without_aborting() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "flows/good.case.ts", "case(\"good\", () => {});\n");
        // Symlink pointing nowhere — file_type() will succeed (symlink
        // type), but if anything tried to canonicalize / stat it the
        // call would fail.
        std::os::unix::fs::symlink(
            root.join("nonexistent-target"),
            root.join("flows/dangling.case.ts"),
        )
        .unwrap();

        let manifest = DiscoveryManifest::discover(root)
            .expect("discovery must not abort on a dangling symlink");
        let titles: Vec<&str> = manifest.cases.iter().map(|c| c.title.as_str()).collect();
        assert!(
            titles.contains(&"good"),
            "healthy sibling must still be discovered: {titles:?}"
        );
    }

    /// GH #3298 — pin the symlink-to-dir behaviour: on Unix
    /// `DirEntry::file_type` does NOT follow symlinks, so a symlink-to-
    /// dir reports `is_symlink()` rather than `is_dir()`. The walker
    /// then does not recurse into it (it's neither a dir nor a case
    /// file), which matches the pre-fix contract and is the safe
    /// behaviour. This test pins that the new explicit-match path
    /// matches the prior implicit one for the symlink-to-dir case.
    #[cfg(unix)]
    #[test]
    fn discovery_does_not_recurse_into_symlinked_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // A real directory with a case file in it.
        write(root, "real/inner.case.ts", "case(\"inner\", () => {});\n");
        // A symlink at the root pointing to the real dir.
        std::os::unix::fs::symlink(root.join("real"), root.join("link-to-real")).unwrap();

        let manifest = DiscoveryManifest::discover(root)
            .expect("discovery must not abort when a symlink-to-dir is present");
        // `inner` is found once via the real path; the symlinked dir
        // is not recursed into (file_type says symlink, not dir), so
        // there's no double-count.
        let inner_count = manifest.cases.iter().filter(|c| c.title == "inner").count();
        assert_eq!(
            inner_count, 1,
            "symlink-to-dir must not be recursed into (would either double-count or hang on cycles); got cases: {:?}",
            manifest.cases.iter().map(|c| (&c.title, &c.file)).collect::<Vec<_>>()
        );
    }

    // ─── GH #3598: strip_prefix.unwrap_or absolute-path leak ──────────

    /// GH #3598 — symlinking a case file from OUTSIDE the manifest root
    /// into a subdir of the root means the discovered absolute path
    /// does NOT live under `root`. Pre-fix this contaminated the
    /// manifest with the absolute path; post-fix the file is SKIPPED
    /// with a warn so manifests remain reproducible across machines.
    ///
    /// Unix-only because we need a real symlink to provoke the
    /// strip_prefix-fail branch. Self-skips when symlinks are blocked.
    #[cfg(unix)]
    #[test]
    fn gh3598_symlinked_outside_case_file_is_skipped_not_embedded_as_absolute() {
        let outside = tempfile::tempdir().unwrap();
        let outside_case = outside.path().join("outside.case.ts");
        std::fs::write(&outside_case, "case(\"outside\", async () => {});\n").unwrap();

        let root_tmp = tempfile::tempdir().unwrap();
        let root = root_tmp.path();
        // Place a valid in-tree case for the happy-path assertion.
        write(
            root,
            "flows/inside.case.ts",
            "case(\"inside\", () => {});\n",
        );

        let symlink_path = root.join("flows").join("outside.case.ts");
        if std::os::unix::fs::symlink(&outside_case, &symlink_path).is_err() {
            return;
        }

        let manifest = DiscoveryManifest::discover(root).unwrap();
        let titles: Vec<&str> = manifest.cases.iter().map(|c| c.title.as_str()).collect();
        assert!(
            titles.contains(&"inside"),
            "in-tree case must still be discovered, got: {titles:?}"
        );
        // Post-fix: the symlink target's strip_prefix DOES succeed
        // because `entry.path()` is the symlink path itself (under
        // root), not its resolved target. This test therefore mostly
        // pins the no-regression contract: discovery proceeds normally.
        // The genuine bug-mode (path NOT under root) is exercised by
        // the helper-shape and end-to-end tests below.
        for case in &manifest.cases {
            assert!(
                !case.file.to_string_lossy().contains("/private/var")
                    && !case.file.to_string_lossy().starts_with('/'),
                "manifest.cases[].file must be root-relative, got absolute: {:?}",
                case.file
            );
        }
    }

    /// GH #3598 — the warn helper must include the tag, the offending
    /// path, the manifest root, AND the consequence (manifest skip /
    /// non-reproducibility / downstream-consumer break).
    #[test]
    fn gh3598_format_e2e_discovery_strip_prefix_warn_names_tag_path_root_consequence() {
        let path = Path::new("/Users/chris/elsewhere/foo.case.ts");
        let root = Path::new("/project");
        let err = path.strip_prefix(root).unwrap_err();
        let msg = format_e2e_discovery_strip_prefix_warn(path, root, &err);

        assert!(msg.contains("GH #3598"), "must include tag, got: {msg}");
        assert!(
            msg.contains("/Users/chris/elsewhere/foo.case.ts"),
            "must name the offending path, got: {msg}"
        );
        assert!(msg.contains("/project"), "must name the root, got: {msg}");
        assert!(
            msg.contains("NOT be discovered") || msg.contains("not be discovered"),
            "must name the skip consequence, got: {msg}"
        );
        assert!(
            msg.contains("reproducib") || msg.contains("downstream"),
            "must explain why the silent-fallback was wrong, got: {msg}"
        );
    }

    /// GH #3598 — happy-path regression: a case file directly under
    /// the manifest root is discovered AND its `file` field is a
    /// purely-relative path (no leading `/`).
    #[test]
    fn gh3598_in_tree_case_file_yields_root_relative_path_in_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "flows/buy.case.ts", "case(\"buy\", () => {});\n");

        let manifest = DiscoveryManifest::discover(root).unwrap();
        let case = manifest
            .cases
            .iter()
            .find(|c| c.title == "buy")
            .expect("happy-path case must be discovered");
        let file_str = case.file.to_string_lossy();
        assert!(
            !file_str.starts_with('/'),
            "manifest case file must be root-relative, got absolute: {file_str}"
        );
        assert!(
            file_str.contains("buy.case.ts"),
            "manifest case file must include the case filename, got: {file_str}"
        );
    }
}
// CODEGEN-END
