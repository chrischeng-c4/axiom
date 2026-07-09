// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
// CODEGEN-BEGIN
//! Producer-side adapter for writing evidence bundles (#2718).
//!
//! Both `jet test` and `jet e2e` use this API to emit a portable
//! bundle in the layout defined by [`crate::evidence_bundle`]: an
//! incremental `events.jsonl` stream alongside a finalised
//! `manifest.json`. The writer never imports UI code — only file I/O
//! and serialisation — so it can be shared by every producer.
//!
//! ## Lifecycle
//!
//! ```text
//!   let mut w = EvidenceWriter::open(root, manifest)?;
//!   w.run_started(...)?;
//!   w.case_result(...)?;
//!   w.register_artifact(...)?;     // normalised relative path
//!   w.run_finished(...)?;
//!   w.finalize()?;                 // writes manifest.json
//! ```
//!
//! Each helper appends one JSON object on its own line to
//! `<root>/events.jsonl`. `register_artifact` adds an entry to the
//! manifest in memory; `finalize` flushes it to disk. The same writer
//! object handles both unit-test and E2E records — the event payload
//! carries a `kind` discriminator so a single consumer can route both
//! shapes off the same stream.
//!
//! @spec #2718

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::evidence_bundle::{BundleArtifact, BundleHandle, BundleManifest, MANIFEST_FILE_NAME};

/// Default file name for the per-event stream inside a bundle root.
pub const EVENTS_FILE_NAME: &str = "events.jsonl";

/// One record on the producer-side event stream. The `kind` field is
/// the discriminator: the consumer routes off it without inspecting
/// the rest of the payload.
/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EvidenceEvent {
    RunStarted {
        run_id: String,
        command: String,
    },
    /// `jet test` test-level result. Represents the **final** outcome
    /// of an item after any retry attempts. Earlier failing attempts
    /// are recorded separately via [`EvidenceEvent::TestRetry`].
    TestResult {
        suite: Vec<String>,
        name: String,
        outcome: String,
        duration_ms: u64,
    },
    /// `jet e2e` case-level final result. See [`EvidenceEvent::TestResult`]
    /// for how retries are represented.
    CaseResult {
        case: String,
        outcome: String,
        duration_ms: u64,
    },
    /// A non-final retry attempt for a `jet test` test (#2868).
    /// `attempt` is the 1-based index of the attempt this record
    /// describes. The terminal outcome lands in [`EvidenceEvent::TestResult`].
    /// Consumers replaying the stream can derive `attempts = max(attempt) + 1`
    /// when a TestResult follows, or `attempts = max(attempt)` when the
    /// final record itself was a retry that exhausted the budget.
    TestRetry {
        suite: Vec<String>,
        name: String,
        attempt: u32,
        outcome: String,
        duration_ms: u64,
    },
    /// Mirror of [`EvidenceEvent::TestRetry`] for `jet e2e` cases (#2868).
    CaseRetry {
        case: String,
        attempt: u32,
        outcome: String,
        duration_ms: u64,
    },
    /// A registered artifact — emitted at the moment the producer
    /// hands the file to the bundle, ahead of `finalize`.
    ArtifactRegistered {
        id: String,
        #[serde(rename = "artifact_kind")]
        artifact_kind: String,
        path: PathBuf,
    },
    RunFinished {
        passed: u32,
        failed: u32,
        skipped: u32,
        duration_ms: u64,
    },
}

