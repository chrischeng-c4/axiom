// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-src.md#schema
// CODEGEN-BEGIN
//! Static report packaging for shareable e2e evidence.
//!
//! Takes a finished evidence bundle and packs it into a self-contained
//! directory that opens as a static report — `index.html`, the bundle
//! JSON, the events JSONL, and all referenced screenshot/trace
//! artifacts. Opening `index.html` in a browser shows the same UI as
//! `jet e2e open` (controls disabled), so the report can be shared,
//! attached to a PR, or archived without needing a live runner.
//!
//! The packager is read-only: it loads evidence, rewrites artifact
//! paths to portable relatives inside the package, and writes files.
//! It does not start the browser, the dev server, or the open-mode
//! adapter.
// @spec #2620

use crate::e2e::{render_pm_report_html, E2eArtifactRef, E2eEvidenceBundle};
use crate::evidence::{events_for, EvidenceBundle};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Outcome of packaging one evidence bundle into a static report
/// directory. The fields are absolute paths to the files written.
// @spec #2620
#[derive(Debug, Clone)]
pub struct StaticReportPackage {
    pub report_dir: PathBuf,
    pub index_html: PathBuf,
    pub bundle_json: PathBuf,
    pub events_jsonl: PathBuf,
    pub artifact_dir: PathBuf,
    pub copied_artifacts: Vec<CopiedArtifact>,
    pub missing_artifacts: Vec<E2eArtifactRef>,
}

/// One artifact that was successfully copied into the package.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone)]
pub struct CopiedArtifact {
    /// Original artifact reference as it appeared in the bundle.
    pub original: E2eArtifactRef,
    /// Absolute destination path inside the package.
    pub absolute: PathBuf,
    /// Package-relative path (joins onto report_dir to get absolute).
    pub relative: PathBuf,
}

/// Package an evidence bundle into a static, shareable report directory.
///
/// - `bundle` is the in-memory evidence (must have already passed
///   `EvidenceBundle` shape — load via `crate::evidence`).
/// - `source_root` is the directory artifact `path` fields are resolved
///   against (typically the directory the `.evidence.json` was loaded
///   from, or the project root for relative paths).
/// - `out_dir` is the report directory. Created if missing.
///
/// Behavior:
/// - Writes `index.html` (PM-facing review UI, identical DOM to open
///   mode, dev-only controls hidden, no network polling).
/// - Writes `<run_id>.evidence.json` and `<run_id>.events.jsonl`.
/// - Copies every referenced artifact (top-level + per-step) into
///   `artifacts/` inside the package, rewriting the embedded bundle's
///   artifact paths so they remain relative to the package.
/// - Missing artifacts are recorded in [`StaticReportPackage::missing_artifacts`]
///   and **do not** abort packaging — the report still opens, and the
///   PM report renders them as unavailable data.
///
/// The returned package's `index_html` opens directly in a browser via
/// `file://` — no live runner process required.
// @spec #2620
pub fn package_static_report(
    bundle: &EvidenceBundle,
    source_root: &Path,
    out_dir: &Path,
) -> Result<StaticReportPackage> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("creating report dir {}", out_dir.display()))?;
    let artifact_dir = out_dir.join("artifacts");
    std::fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("creating artifacts dir {}", artifact_dir.display()))?;

    let mut packaged = bundle.clone();
    let mut copied: Vec<CopiedArtifact> = Vec::new();
    let mut missing: Vec<E2eArtifactRef> = Vec::new();
    let mut used_names: Vec<String> = Vec::new();

    rewrite_artifacts(
        &mut packaged.artifacts,
        source_root,
        &artifact_dir,
        &mut copied,
        &mut missing,
        &mut used_names,
    )?;
    for case in packaged.cases.iter_mut() {
        for step in case.steps.iter_mut() {
            rewrite_artifacts(
                &mut step.context.screenshots,
                source_root,
                &artifact_dir,
                &mut copied,
                &mut missing,
                &mut used_names,
            )?;
        }
    }

    let bundle_json = out_dir.join(format!("{}.evidence.json", packaged.run_id));
    std::fs::write(&bundle_json, serde_json::to_vec_pretty(&packaged)?)
        .with_context(|| format!("writing {}", bundle_json.display()))?;

    let events_jsonl = out_dir.join(format!("{}.events.jsonl", packaged.run_id));
    let mut jsonl = String::new();
    for event in events_for(&packaged) {
        jsonl.push_str(&serde_json::to_string(&event)?);
        jsonl.push('\n');
    }
    std::fs::write(&events_jsonl, jsonl)
        .with_context(|| format!("writing {}", events_jsonl.display()))?;

    let index_html = out_dir.join("index.html");
    let html = render_pm_report_html(&packaged)?;
    std::fs::write(&index_html, html)
        .with_context(|| format!("writing {}", index_html.display()))?;

    Ok(StaticReportPackage {
        report_dir: out_dir.to_path_buf(),
        index_html,
        bundle_json,
        events_jsonl,
        artifact_dir,
        copied_artifacts: copied,
        missing_artifacts: missing,
    })
}

