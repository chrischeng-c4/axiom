// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
// CODEGEN-BEGIN
//! `jet test --list` manifest (#2866).
//!
//! Emits the **discovered** test surface — without executing test
//! bodies — as a stable, machine-readable manifest. Two consumer
//! groups care:
//!
//! * **Agents** that want to know what would run before paying for a
//!   real run (cost gating, change-impact analysis).
//! * **CI sharders** that need a deterministic list to partition over
//!   workers without re-walking the project tree.
//!
//! The shape is intentionally narrower than [`ResolvedDiscovery`]: it
//! drops runner-execution knobs (workers, shard, timeout) and keeps
//! only what an agent needs to identify each test. Invalid discovery
//! still flows through [`crate::test_runner::discovery::DiscoveryConfigError`]
//! so CLI/error paths stay machine-readable per `#2709`.
//!
//! @spec #2866

use crate::test_runner::config::RunnerConfig;
use crate::test_runner::discovery::{
    resolve_discovery, DiscoveryConfigError, ResolvedDiscovery, ResolvedSpec,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Schema version for the list manifest. Bumped only when the JSON
/// shape changes in a way older consumers cannot ignore.
// @spec #2866
pub const LIST_MANIFEST_SCHEMA_VERSION: u32 = 1;

/// One discovered test file as the agent will see it. The `id` is the
/// stable identifier used by reporters and the `jet test -g` rerun
/// hint surface (#2871); right now it equals the relative path.
// @spec #2866
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListedTest {
    pub id: String,
    pub file: PathBuf,
}

/// Discovered test surface, narrowed for the `--list` consumer.
///
/// Drops runner-execution knobs (workers, shard, timeout) so the
/// manifest is independent of how a future run would be scheduled —
/// the same project + same matchers always produce the same manifest.
// @spec #2866
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestListManifest {
    pub schema_version: u32,
    pub project_root: PathBuf,
    pub environment: String,
    pub test_match: Vec<String>,
    pub test_ignore: Vec<String>,
    pub tests: Vec<ListedTest>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl TestListManifest {
    /// Project a [`ResolvedDiscovery`] into the list-manifest shape.
    /// No file IO and no execution — purely a deterministic projection.
    // @spec #2866
    pub fn from_discovery(d: &ResolvedDiscovery) -> Self {
        Self {
            schema_version: LIST_MANIFEST_SCHEMA_VERSION,
            project_root: d.project_root.clone(),
            environment: d.environment.clone(),
            test_match: d.test_match.clone(),
            test_ignore: d.test_ignore.clone(),
            tests: d
                .specs
                .iter()
                .filter_map(|spec| match listed_test(spec) {
                    Some(t) => Some(t),
                    None => {
                        tracing::warn!(
                            target: "jet::test_runner::list_manifest",
                            "{}",
                            format_list_manifest_non_utf8_warn(&spec.relative)
                        );
                        None
                    }
                })
                .collect(),
        }
    }

    /// Resolve discovery from a runner config and project into the
    /// list manifest in one step. Discovery errors propagate as the
    /// machine-readable [`DiscoveryConfigError`] from #2709.
    // @spec #2866
    pub fn from_config(config: &RunnerConfig) -> std::result::Result<Self, DiscoveryConfigError> {
        let discovery = resolve_discovery(config)?;
        Ok(Self::from_discovery(&discovery))
    }

    /// Render the manifest as a stable JSON string. Pretty-printed so
    /// it diffs cleanly in CI artifacts and PR comments.
    // @spec #2866
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("TestListManifest is always serialisable")
    }
}

fn listed_test(spec: &ResolvedSpec) -> Option<ListedTest> {
    // GH #3755 — refuse to lossy-encode the spec ID. A relative path
    // with non-UTF-8 bytes would otherwise get U+FFFD-substituted into
    // the ID, silently collapsing two distinct specs into one and
    // breaking the round-trip from manifest id back into `--only-files`.
    let id = spec.relative.to_str()?.to_string();
    Some(ListedTest {
        id,
        file: spec.relative.clone(),
    })
}

/// Format the `tracing::warn!` payload emitted when a spec's relative
/// path cannot be losslessly represented as UTF-8 and is therefore
/// skipped from the list manifest. The "GH #3755" tag lets operators
/// grep the warning back to the originating issue.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn format_list_manifest_non_utf8_warn(rel: &Path) -> String {
    format!(
        "GH #3755: skipping spec from list manifest because its relative path is not valid UTF-8 \
         (refusing silent U+FFFD substitution into the test ID); rel={:?}",
        rel
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_runner::config::{RunnerConfig, TestEnvironment};
    use std::fs;
    use tempfile::TempDir;

    fn sample_project() -> (TempDir, RunnerConfig) {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_path_buf();
        let test_dir = root.join("tests");
        fs::create_dir_all(&test_dir).unwrap();
        // Two real spec files + one ignored helper.
        fs::write(test_dir.join("a.spec.ts"), b"export const _ = 0;\n").unwrap();
        fs::write(test_dir.join("b.spec.ts"), b"export const _ = 0;\n").unwrap();
        fs::write(test_dir.join("helper.ts"), b"export const _ = 0;\n").unwrap();

        // Start from `default_for_root` so we exercise the same
        // resolver path real `jet test` uses, then narrow the
        // matchers + test_dir for a deterministic fixture.
        let mut cfg = RunnerConfig::default_for_root(&root).unwrap();
        cfg.test_dir = cfg.project_root.join("tests");
        cfg.test_match = vec!["**/*.spec.ts".into()];
        cfg.test_ignore = vec![];
        cfg.environment = TestEnvironment::Node;
        cfg.workers = 4;
        (tmp, cfg)
    }

    #[test]
    fn sample_project_produces_a_stable_list_manifest() {
        // @spec #2866 stop condition — same inputs must produce the
        // same manifest bytes across runs (deterministic order +
        // stable shape).
        let (_tmp, cfg) = sample_project();
        let m1 = TestListManifest::from_config(&cfg).unwrap();
        let m2 = TestListManifest::from_config(&cfg).unwrap();
        assert_eq!(m1, m2, "manifest must be deterministic");

        // File-level discovery surfaces both spec files, ignores the helper.
        let ids: Vec<&str> = m1.tests.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.iter().any(|id| id.ends_with("a.spec.ts")), "{ids:?}");
        assert!(ids.iter().any(|id| id.ends_with("b.spec.ts")), "{ids:?}");
        assert!(!ids.iter().any(|id| id.contains("helper")), "{ids:?}");

        // The list manifest does NOT carry execution knobs — workers
        // changed in the config must not affect the manifest bytes.
        let mut cfg2 = cfg.clone();
        cfg2.workers = 1;
        let m3 = TestListManifest::from_config(&cfg2).unwrap();
        assert_eq!(
            m1.to_json(),
            m3.to_json(),
            "workers must not perturb manifest"
        );
    }

    #[test]
    fn list_manifest_round_trips_through_json() {
        let (_tmp, cfg) = sample_project();
        let manifest = TestListManifest::from_config(&cfg).unwrap();
        let json = manifest.to_json();
        let back: TestListManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, back);
        // Schema version is present so older readers can refuse loudly.
        assert!(json.contains(r#""schema_version": 1"#), "{json}");
    }

    #[test]
    fn invalid_discovery_returns_machine_readable_error() {
        // @spec #2866 AC — invalid discovery errors stay machine-readable.
        // We exercise the same DiscoveryConfigError surface from #2709
        // so callers can parse failures off `code` without scraping a
        // human message.
        let tmp = TempDir::new().unwrap();
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.test_dir = cfg.project_root.join("does/not/exist");
        let err = TestListManifest::from_config(&cfg).unwrap_err();
        assert_eq!(err.code, "invalid_test_dir");
        let json = err.to_json();
        assert!(json.contains(r#""code":"invalid_test_dir""#), "{json}");
        assert!(
            json.contains(r#""kind":"discovery_config_error""#),
            "{json}"
        );
    }

    #[test]
    fn list_manifest_drops_runner_execution_knobs() {
        // The manifest must NOT carry timeout/workers/shard/only_files —
        // those are run-time knobs, not discovery facts.
        let (_tmp, cfg) = sample_project();
        let json = TestListManifest::from_config(&cfg).unwrap().to_json();
        for omitted in ["workers", "timeout_ms", "shard", "only_files"] {
            assert!(
                !json.contains(omitted),
                "manifest must not contain runtime knob {omitted}: {json}",
            );
        }
    }
}

/// GH #3755 — silent lossy UTF-8 substitution in list-manifest test IDs.
///
/// The fix: `listed_test` returns `Option<ListedTest>` and `from_discovery`
/// emits a `tracing::warn!` (carrying the "GH #3755" tag) for any spec
/// whose relative path is not valid UTF-8, rather than U+FFFD-substituting
/// the ID. These tests cover the helper contract; the integration path
/// is covered by the existing manifest tests above, which exercise the
/// happy path through `from_config`.
#[cfg(test)]
mod gh3755_non_utf8_id_warn_tests {
    use super::*;
    use crate::test_runner::discovery::ResolvedSpec;
    use std::path::PathBuf;

    fn good_spec() -> ResolvedSpec {
        ResolvedSpec {
            path: PathBuf::from("/proj/tests/a.spec.ts"),
            relative: PathBuf::from("tests/a.spec.ts"),
        }
    }

    #[test]
    fn gh3755_warn_message_contains_issue_tag_and_path() {
        let rel = PathBuf::from("tests/weird.spec.ts");
        let msg = format_list_manifest_non_utf8_warn(&rel);
        assert!(msg.contains("GH #3755"), "issue tag missing: {msg}");
        assert!(msg.contains("not valid UTF-8"), "shape missing: {msg}");
        assert!(msg.contains("rel="), "rel kv missing: {msg}");
    }

    #[test]
    fn gh3755_warn_message_is_deterministic() {
        let rel = PathBuf::from("tests/a.spec.ts");
        let a = format_list_manifest_non_utf8_warn(&rel);
        let b = format_list_manifest_non_utf8_warn(&rel);
        assert_eq!(a, b);
    }

    #[test]
    fn gh3755_different_paths_produce_distinct_messages() {
        let a = format_list_manifest_non_utf8_warn(&PathBuf::from("tests/a"));
        let b = format_list_manifest_non_utf8_warn(&PathBuf::from("tests/b"));
        assert_ne!(a, b);
    }

    #[test]
    fn gh3755_helper_name_follows_family_convention() {
        // Sibling helpers in this crate use the `format_<area>_<thing>_warn`
        // shape. Discoverability test: callers searching for "format_list_manifest"
        // should find our helper in this module.
        let msg = format_list_manifest_non_utf8_warn(&PathBuf::from("x"));
        assert!(!msg.is_empty());
    }

    #[test]
    fn gh3755_warn_is_distinct_from_sibling_hash_warn() {
        // The hash-input non-UTF-8 helper from #3753 emits a similar
        // message; ensure operators grepping "GH #3755" don't confuse
        // the two by including different anchor text.
        let msg = format_list_manifest_non_utf8_warn(&PathBuf::from("x.spec.ts"));
        assert!(msg.contains("list manifest"), "missing area anchor: {msg}");
        assert!(!msg.contains("GH #3753"), "must not mention sibling: {msg}");
    }

    #[test]
    fn gh3755_listed_test_some_for_valid_utf8() {
        let spec = good_spec();
        let listed = listed_test(&spec).expect("valid UTF-8 path must produce Some");
        assert_eq!(listed.id, "tests/a.spec.ts");
        assert_eq!(listed.file, spec.relative);
    }

    #[cfg(unix)]
    #[test]
    fn gh3755_listed_test_none_for_non_utf8() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let bad = PathBuf::from(OsStr::from_bytes(&[
            b't', b'/', b'a', 0xFF, b'.', b't', b's',
        ]));
        let spec = ResolvedSpec {
            path: PathBuf::from("/proj").join(&bad),
            relative: bad,
        };
        assert!(
            listed_test(&spec).is_none(),
            "non-UTF-8 relative must produce None"
        );
    }

    #[test]
    fn gh3755_from_discovery_skips_non_utf8_and_keeps_valid() {
        // We can't build a non-UTF-8 PathBuf inline cross-platform via
        // `from_discovery` (we'd need a real filesystem), so this case
        // exercises the happy path: from_discovery must keep every
        // valid-UTF-8 spec and produce one ListedTest per input.
        use crate::test_runner::discovery::ResolvedDiscovery;

        let discovery = ResolvedDiscovery {
            schema_version: 1,
            project_root: PathBuf::from("/proj"),
            environment: "node".into(),
            test_match: vec!["**/*.spec.ts".into()],
            test_ignore: vec![],
            test_dir: PathBuf::from("/proj/tests"),
            workers: 1,
            timeout_ms: 1000,
            shard: None,
            only_files: vec![],
            specs: vec![
                good_spec(),
                ResolvedSpec {
                    path: PathBuf::from("/proj/tests/b.spec.ts"),
                    relative: PathBuf::from("tests/b.spec.ts"),
                },
            ],
        };
        let m = TestListManifest::from_discovery(&discovery);
        assert_eq!(m.tests.len(), 2);
        assert_eq!(m.tests[0].id, "tests/a.spec.ts");
        assert_eq!(m.tests[1].id, "tests/b.spec.ts");
    }

    #[cfg(unix)]
    #[test]
    fn gh3755_from_discovery_filters_non_utf8_specs() {
        use crate::test_runner::discovery::ResolvedDiscovery;
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let bad_rel = PathBuf::from(OsStr::from_bytes(&[
            b't', b'/', b'x', 0xFF, b'.', b't', b's',
        ]));
        let discovery = ResolvedDiscovery {
            schema_version: 1,
            project_root: PathBuf::from("/proj"),
            environment: "node".into(),
            test_match: vec!["**/*.spec.ts".into()],
            test_ignore: vec![],
            test_dir: PathBuf::from("/proj/tests"),
            workers: 1,
            timeout_ms: 1000,
            shard: None,
            only_files: vec![],
            specs: vec![
                good_spec(),
                ResolvedSpec {
                    path: PathBuf::from("/proj").join(&bad_rel),
                    relative: bad_rel,
                },
            ],
        };
        let m = TestListManifest::from_discovery(&discovery);
        // Only the valid-UTF-8 spec survives.
        assert_eq!(m.tests.len(), 1, "{:?}", m.tests);
        assert_eq!(m.tests[0].id, "tests/a.spec.ts");
    }

    #[test]
    fn gh3755_warn_messages_contain_distinct_paths_for_distinct_specs() {
        let a = format_list_manifest_non_utf8_warn(&PathBuf::from("tests/x.spec.ts"));
        let b = format_list_manifest_non_utf8_warn(&PathBuf::from("tests/y.spec.ts"));
        assert!(a.contains("x.spec.ts"));
        assert!(b.contains("y.spec.ts"));
    }
}
// CODEGEN-END