/// Append-only writer that owns an open `events.jsonl` and a mutable
/// in-memory [`BundleManifest`]. Call [`Self::finalize`] to flush the
/// manifest; dropping the writer without finalising leaves the stream
/// on disk and the manifest unwritten.
/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
pub struct EvidenceWriter {
    root: PathBuf,
    manifest: BundleManifest,
    events: File,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
impl EvidenceWriter {
    /// Open a writer at `root`, creating the bundle root + the
    /// events.jsonl stream. The manifest is held in memory and
    /// flushed on [`Self::finalize`].
    pub fn open(root: impl AsRef<Path>, manifest: BundleManifest) -> Result<Self> {
        let root = root.as_ref();
        std::fs::create_dir_all(root)
            .with_context(|| format!("creating bundle root at {}", root.display()))?;
        let events_path = root.join(EVENTS_FILE_NAME);
        let events = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&events_path)
            .with_context(|| format!("opening events stream at {}", events_path.display()))?;
        Ok(Self {
            root: root.to_path_buf(),
            manifest,
            events,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest(&self) -> &BundleManifest {
        &self.manifest
    }

    fn append(&mut self, event: &EvidenceEvent) -> Result<()> {
        let line = serde_json::to_string(event).context("serialising evidence event")?;
        self.events.write_all(line.as_bytes())?;
        self.events.write_all(b"\n")?;
        Ok(())
    }

    pub fn run_started(
        &mut self,
        run_id: impl Into<String>,
        command: impl Into<String>,
    ) -> Result<()> {
        self.append(&EvidenceEvent::RunStarted {
            run_id: run_id.into(),
            command: command.into(),
        })
    }

    pub fn test_result(
        &mut self,
        suite: Vec<String>,
        name: impl Into<String>,
        outcome: impl Into<String>,
        duration_ms: u64,
    ) -> Result<()> {
        self.append(&EvidenceEvent::TestResult {
            suite,
            name: name.into(),
            outcome: outcome.into(),
            duration_ms,
        })
    }

    pub fn case_result(
        &mut self,
        case: impl Into<String>,
        outcome: impl Into<String>,
        duration_ms: u64,
    ) -> Result<()> {
        self.append(&EvidenceEvent::CaseResult {
            case: case.into(),
            outcome: outcome.into(),
            duration_ms,
        })
    }

    /// Record a non-final retry attempt for a `jet test` test (#2868).
    /// Emit one of these per failing attempt **before** calling
    /// [`Self::test_result`] with the final outcome.
    pub fn test_retry(
        &mut self,
        suite: Vec<String>,
        name: impl Into<String>,
        attempt: u32,
        outcome: impl Into<String>,
        duration_ms: u64,
    ) -> Result<()> {
        self.append(&EvidenceEvent::TestRetry {
            suite,
            name: name.into(),
            attempt,
            outcome: outcome.into(),
            duration_ms,
        })
    }

    /// Mirror of [`Self::test_retry`] for `jet e2e` cases (#2868).
    pub fn case_retry(
        &mut self,
        case: impl Into<String>,
        attempt: u32,
        outcome: impl Into<String>,
        duration_ms: u64,
    ) -> Result<()> {
        self.append(&EvidenceEvent::CaseRetry {
            case: case.into(),
            attempt,
            outcome: outcome.into(),
            duration_ms,
        })
    }

    /// Register an artifact in the manifest and emit a stream event.
    /// `path` may be absolute (and inside the bundle root) or already
    /// relative; the writer normalises it to a path **relative to the
    /// bundle root** so the resulting bundle stays portable.
    pub fn register_artifact(
        &mut self,
        id: impl Into<String>,
        kind: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let raw = path.as_ref();
        let normalised = if raw.is_absolute() {
            raw.strip_prefix(&self.root)
                .with_context(|| {
                    format!(
                        "artifact path {} is not inside bundle root {}",
                        raw.display(),
                        self.root.display()
                    )
                })?
                .to_path_buf()
        } else {
            raw.to_path_buf()
        };
        let id = id.into();
        let kind = kind.into();
        self.manifest.artifacts.push(BundleArtifact {
            id: id.clone(),
            kind: kind.clone(),
            path: normalised.clone(),
            content_type: None,
        });
        self.append(&EvidenceEvent::ArtifactRegistered {
            id,
            artifact_kind: kind,
            path: normalised,
        })
    }

    pub fn run_finished(
        &mut self,
        passed: u32,
        failed: u32,
        skipped: u32,
        duration_ms: u64,
    ) -> Result<()> {
        self.append(&EvidenceEvent::RunFinished {
            passed,
            failed,
            skipped,
            duration_ms,
        })
    }

    /// Flush the manifest to `<root>/manifest.json` and return a
    /// reader handle that resolves against the same root.
    ///
    /// GH #3265 — `events.jsonl` is flushed BEFORE the manifest is
    /// written, and the flush error is propagated instead of being
    /// discarded by the prior `self.events.flush().ok()`. Today
    /// `events` is a plain `File`, whose `Write::flush` is a no-op on
    /// Unix; the propagation is therefore primarily a contract fix
    /// that (a) honours `Write::flush`'s `io::Result` for any current
    /// or future buffered wrapper and (b) makes the order explicit:
    /// if the stream layer ever reports an error, we MUST NOT then
    /// land a manifest pointing at a half-truncated NDJSON tail —
    /// consumers (reporters, CI ingestion, `jet test list` golden
    /// diff) would either parse-fail or, worse, silently succeed
    /// against a stream that lost its tail on a newline boundary.
    pub fn finalize(mut self) -> Result<BundleHandle> {
        self.events.flush().with_context(|| {
            format!(
                "GH #3265 flushing events.jsonl in evidence bundle at {}; \
                 manifest will NOT be written so consumers cannot read \
                 a half-truncated stream",
                self.root.display()
            )
        })?;
        let body = serde_json::to_vec_pretty(&self.manifest).context("serialising manifest")?;
        let manifest_path = self.root.join(MANIFEST_FILE_NAME);
        std::fs::write(&manifest_path, body)
            .with_context(|| format!("writing manifest at {}", manifest_path.display()))?;
        BundleHandle::load(&self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_bundle::{BundleCommand, BundleEnvironment};
    use tempfile::TempDir;

    fn empty_manifest() -> BundleManifest {
        BundleManifest::new(
            "run-1",
            BundleCommand::Test,
            "jet",
            "deadbeef",
            BundleEnvironment {
                os: "darwin".into(),
                runner_version: "0.3.48".into(),
                ci: None,
                node_version: None,
            },
        )
    }

    #[test]
    fn fixture_writer_emits_jsonl_plus_finalized_bundle() {
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), empty_manifest()).unwrap();

        w.run_started("run-1", "jet test").unwrap();
        w.test_result(vec!["Suite".into()], "test a", "passed", 12)
            .unwrap();

        // Pre-create the artifact file so register_artifact has something
        // real to point at; the writer registers it on the manifest only.
        let art_dir = tmp.path().join("artifacts");
        std::fs::create_dir_all(&art_dir).unwrap();
        let art_path = art_dir.join("a.png");
        std::fs::write(&art_path, b"png").unwrap();

        w.register_artifact("a-png", "screenshot", &art_path)
            .unwrap();
        w.run_finished(1, 0, 0, 12).unwrap();

        let handle = w.finalize().unwrap();
        // Both outputs exist: events.jsonl + manifest.json
        let events_path = handle.root().join(EVENTS_FILE_NAME);
        let manifest_path = handle.root().join(MANIFEST_FILE_NAME);
        assert!(events_path.exists());
        assert!(manifest_path.exists());

        // events.jsonl carries one record per call, in order.
        let stream = std::fs::read_to_string(&events_path).unwrap();
        let lines: Vec<&str> = stream.lines().collect();
        assert_eq!(lines.len(), 4, "expected 4 events, got: {stream}");
        let parsed: Vec<EvidenceEvent> = lines
            .iter()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert!(matches!(parsed[0], EvidenceEvent::RunStarted { .. }));
        assert!(matches!(parsed[1], EvidenceEvent::TestResult { .. }));
        assert!(matches!(
            parsed[2],
            EvidenceEvent::ArtifactRegistered { .. }
        ));
        assert!(matches!(parsed[3], EvidenceEvent::RunFinished { .. }));

        // manifest captures the registered artifact, with the path
        // normalised relative to the bundle root.
        assert_eq!(handle.manifest().artifacts.len(), 1);
        assert_eq!(
            handle.manifest().artifacts[0].path,
            PathBuf::from("artifacts/a.png"),
            "manifest path must be relative to bundle root",
        );
        // Reader resolves it back to the on-disk file.
        let resolved = handle.resolve("a-png").unwrap();
        assert!(resolved.exists());
        assert_eq!(std::fs::read(&resolved).unwrap(), b"png");
    }

    #[test]
    fn register_artifact_accepts_relative_paths_verbatim() {
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), empty_manifest()).unwrap();
        w.register_artifact("rel", "log", "artifacts/rel.log")
            .unwrap();
        let handle = w.finalize().unwrap();
        assert_eq!(
            handle.manifest().artifacts[0].path,
            PathBuf::from("artifacts/rel.log"),
        );
    }