fn rewrite_artifacts(
    artifacts: &mut Vec<E2eArtifactRef>,
    source_root: &Path,
    artifact_dir: &Path,
    copied: &mut Vec<CopiedArtifact>,
    missing: &mut Vec<E2eArtifactRef>,
    used_names: &mut Vec<String>,
) -> Result<()> {
    for artifact in artifacts.iter_mut() {
        let original = artifact.clone();
        let source = resolve_source_path(&artifact.path, source_root);
        if !source.exists() {
            missing.push(original);
            continue;
        }
        let dest_name = pick_unique_basename(&artifact.path, used_names);
        let dest_abs = artifact_dir.join(&dest_name);
        std::fs::copy(&source, &dest_abs)
            .with_context(|| format!("copying {} → {}", source.display(), dest_abs.display()))?;
        artifact.path = PathBuf::from("artifacts").join(&dest_name);
        copied.push(CopiedArtifact {
            original,
            absolute: dest_abs,
            relative: artifact.path.clone(),
        });
    }
    Ok(())
}

fn resolve_source_path(path: &Path, source_root: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        source_root.join(path)
    }
}

/// GH #3741 — build the warn body for `pick_unique_basename` falling
/// back to `"artifact.bin"`. Two distinct reasons:
/// `"no file_name component"` (path is `/`, `.`, `..`) and
/// `"file_name is not valid UTF-8"` (filesystem allows arbitrary bytes
/// on Linux/macOS). Extracted so the wording is unit-testable without
/// having to construct a real filesystem entry with non-UTF-8 bytes.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn format_artifact_basename_fallback_warn(path: &Path, reason: &str) -> String {
    format!(
        "GH #3741 report_package::pick_unique_basename falling back to \
         `artifact.bin` for path {:?} ({reason}); the static report will \
         show a generic name and viewers of the SHARED report cannot \
         map it back to the original. Check whether the artifact \
         pipeline upstream is passing a nameless or non-UTF-8 path.",
        path.display()
    )
}

fn pick_unique_basename(original: &Path, used: &mut Vec<String>) -> String {
    let base = match original.file_name() {
        Some(name) => match name.to_str() {
            Some(s) => s.to_string(),
            None => {
                let reason = "file_name is not valid UTF-8";
                tracing::warn!(
                    target: "jet::report_package",
                    path = %original.display(),
                    lossy_file_name = %name.to_string_lossy(),
                    reason = %reason,
                    "{}",
                    format_artifact_basename_fallback_warn(original, reason)
                );
                "artifact.bin".to_string()
            }
        },
        None => {
            let reason = "no file_name component";
            tracing::warn!(
                target: "jet::report_package",
                path = %original.display(),
                reason = %reason,
                "{}",
                format_artifact_basename_fallback_warn(original, reason)
            );
            "artifact.bin".to_string()
        }
    };
    if !used.contains(&base) {
        used.push(base.clone());
        return base;
    }
    let (stem, ext) = match base.rsplit_once('.') {
        Some((s, e)) => (s.to_string(), format!(".{e}")),
        None => (base.clone(), String::new()),
    };
    for n in 1..u32::MAX {
        let candidate = format!("{stem}-{n}{ext}");
        if !used.contains(&candidate) {
            used.push(candidate.clone());
            return candidate;
        }
    }
    base
}

