// End-to-end uv-parity smoke matrix (Tick 30).
//
// Goal of this module: a single in-process matrix that exercises every
// uv-parity surface mamba's pkg-mgmt now exposes, so a regression in
// any one of them shows up as a focused failed parity case rather than
// surfacing far downstream as a confusing resolver / installer crash.
//
// What this is:
//   * A `ParityMatrix` of `ParityCheck`s. Each check is a closure that
//     returns `ParityOutcome::{Pass, Fail{detail}, Skipped{reason}}`.
//   * A reporter that runs every case and aggregates a `ParityReport`
//     with pass/fail/skip counts and a human-readable summary.
//   * A `default_matrix()` that wires up one smoke case per pkg-mgmt
//     module shipped in Ticks 20–29, exercising real fixtures and
//     asserting the canonical outputs. No network, no subprocess.
//
// What this is NOT:
//   * A reimplementation of uv. The "uv parity" baseline here is the
//     spec-derived behaviors we claim to match (PEP 503 / 508 / 517 /
//     631 / 660 / 723 / 735 + uv's `[tool.uv.*]` schema), checked
//     against the modules that implement them.
//   * A perf test. Tick 29's `benchmark` module covers that.

use std::collections::BTreeSet;

use crate::pkgmanage::pkgmgr::benchmark;
use crate::pkgmanage::pkgmgr::bytecode;
use crate::pkgmanage::pkgmgr::freshness;
use crate::pkgmanage::pkgmgr::groups::{
    DependencyGroups, GroupEntry, ProjectExtras,
};
use crate::pkgmanage::pkgmgr::indexes;
use crate::pkgmanage::pkgmgr::pep723;
use crate::pkgmanage::pkgmgr::platforms;
use crate::pkgmanage::pkgmgr::toolchain;
use crate::pkgmanage::pkgmgr::types::{FileHash, ReleaseFile};
use crate::pkgmanage::pkgmgr::upgrade::{
    pick_candidate, CandidateContext, ResolutionStrategy, UpgradeScope,
};
use crate::pkgmanage::pkgmgr::workspace;

/// Single parity outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParityOutcome {
    Pass,
    Fail { detail: String },
    Skipped { reason: String },
}

/// One row in the matrix.
pub struct ParityCheck {
    /// Short stable id, e.g. "workspace.discover_glob".
    pub id: &'static str,
    /// Which uv-parity surface this check belongs to.
    pub surface: &'static str,
    /// Closure that runs the check. Built once, called once per
    /// matrix run; the closure owns its fixtures.
    pub run: Box<dyn Fn() -> ParityOutcome + Send + Sync>,
}

impl std::fmt::Debug for ParityCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParityCheck")
            .field("id", &self.id)
            .field("surface", &self.surface)
            .finish()
    }
}

/// Collection of checks.
#[derive(Default)]
pub struct ParityMatrix {
    pub checks: Vec<ParityCheck>,
}

impl ParityMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<F>(&mut self, id: &'static str, surface: &'static str, run: F)
    where
        F: Fn() -> ParityOutcome + Send + Sync + 'static,
    {
        self.checks.push(ParityCheck {
            id,
            surface,
            run: Box::new(run),
        });
    }

    pub fn len(&self) -> usize {
        self.checks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.checks.is_empty()
    }
}

/// One row in the report.
#[derive(Debug, Clone)]
pub struct ParityResult {
    pub id: &'static str,
    pub surface: &'static str,
    pub outcome: ParityOutcome,
}

/// Aggregated report.
#[derive(Debug, Clone, Default)]
pub struct ParityReport {
    pub results: Vec<ParityResult>,
}

impl ParityReport {
    pub fn passed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, ParityOutcome::Pass))
            .count()
    }
    pub fn failed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, ParityOutcome::Fail { .. }))
            .count()
    }
    pub fn skipped(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, ParityOutcome::Skipped { .. }))
            .count()
    }
    pub fn total(&self) -> usize {
        self.results.len()
    }
    pub fn all_passed(&self) -> bool {
        self.failed() == 0
    }
    /// Multi-line text report. Suitable for CI log dump or human review.
    pub fn render_text(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "parity matrix: {} passed, {} failed, {} skipped ({} total)",
            self.passed(),
            self.failed(),
            self.skipped(),
            self.total(),
        );
        for r in &self.results {
            let tag = match &r.outcome {
                ParityOutcome::Pass => "ok      ",
                ParityOutcome::Fail { .. } => "FAIL    ",
                ParityOutcome::Skipped { .. } => "skip    ",
            };
            let _ = writeln!(out, "  {tag}{:32} [{}]", r.id, r.surface);
            if let ParityOutcome::Fail { detail } = &r.outcome {
                let _ = writeln!(out, "          -> {detail}");
            }
            if let ParityOutcome::Skipped { reason } = &r.outcome {
                let _ = writeln!(out, "          -> {reason}");
            }
        }
        out
    }
}