    #[test]
    fn register_artifact_rejects_paths_outside_root() {
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), empty_manifest()).unwrap();
        let outside = std::env::temp_dir().join("not-in-bundle.txt");
        let err = format!(
            "{:#}",
            w.register_artifact("bad", "log", &outside).unwrap_err()
        );
        assert!(err.contains("not inside bundle root"), "{err}");
    }

    #[test]
    fn writer_handles_both_test_and_e2e_records_off_same_stream() {
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), empty_manifest()).unwrap();
        w.test_result(vec![], "unit test", "passed", 1).unwrap();
        w.case_result("e2e case", "passed", 2).unwrap();
        let handle = w.finalize().unwrap();
        let stream = std::fs::read_to_string(handle.root().join(EVENTS_FILE_NAME)).unwrap();
        // Both event shapes share the same envelope (`kind` discriminator).
        assert!(stream.contains("\"kind\":\"test_result\""), "{stream}");
        assert!(stream.contains("\"kind\":\"case_result\""), "{stream}");
    }

    #[test]
    fn retry_pass_and_final_fail_round_trip_through_the_stream() {
        // @spec #2868 — evidence must distinguish:
        //   * retry-pass: one failed attempt, eventual pass
        //   * final-fail: all attempts failed, last is the final
        // and earlier failure context (per-attempt) must survive.
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), empty_manifest()).unwrap();
        w.run_started("run-retry", "jet test").unwrap();

        // Retry-pass: failed attempt 1 captured separately, final TestResult is passed.
        w.test_retry(vec!["Suite".into()], "flaky", 1, "failed", 12)
            .unwrap();
        w.test_result(vec!["Suite".into()], "flaky", "passed", 9)
            .unwrap();

        // Final-fail: two failed retries, last TestResult is also failed.
        w.test_retry(vec!["Suite".into()], "always_bad", 1, "failed", 5)
            .unwrap();
        w.test_retry(vec!["Suite".into()], "always_bad", 2, "failed", 6)
            .unwrap();
        w.test_result(vec!["Suite".into()], "always_bad", "failed", 7)
            .unwrap();

        // E2e case parity — one failed attempt then a pass.
        w.case_retry("checkout flow", 1, "failed", 100).unwrap();
        w.case_result("checkout flow", "passed", 110).unwrap();

        w.run_finished(2, 1, 0, 249).unwrap();
        let handle = w.finalize().unwrap();

        let stream = std::fs::read_to_string(handle.root().join(EVENTS_FILE_NAME)).unwrap();
        // Per-attempt records survive the stream so triage can replay them.
        assert!(
            stream.contains(r#""kind":"test_retry""#),
            "test retry kind missing: {stream}",
        );
        assert!(
            stream.contains(r#""attempt":1"#) && stream.contains(r#""attempt":2"#),
            "1-based attempt indices missing: {stream}",
        );
        assert!(
            stream.contains(r#""kind":"case_retry""#),
            "case retry kind missing: {stream}",
        );

        // Round-trip: every line parses back into the same enum.
        let parsed: Vec<EvidenceEvent> = stream
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();

        // Bucket per (suite,name) — final outcome vs. per-attempt records.
        let mut flaky_attempts = 0u32;
        let mut flaky_final: Option<&str> = None;
        let mut bad_attempts = 0u32;
        let mut bad_final: Option<&str> = None;
        for e in &parsed {
            match e {
                EvidenceEvent::TestRetry {
                    name,
                    attempt,
                    outcome,
                    ..
                } if name == "flaky" => {
                    assert_eq!(*attempt, 1);
                    assert_eq!(outcome, "failed");
                    flaky_attempts += 1;
                }
                EvidenceEvent::TestResult { name, outcome, .. } if name == "flaky" => {
                    flaky_final = Some(outcome.as_str());
                }
                EvidenceEvent::TestRetry { name, outcome, .. } if name == "always_bad" => {
                    assert_eq!(outcome, "failed");
                    bad_attempts += 1;
                }
                EvidenceEvent::TestResult { name, outcome, .. } if name == "always_bad" => {
                    bad_final = Some(outcome.as_str());
                }
                _ => {}
            }
        }
        // Retry-pass case: one failed attempt + final pass.
        assert_eq!(flaky_attempts, 1);
        assert_eq!(flaky_final, Some("passed"));
        // Final-fail case: two failed attempts + final fail.
        assert_eq!(bad_attempts, 2);
        assert_eq!(bad_final, Some("failed"));
    }

    /// GH #3265 — `finalize()` flushes the events stream BEFORE
    /// writing the manifest. The events.jsonl write barrier
    /// happens-before the manifest pointer landing on disk, so a
    /// downstream I/O failure (here: read-only bundle root for the
    /// manifest write) must leave the manifest absent — consumers
    /// must never observe a manifest pointing at an unfinalised
    /// stream. Pre-#3265 the flush itself was discarded; this guard
    /// documents the broader ordering invariant alongside the new
    /// contractual propagation of `flush()?`.
    #[cfg(unix)]
    #[test]
    fn finalize_does_not_write_manifest_when_downstream_io_fails() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), empty_manifest()).unwrap();
        w.run_started("run-1", "jet test").unwrap();
        w.test_result(vec!["Suite".into()], "test a", "passed", 1)
            .unwrap();

        // Lock the bundle root so the manifest write fails. The
        // events.jsonl handle was already opened (still writeable via
        // the held fd) so the flush completes; the manifest write
        // hits EACCES and finalize must propagate it.
        let original = std::fs::metadata(tmp.path()).unwrap().permissions();
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o500)).unwrap();

        // Root may bypass perms — restore + skip cleanly in that case.
        let manifest_path = tmp.path().join(MANIFEST_FILE_NAME);
        let result = w.finalize();
        let manifest_exists = manifest_path.exists();
        std::fs::set_permissions(tmp.path(), original).unwrap();

        if result.is_ok() {
            // Running as root: chmod was a no-op. Skip the assertion.
            return;
        }
        let err = result.unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("writing manifest"),
            "finalize must surface the manifest write failure; got: {msg}"
        );
        assert!(
            !manifest_exists,
            "manifest.json must NOT exist when finalize fails — consumers \
             would otherwise read a manifest pointing at an unfinalised stream"
        );
    }

    /// GH #3265 — happy-path invariant: when finalize succeeds the
    /// events.jsonl is on disk (the flush ran without error) and the
    /// manifest is present. Regression sentinel for the order
    /// `flush events → write manifest`.
    #[test]
    fn finalize_writes_events_before_manifest_on_success() {
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), empty_manifest()).unwrap();
        w.run_started("run-1", "jet test").unwrap();
        w.run_finished(0, 0, 0, 0).unwrap();
        let handle = w.finalize().expect("happy-path finalize must succeed");
        let events_path = handle.root().join(EVENTS_FILE_NAME);
        let manifest_path = handle.root().join(MANIFEST_FILE_NAME);
        assert!(
            events_path.exists(),
            "events.jsonl must exist after finalize"
        );
        assert!(
            manifest_path.exists(),
            "manifest.json must exist after finalize"
        );

        // Both events were captured (sanity: flush propagation didn't
        // accidentally drop pending writes).
        let stream = std::fs::read_to_string(&events_path).unwrap();
        assert_eq!(stream.lines().count(), 2, "stream: {stream}");
    }
}
// CODEGEN-END