/// Convenience: read a bundle from a `.evidence.json` and package it.
///
/// Source root for artifact resolution defaults to the bundle file's
/// parent directory — matches the layout `jet e2e run` writes.
// @spec #2620
pub fn package_from_file(bundle_path: &Path, out_dir: &Path) -> Result<StaticReportPackage> {
    let bundle: E2eEvidenceBundle = serde_json::from_slice(&std::fs::read(bundle_path)?)
        .with_context(|| format!("parsing evidence bundle at {}", bundle_path.display()))?;
    let source_root = bundle_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    package_static_report(&bundle, &source_root, out_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::e2e::{
        E2eCaseEvidence, E2eMode, E2eProductStep, E2eStepContext, E2eSummary,
        EVIDENCE_SCHEMA_VERSION,
    };

    fn bundle_with_screenshots() -> E2eEvidenceBundle {
        E2eEvidenceBundle {
            schema_version: EVIDENCE_SCHEMA_VERSION.to_string(),
            mode: E2eMode::Run,
            run_id: "static-run".to_string(),
            started_at_ms: 1_000,
            finished_at_ms: 2_000,
            summary: E2eSummary {
                passed: 0,
                failed: 1,
                skipped: 0,
                duration_ms: 1_000,
                exit_code: 1,
            },
            cases: vec![E2eCaseEvidence {
                id: "case-1".to_string(),
                title: "promote".to_string(),
                file: PathBuf::from("e2e/promote.spec.js"),
                outcome: "failed".to_string(),
                duration_ms: 1_000,
                steps: vec![E2eProductStep {
                    id: "step-1".to_string(),
                    title: "publish".to_string(),
                    status: "failed".to_string(),
                    duration_ms: 500,
                    assertion: None,
                    context: E2eStepContext {
                        screenshots: vec![E2eArtifactRef {
                            kind: "screenshot".to_string(),
                            path: PathBuf::from("shots/failure.png"),
                            label: Some("step capture".to_string()),
                        }],
                        ..Default::default()
                    },
                }],
            }],
            artifacts: vec![E2eArtifactRef {
                kind: "trace".to_string(),
                path: PathBuf::from("trace.zip"),
                label: Some("trace zip".to_string()),
            }],
            open_control: None,
        }
    }

    #[test]
    fn package_writes_index_evidence_events_and_artifacts() {
        let src = tempfile::tempdir().unwrap();
        let shots = src.path().join("shots");
        std::fs::create_dir_all(&shots).unwrap();
        std::fs::write(shots.join("failure.png"), b"png-bytes").unwrap();
        std::fs::write(src.path().join("trace.zip"), b"zip-bytes").unwrap();

        let out = tempfile::tempdir().unwrap();
        let pkg = package_static_report(&bundle_with_screenshots(), src.path(), out.path())
            .expect("package");

        assert!(pkg.index_html.exists());
        assert!(pkg.bundle_json.exists());
        assert!(pkg.events_jsonl.exists());
        assert!(pkg.artifact_dir.exists());
        assert_eq!(pkg.copied_artifacts.len(), 2);
        assert!(pkg.missing_artifacts.is_empty());

        // All copied artifacts are reachable on disk.
        for c in &pkg.copied_artifacts {
            assert!(c.absolute.exists(), "{:?} missing", c.absolute);
            assert!(c.relative.starts_with("artifacts"));
        }
    }

    #[test]
    fn package_rewrites_bundle_paths_to_relative_package_paths() {
        // @spec #2620 — artifact paths in the packaged bundle stay
        // relative, so the report directory is portable when zipped.
        let src = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("shots")).unwrap();
        std::fs::write(src.path().join("shots/failure.png"), b"png").unwrap();
        std::fs::write(src.path().join("trace.zip"), b"zip").unwrap();
        let out = tempfile::tempdir().unwrap();
        package_static_report(&bundle_with_screenshots(), src.path(), out.path()).unwrap();

        let bundle_path = out.path().join("static-run.evidence.json");
        let body = std::fs::read_to_string(&bundle_path).unwrap();
        let parsed: E2eEvidenceBundle = serde_json::from_str(&body).unwrap();
        for artifact in &parsed.artifacts {
            assert!(
                artifact.path.starts_with("artifacts"),
                "top-level artifact path stays in-package: {:?}",
                artifact.path,
            );
            assert!(!artifact.path.is_absolute());
        }
        for case in &parsed.cases {
            for step in &case.steps {
                for shot in &step.context.screenshots {
                    assert!(
                        shot.path.starts_with("artifacts"),
                        "step screenshot path stays in-package: {:?}",
                        shot.path,
                    );
                }
            }
        }
    }

    #[test]
    fn package_records_missing_artifacts_without_aborting() {
        // @spec #2620 — packaging proceeds even if some artifacts are
        // gone; missing ones are reported, the report still opens.
        let src = tempfile::tempdir().unwrap();
        // Only the trace exists; the screenshot is absent.
        std::fs::write(src.path().join("trace.zip"), b"zip").unwrap();
        let out = tempfile::tempdir().unwrap();
        let pkg = package_static_report(&bundle_with_screenshots(), src.path(), out.path())
            .expect("package");

        assert!(pkg.index_html.exists());
        assert_eq!(pkg.copied_artifacts.len(), 1, "trace was present");
        assert_eq!(pkg.missing_artifacts.len(), 1, "screenshot was missing");
        assert!(pkg.missing_artifacts[0]
            .path
            .as_os_str()
            .to_str()
            .unwrap()
            .contains("failure.png"));
    }

    #[test]
    fn package_index_html_uses_pm_report_review_ui() {
        // @spec #2620, #2622 — packaged index.html embeds the PM-facing
        // flavor of the review UI; it does NOT require jet e2e open
        // running, and dev-only controls are hidden via CSS.
        let src = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("shots")).unwrap();
        std::fs::write(src.path().join("shots/failure.png"), b"x").unwrap();
        std::fs::write(src.path().join("trace.zip"), b"x").unwrap();
        let out = tempfile::tempdir().unwrap();
        let pkg =
            package_static_report(&bundle_with_screenshots(), src.path(), out.path()).unwrap();

        let html = std::fs::read_to_string(&pkg.index_html).unwrap();
        assert!(
            html.contains("data-mode=\"pm-report\""),
            "packaged HTML must render in pm-report mode (no live polling)",
        );
        // CSS rule that hides the dev-only toolbar and command log when
        // the body carries the pm-report data-mode flag must be present.
        assert!(
            html.contains("body[data-mode=\"pm-report\"] .toolbar"),
            "pm-report mode must hide the dev toolbar via CSS",
        );
        // No control-protocol fetches over network.
        assert!(
            !html.contains("/api/live-control"),
            "static report must not embed a live-control endpoint",
        );
    }

    #[test]
    fn package_basename_collisions_disambiguate() {
        // Two artifacts referencing the same basename should both land
        // in the package without one clobbering the other.
        let src = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("a")).unwrap();
        std::fs::create_dir_all(src.path().join("b")).unwrap();
        std::fs::write(src.path().join("a/shot.png"), b"a").unwrap();
        std::fs::write(src.path().join("b/shot.png"), b"b").unwrap();

        let bundle = E2eEvidenceBundle {
            schema_version: EVIDENCE_SCHEMA_VERSION.to_string(),
            mode: E2eMode::Run,
            run_id: "collide".to_string(),
            started_at_ms: 0,
            finished_at_ms: 0,
            summary: E2eSummary::default(),
            cases: vec![],
            artifacts: vec![
                E2eArtifactRef {
                    kind: "screenshot".to_string(),
                    path: PathBuf::from("a/shot.png"),
                    label: None,
                },
                E2eArtifactRef {
                    kind: "screenshot".to_string(),
                    path: PathBuf::from("b/shot.png"),
                    label: None,
                },
            ],
            open_control: None,
        };

        let out = tempfile::tempdir().unwrap();
        let pkg = package_static_report(&bundle, src.path(), out.path()).unwrap();
        assert_eq!(pkg.copied_artifacts.len(), 2);
        // Two distinct destination filenames.
        let names: std::collections::BTreeSet<&str> = pkg
            .copied_artifacts
            .iter()
            .filter_map(|c| c.absolute.file_name().and_then(|n| n.to_str()))
            .collect();
        assert_eq!(names.len(), 2, "collisions disambiguated, got {names:?}");
    }

    #[test]
    fn package_from_file_loads_and_packages_disk_bundle() {
        let src = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("shots")).unwrap();
        std::fs::write(src.path().join("shots/failure.png"), b"x").unwrap();
        std::fs::write(src.path().join("trace.zip"), b"x").unwrap();
        let bundle_path = src.path().join("static-run.evidence.json");
        std::fs::write(
            &bundle_path,
            serde_json::to_vec_pretty(&bundle_with_screenshots()).unwrap(),
        )
        .unwrap();

        let out = tempfile::tempdir().unwrap();
        let pkg = package_from_file(&bundle_path, out.path()).expect("package");
        assert!(pkg.index_html.exists());
        assert_eq!(pkg.copied_artifacts.len(), 2);
    }
}