/// Run every check in the matrix.
pub fn run_matrix(matrix: &ParityMatrix) -> ParityReport {
    let mut results = Vec::with_capacity(matrix.checks.len());
    for check in &matrix.checks {
        let outcome = (check.run)();
        results.push(ParityResult {
            id: check.id,
            surface: check.surface,
            outcome,
        });
    }
    ParityReport { results }
}

// --- check helpers (kept short so default_matrix() reads as a table) ---

fn ok() -> ParityOutcome {
    ParityOutcome::Pass
}

fn fail(detail: impl Into<String>) -> ParityOutcome {
    ParityOutcome::Fail {
        detail: detail.into(),
    }
}

fn expect_eq<T: PartialEq + std::fmt::Debug>(
    label: &str,
    actual: T,
    expected: T,
) -> ParityOutcome {
    if actual == expected {
        ok()
    } else {
        fail(format!("{label}: expected {expected:?}, got {actual:?}"))
    }
}

// --- the default smoke matrix -----------------------------------------------

/// Build a matrix of one canonical check per shipped pkg-mgmt module.
/// Each check uses a deterministic in-memory fixture.
pub fn default_matrix() -> ParityMatrix {
    let mut m = ParityMatrix::new();

    // Tick 20: workspace member glob.
    m.push("workspace.parse_simple", "uv [tool.uv.workspace]", || {
        let toml_src = r#"
[project]
name = "root"
version = "0.1.0"

[tool.uv.workspace]
members = ["packages/*"]
exclude = ["packages/legacy"]
"#;
        match workspace::parse_workspace_config(toml_src) {
            Ok(Some(cfg)) => {
                if cfg.members == vec!["packages/*".to_string()]
                    && cfg.exclude == vec!["packages/legacy".to_string()]
                {
                    ok()
                } else {
                    fail(format!("unexpected workspace config: {cfg:?}"))
                }
            }
            Ok(None) => fail("expected Some(WorkspaceConfig), got None"),
            Err(e) => fail(format!("parse error: {e}")),
        }
    });

    // Tick 21: Python toolchain version parsing.
    m.push("toolchain.parse_version", "PEP 440 python version", || {
        let parsed: Result<toolchain::PythonVersion, _> = "3.12.5".parse();
        match parsed {
            Ok(v) if v.major == 3 && v.minor == 12 && v.patch == 5 => ok(),
            Ok(v) => fail(format!("unexpected version: {v:?}")),
            Err(e) => fail(format!("parse error: {e}")),
        }
    });

    // Tick 22: PEP 723 inline-script metadata.
    m.push("pep723.find_block_basic", "PEP 723", || {
        let src = "\
# /// script
# requires-python = \">=3.11\"
# dependencies = [\"requests\"]
# ///
import requests
";
        match pep723::parse_pep723(src) {
            Ok(Some(md)) => {
                if md.requires_python_raw.as_deref() == Some(">=3.11")
                    && md.dependencies == vec!["requests".to_string()]
                {
                    ok()
                } else {
                    fail(format!("unexpected metadata: {md:?}"))
                }
            }
            Ok(None) => fail("expected Some(ScriptMetadata), got None"),
            Err(e) => fail(format!("parse error: {e}")),
        }
    });

    // Tick 23: dependency-group expansion with include-group.
    m.push("groups.expand_include", "PEP 735", || {
        let mut g = DependencyGroups::default();
        g.by_name
            .insert("dev".to_string(), vec![GroupEntry::Requirement("pytest".into())]);
        g.by_name.insert(
            "test".to_string(),
            vec![
                GroupEntry::IncludeGroup("dev".into()),
                GroupEntry::Requirement("hypothesis".into()),
            ],
        );
        match g.expand(&["test"]) {
            Ok(reqs) => {
                let set: BTreeSet<String> = reqs.into_iter().collect();
                let expected: BTreeSet<String> =
                    ["pytest", "hypothesis"].iter().map(|s| s.to_string()).collect();
                if set == expected {
                    ok()
                } else {
                    fail(format!("unexpected expansion: {set:?}"))
                }
            }
            Err(e) => fail(format!("expand error: {e}")),
        }
    });

    // Tick 23 bonus: extras lookup (PEP 631).
    m.push("groups.extras_basic", "PEP 631 extras", || {
        let mut extras = ProjectExtras::default();
        extras
            .by_name
            .insert("docs".to_string(), vec!["sphinx".to_string()]);
        match extras.requirements_for("docs") {
            Some(reqs) => expect_eq("extras", reqs.to_vec(), vec!["sphinx".to_string()]),
            None => fail("expected Some(extras), got None"),
        }
    });

    // Tick 24: upgrade strategy → highest pick.
    m.push("upgrade.highest", "uv --resolution=highest", || {
        let available = ["1.0.0", "1.1.0", "2.0.0", "2.1.0"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let compatible = available.clone();
        let ctx = CandidateContext {
            package: "foo",
            available: &available,
            compatible: &compatible,
            locked: None,
            is_direct: true,
        };
        match pick_candidate(&ctx, ResolutionStrategy::Highest, &UpgradeScope::All) {
            Ok(v) => expect_eq("highest", v, "2.1.0".to_string()),
            Err(e) => fail(format!("pick error: {e}")),
        }
    });

    // Tick 24: upgrade strategy → lowest pick.
    m.push("upgrade.lowest", "uv --resolution=lowest", || {
        let available = ["1.0.0", "1.1.0", "2.0.0", "2.1.0"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let compatible = available.clone();
        let ctx = CandidateContext {
            package: "foo",
            available: &available,
            compatible: &compatible,
            locked: None,
            is_direct: true,
        };
        match pick_candidate(&ctx, ResolutionStrategy::Lowest, &UpgradeScope::All) {
            Ok(v) => expect_eq("lowest", v, "1.0.0".to_string()),
            Err(e) => fail(format!("pick error: {e}")),
        }
    });

    // Tick 25: cross-platform marker evaluation produces a non-empty
    // applicable environment set for a plain marker.
    m.push("platforms.python_version_gate", "PEP 508 + envs", || {
        let envs = platforms::EnvironmentSet::standard_matrix();
        match platforms::evaluate_marker_across_envs(
            Some("python_version >= \"3.10\""),
            &envs,
        ) {
            Ok(applicable) => {
                // All standard envs are >= 3.10 (the matrix starts at 3.10).
                expect_eq("matrix size", applicable.len(), envs.len())
            }
            Err(e) => fail(format!("marker eval: {e}")),
        }
    });

    // Tick 26: bytecode count walker — no temp dir; just validate that
    // `count_py_files` correctly errors on a definitely-missing path
    // (smoke for the I/O wiring; real file walking is unit-tested).
    m.push("bytecode.count_missing_errs", "compileall preflight", || {
        let bogus = std::path::Path::new("/definitely/not/a/real/path/abc-xyz");
        match bytecode::count_py_files(bogus) {
            Ok(n) => fail(format!("expected error, got count={n}")),
            Err(e) => {
                let msg = format!("{e}");
                if msg.contains("does not exist") {
                    ok()
                } else {
                    fail(format!("wrong error variant: {msg}"))
                }
            }
        }
    });

    // Tick 27: multi-index parse + implicit PyPI tail.
    m.push("indexes.parse_and_order", "[[tool.uv.index]]", || {
        let toml_src = r#"
[[tool.uv.index]]
name = "internal"
url = "https://internal.example/simple"
"#;
        match indexes::parse_indexes(toml_src) {
            Ok(cfg) => {
                let order = indexes::query_order(&cfg, "anything");
                if order.len() == 2
                    && order[0].name == "internal"
                    && order[1].name == "pypi"
                {
                    ok()
                } else {
                    let names: Vec<&str> =
                        order.iter().map(|i| i.name.as_str()).collect();
                    fail(format!("unexpected order: {names:?}"))
                }
            }
            Err(e) => fail(format!("parse error: {e}")),
        }
    });

    // Tick 28: yanked filter — pinned override.
    m.push("freshness.yanked_pinned_override", "PEP 592", || {
        let files = vec![ReleaseFile {
            filename: "foo-1.5.0-py3-none-any.whl".into(),
            url: "https://example.test/foo-1.5.0".into(),
            hash: FileHash::default(),
            requires_python: None,
            size: None,
            upload_time: None,
            yanked: true,
            yanked_reason: Some("CVE".into()),
            dist_info_metadata: serde_json::Value::Null,
            source: None,
        }];
        let mut pinned = BTreeSet::new();
        pinned.insert("1.5.0".to_string());
        let decision = freshness::filter_yanked(
            &files,
            freshness::YankedPolicy::AllowPinnedOnly,
            &pinned,
            |f| {
                // Parse "foo-X.Y.Z-..." → "X.Y.Z".
                f.filename.split('-').nth(1).unwrap_or("")
            },
        );
        if decision.usable.len() == 1 && decision.warnings.len() == 1 {
            ok()
        } else {
            fail(format!("unexpected decision: {decision:?}"))
        }
    });

    // Tick 29: comparison verdict — parity band.
    m.push("benchmark.parity_band", "harness vs uv", || {
        let mamba = benchmark::BenchSummary {
            label: "mamba".into(),
            samples: vec![benchmark::BenchSample {
                wall: std::time::Duration::from_millis(100),
                exit_code: Some(0),
                stderr_tail: String::new(),
            }],
            failures: 0,
        };
        let uv = benchmark::BenchSummary {
            label: "uv".into(),
            samples: vec![benchmark::BenchSample {
                wall: std::time::Duration::from_millis(102),
                exit_code: Some(0),
                stderr_tail: String::new(),
            }],
            failures: 0,
        };
        let c = benchmark::compare(mamba, uv, 0.05);
        if c.verdict == benchmark::ComparisonVerdict::ParityWith {
            ok()
        } else {
            fail(format!("expected ParityWith, got {:?}", c.verdict))
        }
    });

    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matrix_is_non_empty() {
        let m = default_matrix();
        assert!(m.len() >= 10, "expected ≥10 parity cells, got {}", m.len());
    }

    #[test]
    fn default_matrix_all_pass() {
        let report = run_matrix(&default_matrix());
        assert!(
            report.all_passed(),
            "parity report had failures:\n{}",
            report.render_text(),
        );
        assert_eq!(report.skipped(), 0, "no defaults should skip");
    }

    #[test]
    fn report_counts_each_outcome_bucket() {
        let mut m = ParityMatrix::new();
        m.push("a", "surface-a", || ParityOutcome::Pass);
        m.push("b", "surface-b", || ParityOutcome::Fail {
            detail: "synthetic".into(),
        });
        m.push("c", "surface-c", || ParityOutcome::Skipped {
            reason: "fixture missing".into(),
        });
        let report = run_matrix(&m);
        assert_eq!(report.total(), 3);
        assert_eq!(report.passed(), 1);
        assert_eq!(report.failed(), 1);
        assert_eq!(report.skipped(), 1);
        assert!(!report.all_passed());
    }

    #[test]
    fn render_text_marks_failures_distinctly() {
        let mut m = ParityMatrix::new();
        m.push("ok", "surface", || ParityOutcome::Pass);
        m.push("bad", "surface", || ParityOutcome::Fail {
            detail: "details here".into(),
        });
        let text = run_matrix(&m).render_text();
        assert!(text.contains("ok      ok"), "missing ok marker in: {text}");
        assert!(text.contains("FAIL    bad"), "missing FAIL marker in: {text}");
        assert!(text.contains("details here"), "missing detail in: {text}");
    }

    #[test]
    fn render_text_surfaces_skip_reason() {
        let mut m = ParityMatrix::new();
        m.push("s", "surface", || ParityOutcome::Skipped {
            reason: "no python on PATH".into(),
        });
        let text = run_matrix(&m).render_text();
        assert!(text.contains("skip    s"));
        assert!(text.contains("no python on PATH"));
    }

    #[test]
    fn expect_eq_helper_passes_on_match() {
        let r = expect_eq("k", 5, 5);
        assert_eq!(r, ParityOutcome::Pass);
    }

    #[test]
    fn expect_eq_helper_fails_with_diagnostic() {
        let r = expect_eq("k", 5, 6);
        match r {
            ParityOutcome::Fail { detail } => assert!(detail.contains("expected 6")),
            other => panic!("expected Fail, got {other:?}"),
        }
    }
}