#[cfg(test)]
mod gh3741_artifact_basename_fallback_warn_tests {
    //! GH #3741 — `pick_unique_basename` previously did
    //! `original.file_name().and_then(|n| n.to_str()).unwrap_or("artifact.bin")`,
    //! silently collapsing two distinct failure modes (nameless path
    //! vs non-UTF-8 file_name) to a generic name. These tests pin the
    //! helper wording and the two-reason distinguishing behaviour.
    use super::*;

    #[test]
    fn gh3741_helper_includes_tag_path_and_reason() {
        let p = Path::new("/proj/evidence/case-42/trace.zip");
        for reason in ["no file_name component", "file_name is not valid UTF-8"] {
            let msg = format_artifact_basename_fallback_warn(p, reason);
            assert!(
                msg.contains("GH #3741"),
                "must include issue tag (reason={reason}): {msg}"
            );
            assert!(
                msg.contains("/proj/evidence/case-42/trace.zip"),
                "must name the offending path (reason={reason}): {msg}"
            );
            assert!(
                msg.contains(reason),
                "must name the fallback reason (reason={reason}): {msg}"
            );
            assert!(
                msg.contains("artifact.bin"),
                "must name the fallback target so log readers know what to grep: {msg}"
            );
        }
    }

    #[test]
    fn gh3741_two_reasons_are_pairwise_distinct() {
        let p = Path::new("/x");
        let r1 = format_artifact_basename_fallback_warn(p, "no file_name component");
        let r2 = format_artifact_basename_fallback_warn(p, "file_name is not valid UTF-8");
        assert_ne!(r1, r2);
    }

    #[test]
    fn gh3741_helper_includes_remediation_hint() {
        let msg = format_artifact_basename_fallback_warn(Path::new("/x"), "no file_name component");
        assert!(
            msg.contains("upstream") || msg.contains("artifact pipeline"),
            "must hint at the upstream root cause: {msg}"
        );
    }

    #[test]
    fn gh3741_helper_is_deterministic() {
        let p = Path::new("/x");
        let a = format_artifact_basename_fallback_warn(p, "no file_name component");
        let b = format_artifact_basename_fallback_warn(p, "no file_name component");
        assert_eq!(a, b);
    }

    /// The fix preserves behaviour: nameless input still returns
    /// `"artifact.bin"`. Only the silence is fixed (a warn now fires).
    #[test]
    fn gh3741_pick_unique_basename_on_root_path_returns_artifact_bin() {
        let mut used = Vec::new();
        let out = pick_unique_basename(Path::new("/"), &mut used);
        assert_eq!(out, "artifact.bin");
        assert_eq!(used, vec!["artifact.bin"]);
    }

    /// Dedupe still works on top of the fallback: two nameless paths
    /// get `artifact.bin` and `artifact-1.bin`.
    #[test]
    fn gh3741_pick_unique_basename_dedupes_two_root_paths() {
        let mut used = Vec::new();
        let a = pick_unique_basename(Path::new("/"), &mut used);
        let b = pick_unique_basename(Path::new("/"), &mut used);
        assert_eq!(a, "artifact.bin");
        assert_eq!(b, "artifact-1.bin");
    }

    /// Happy path: a real filename is unaffected (no fallback, no warn).
    #[test]
    fn gh3741_happy_path_named_file_is_unchanged() {
        let mut used = Vec::new();
        let out = pick_unique_basename(Path::new("/proj/screenshot.png"), &mut used);
        assert_eq!(out, "screenshot.png");
        assert_eq!(used, vec!["screenshot.png"]);
    }

    /// Non-UTF-8 file_name on Unix: construct an `OsStr` from bytes and
    /// verify the fallback path returns `"artifact.bin"`. The warn
    /// fires inside `pick_unique_basename` (not asserted in this test
    /// because tracing isn't a captured output by default — the warn
    /// body's contract is pinned by the helper tests above).
    #[cfg(unix)]
    #[test]
    fn gh3741_non_utf8_file_name_returns_artifact_bin_fallback() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        // Bytes 0xFF 0xFE 0xFD are not a valid UTF-8 sequence.
        let bad_bytes: &[u8] = &[
            b'/', b't', b'r', b'a', b'c', b'e', 0xFF, 0xFE, b'.', b'z', b'i', b'p',
        ];
        let bad_os = OsStr::from_bytes(bad_bytes);
        let bad_path = Path::new(bad_os);
        // Sanity: file_name() returns Some but to_str() returns None.
        let name = bad_path.file_name().expect("must have a file_name");
        assert!(
            name.to_str().is_none(),
            "test premise: file_name must be non-UTF-8"
        );

        let mut used = Vec::new();
        let out = pick_unique_basename(bad_path, &mut used);
        assert_eq!(out, "artifact.bin");
    }

    /// Sibling-distinctness: this helper's tag must NOT collide with
    /// prior `format_*_warn` helpers in the silent-fallback family.
    #[test]
    fn gh3741_helper_does_not_leak_sibling_tags() {
        let msg = format_artifact_basename_fallback_warn(Path::new("/x"), "no file_name component");
        for sibling in [
            "GH #3725", "GH #3727", "GH #3730", "GH #3732", "GH #3734", "GH #3737", "GH #3739",
        ] {
            assert!(
                !msg.contains(sibling),
                "#3741 msg must not leak sibling tag {sibling}: {msg}"
            );
        }
    }

    /// Naming convention discoverability: helper is named
    /// `format_artifact_basename_fallback_warn`, matching the
    /// project-wide `format_<area>_<thing>_warn` convention.
    #[test]
    fn gh3741_helper_name_follows_family_convention() {
        let name = "format_artifact_basename_fallback_warn";
        assert!(name.starts_with("format_"));
        assert!(name.ends_with("_warn"));
    }
}
// CODEGEN-END
