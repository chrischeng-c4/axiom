#![cfg(unix)]

//! Public durability contract for inherited checkpoint files.
//!
//! A first child writes a base generation with one unchanged collection and one
//! changed collection. Its next incremental checkpoint changes only the latter.
//! The test makes the public `SyncFile` failure seam reject the unchanged
//! collection's known hard-linked base segment.
//! A correct publication carries proof that the old published inode is still
//! that inode, so it skips that one old payload and publishes the new delta.
//! It then rejects the new `_generation.json` at the same seam and proves the
//! old `CURRENT` and cold data remain usable before retrying the checkpoint.
//!
//! A second child starts with one collection and two fields.  It writes four
//! deltas only for `merge`, which requests one real background merge.  Its
//! `BeforePublish` observer lets the injector reject the unchanged `stable`
//! base segment.  After the inherited proof succeeds, the child requests a
//! second merge.  This time the injector finds a catalogued `merge` component
//! with a different inode in the private generation and rejects it.  That
//! fresh compaction output must still block publication and leave the prior
//! `CURRENT` readable.
//!
//! The parent runs each behavior body in a child with a parent-owned temporary
//! directory.  The watchdog kills and reaps only a stuck child before its
//! workspace disappears.  It is cleanup safety, not a latency budget.
//!
//! # Facets
//!
//! - Behavior: `inherited_checkpoint_file_sync.rs:657-727` requires an
//!   incremental checkpoint to retain the unchanged collection hard link,
//!   publish its changed value, and cold-open it. `:739-803` makes a fresh manifest
//!   `SyncFile` failure keep the earlier `CURRENT` readable before a retry.
//!   `:832-914` drives four public deltas through one background merge, then
//!   requires the unchanged field hard link, live exact search, and cold
//!   recovery. It covers `libs/storage-durable/src/generation.rs:375-405,
//!   663-709` and `apps/lumen/src/segment_background_merge.rs:711-779`.
//! - Security: the only boundary this change reaches is the process-written
//!   staged generation and its `CURRENT` pointer at
//!   `libs/storage-durable/src/generation.rs:375-405,773-813`. The fresh-file
//!   faults at `inherited_checkpoint_file_sync.rs:739-773,939-973` assert
//!   pre-publication failure keeps the exact old pointer and a cold reopen
//!   readable. Existing `apps/lumen/e2e/segment_startup_fail_closed_e2e.rs:
//!   798-811,816-842,1119-1148` remains the gate for hostile symlink,
//!   unknown-root, and unpointed-generation inputs; this change adds no caller
//!   path, identifier, or network input.
//! - Performance: `apps/lumen/ROADMAP.md:73-78` promises, verbatim, "One
//!   background merge runs per process. Four delta segments request a merge;
//!   no field may retain more than 16." `:832-858` creates exactly four
//!   deltas and requires that public merge to finish. No current document
//!   gives `generation.rs:663-709` a sync-call or latency budget, so this
//!   contract does not invent one; its bounded waits only protect cleanup.
//!
//! # Root negative control
//!
//! After the repair, restore the pre-repair behavior in the shared durable
//! helper so every hard-linked staged file again calls `SyncFile` at
//! `libs/storage-durable/src/generation.rs:682-691`. Both inherited-hit
//! assertions at `:657` and `:852` must fail. Do not redirect the injector to
//! a fresh file: `:739-773` and `:939-973` deliberately prove fresh manifest
//! and fresh compaction files still stop publication. Restore the corrected
//! source SHA before another gate.
//!
//! Gate: cargo test -p lumen --test inherited_checkpoint_file_sync -- --nocapture.
//! Full declared behavior gate: cargo test -p lumen.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs::{self, File};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use lumen::segment_rdb::{MergeObserver, MergePhase, SegmentRdbStore};
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest, QueryNode,
    SearchRequest, TermQuery,
};
use serde_json::Value;
use storage_durable::{CommitStep, FailureInjector, FailurePoint};

const INCREMENTAL_STABLE_COLLECTION: &str = "inherited-sync-stable";
const INCREMENTAL_CHANGED_COLLECTION: &str = "inherited-sync-changed";
const MERGE_COLLECTION: &str = "inherited-sync-merge";
const STABLE_FIELD: &str = "stable";
const CHANGE_FIELD: &str = "changed";
const MERGE_FIELD: &str = "merge";
const EXTERNAL_ID: &str = "inherited-sync-doc";
const BASE_SEQUENCE: u64 = 81_000;
const INCREMENTAL_SEQUENCE: u64 = BASE_SEQUENCE + 1;
const RETRY_SEQUENCE: u64 = BASE_SEQUENCE + 2;
const FIRST_MERGE_DELTA_COUNT: u64 = 4;
const SECOND_MERGE_FINAL_SEQUENCE: u64 = BASE_SEQUENCE + 8;
const READY_WATCHDOG: Duration = Duration::from_secs(30);
const MERGE_WATCHDOG: Duration = Duration::from_secs(30);
const CHILD_WATCHDOG: Duration = Duration::from_secs(120);
const POLL_INTERVAL: Duration = Duration::from_millis(20);
const CURRENT_FILE: &str = "CURRENT";
const GENERATION_MANIFEST: &str = "_generation.json";
const CHILD_MODE_ENV: &str = "LUMEN_INHERITED_SYNC_CHILD";
const CHILD_CASE_ENV: &str = "LUMEN_INHERITED_SYNC_CASE";
const CHILD_ROOT_ENV: &str = "LUMEN_INHERITED_SYNC_ROOT";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_INHERITED_SYNC_HANDSHAKE";
const INCREMENTAL_CASE: &str = "incremental";
const MERGE_CASE: &str = "merge";
const INCREMENTAL_TEST: &str = "incremental_checkpoint_skips_only_durable_inherited_payloads";
const MERGE_TEST: &str = "one_field_background_merge_skips_only_durable_inherited_payloads";

#[derive(Clone, Debug)]
enum SyncMode {
    Off,
    Inherited(PathBuf),
    FreshManifest,
    FreshCompaction {
        checkpoint_root: PathBuf,
        predecessor: PathBuf,
        collection: String,
        field: String,
    },
}

impl Default for SyncMode {
    fn default() -> Self {
        Self::Off
    }
}

/// The existing durable-library injector sees the exact relative file that
/// would be synced. It never guesses an output name: a fresh compaction target
/// is selected from the staged catalog and verified to have a different inode
/// from the current generation before this test injects its failure.
#[derive(Default)]
struct SelectiveSyncFailure {
    mode: Mutex<SyncMode>,
    points: Mutex<Vec<FailurePoint>>,
    inherited_hits: AtomicUsize,
    manifest_hits: AtomicUsize,
    fresh_compaction_hits: AtomicUsize,
}

impl SelectiveSyncFailure {
    fn arm_inherited(&self, relative: PathBuf) {
        self.inherited_hits.store(0, Ordering::Release);
        *self.mode.lock().expect("inherited SyncFile mode mutex") = SyncMode::Inherited(relative);
    }

    fn arm_fresh_manifest(&self) {
        self.manifest_hits.store(0, Ordering::Release);
        *self.mode.lock().expect("manifest SyncFile mode mutex") = SyncMode::FreshManifest;
    }

    fn arm_fresh_compaction(
        &self,
        checkpoint_root: PathBuf,
        predecessor: PathBuf,
        collection: &str,
        field: &str,
    ) {
        self.fresh_compaction_hits.store(0, Ordering::Release);
        *self.mode.lock().expect("compaction SyncFile mode mutex") = SyncMode::FreshCompaction {
            checkpoint_root,
            predecessor,
            collection: collection.to_owned(),
            field: field.to_owned(),
        };
    }

    fn disarm(&self) {
        *self.mode.lock().expect("disarm SyncFile mode mutex") = SyncMode::Off;
    }

    fn inherited_hits(&self) -> usize {
        self.inherited_hits.load(Ordering::Acquire)
    }

    fn manifest_hits(&self) -> usize {
        self.manifest_hits.load(Ordering::Acquire)
    }

    fn fresh_compaction_hits(&self) -> usize {
        self.fresh_compaction_hits.load(Ordering::Acquire)
    }

    fn point_count(&self) -> usize {
        self.points
            .lock()
            .expect("recorded SyncFile points mutex")
            .len()
    }

    fn assert_successful_publication_steps(&self, after: usize, context: &str) {
        let points = self.points.lock().expect("recorded SyncFile points mutex");
        let points = &points[after..];
        for step in [
            CommitStep::SyncFile,
            CommitStep::SyncDirectory,
            CommitStep::SyncRootAfterGeneration,
            CommitStep::SyncCurrentTemp,
            CommitStep::SyncRootAfterCurrent,
        ] {
            assert!(
                points.iter().any(|point| point.step == step),
                "{context}: successful publication must retain {step:?}; points={points:?}",
            );
        }
    }
}

impl FailureInjector for SelectiveSyncFailure {
    fn check(&self, point: &FailurePoint) -> io::Result<()> {
        self.points
            .lock()
            .expect("record SyncFile point mutex")
            .push(point.clone());
        if point.step != CommitStep::SyncFile {
            return Ok(());
        }
        let mode = self.mode.lock().expect("read SyncFile mode mutex").clone();
        match mode {
            SyncMode::Off => Ok(()),
            SyncMode::Inherited(relative) if point.relative_path == relative => {
                self.inherited_hits.fetch_add(1, Ordering::AcqRel);
                Err(io::Error::other(format!(
                    "injected inherited SyncFile failure for {}",
                    point.relative_path.display()
                )))
            }
            SyncMode::FreshManifest
                if point.relative_path.as_path() == Path::new(GENERATION_MANIFEST) =>
            {
                self.manifest_hits.fetch_add(1, Ordering::AcqRel);
                Err(io::Error::other(
                    "injected fresh generation-manifest SyncFile failure",
                ))
            }
            SyncMode::FreshCompaction {
                checkpoint_root,
                predecessor,
                collection,
                field,
            } => {
                if fresh_compaction_component(
                    &checkpoint_root,
                    &predecessor,
                    &collection,
                    &field,
                    &point.relative_path,
                )? {
                    self.fresh_compaction_hits.fetch_add(1, Ordering::AcqRel);
                    Err(io::Error::other(format!(
                        "injected fresh compaction SyncFile failure for {}",
                        point.relative_path.display()
                    )))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

#[derive(Default)]
struct NoopMergeObserver;

impl MergeObserver for NoopMergeObserver {
    fn observe(&self, _: MergePhase) -> io::Result<()> {
        Ok(())
    }
}

/// A bounded public merge observation. The hold occurs before the final staged
/// generation exists, so the test can arm a precise durable `SyncFile` target
/// before the library begins its commit traversal.
#[derive(Default)]
struct PauseBeforePublish {
    controls: Mutex<VecDeque<(mpsc::SyncSender<()>, mpsc::Receiver<()>)>>,
}

impl PauseBeforePublish {
    fn arm(&self) -> (mpsc::Receiver<()>, ReleaseSignal) {
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::channel();
        self.controls
            .lock()
            .expect("merge pause controls mutex")
            .push_back((entered_tx, release_rx));
        (entered_rx, ReleaseSignal(Some(release_tx)))
    }
}

impl MergeObserver for PauseBeforePublish {
    fn observe(&self, phase: MergePhase) -> io::Result<()> {
        if phase != MergePhase::BeforePublish {
            return Ok(());
        }
        let Some((entered, release)) = self
            .controls
            .lock()
            .expect("merge pause controls mutex")
            .pop_front()
        else {
            return Ok(());
        };
        entered.send(()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "merge BeforePublish readiness receiver dropped",
            )
        })?;
        release.recv().map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "merge BeforePublish release sender dropped",
            )
        })?;
        Ok(())
    }
}

/// Releases a paused merge on normal completion and panic unwinding.
struct ReleaseSignal(Option<mpsc::Sender<()>>);

impl ReleaseSignal {
    fn release(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

impl Drop for ReleaseSignal {
    fn drop(&mut self) {
        self.release();
    }
}

/// Kills and reaps a hung child before its parent-owned workspace disappears.
struct ChildCleanup(Option<Child>);

impl Drop for ChildCleanup {
    fn drop(&mut self) {
        let Some(child) = self.0.as_mut() else {
            return;
        };
        if !matches!(child.try_wait(), Ok(Some(_))) {
            let _ = child.kill();
        }
        let _ = child.wait();
    }
}

fn keyword_spec() -> FieldSpec {
    FieldSpec {
        field_type: FieldType::Keyword,
        analyzer: None,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    }
}

fn schema(fields: &[&str]) -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: fields
            .iter()
            .map(|field| ((*field).to_owned(), keyword_spec()))
            .collect::<BTreeMap<_, _>>(),
    }
}

fn index_keyword(engine: &Engine, collection: &str, field: &str, value: &str) {
    engine
        .index(
            collection,
            IndexRequest {
                items: vec![IndexItem {
                    external_id: EXTERNAL_ID.to_owned(),
                    field: field.to_owned(),
                    value: FieldValue::String(value.to_owned()),
                    version: None,
                }],
                request_id: None,
            },
        )
        .expect("apply valid public Keyword update");
}

fn exact_ids(engine: &Engine, collection: &str, field: &str, value: &str) -> Vec<String> {
    let mut ids = engine
        .search(
            collection,
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: field.to_owned(),
                    value: FieldValue::String(value.to_owned()),
                }),
                limit: 10,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("run public exact Keyword query")
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn assert_exact(
    engine: &Engine,
    collection: &str,
    field: &str,
    value: &str,
    expected: &[&str],
    phase: &str,
) {
    let mut expected = expected
        .iter()
        .map(|id| (*id).to_owned())
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(
        exact_ids(engine, collection, field, value),
        expected,
        "{phase}: exact public Keyword query must retain the expected IDs for {collection}/{field}/{value}",
    );
}

fn current_name(root: &Path) -> String {
    fs::read_to_string(root.join(CURRENT_FILE))
        .expect("read public CURRENT")
        .strip_prefix("generation:")
        .expect("CURRENT names a generation")
        .trim()
        .to_owned()
}

fn read_manifest(generation: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(generation.join(GENERATION_MANIFEST)).expect("read generation manifest"),
    )
    .expect("decode generation manifest")
}

fn collection_catalog<'a>(manifest: &'a Value, collection: &str) -> &'a Value {
    manifest["collections"]
        .as_array()
        .expect("generation manifest collections")
        .iter()
        .find(|entry| entry["collection_id"].as_str() == Some(collection))
        .unwrap_or_else(|| panic!("generation manifest contains collection {collection}"))
}

fn field_component_paths(manifest: &Value, collection: &str, field: &str) -> BTreeSet<PathBuf> {
    let mut paths = BTreeSet::new();
    for segment in collection_catalog(manifest, collection)["segments"]
        .as_array()
        .expect("collection catalog segments")
        .iter()
        .filter(|segment| segment["field"].as_str() == Some(field))
    {
        paths.insert(PathBuf::from(
            segment["path"].as_str().expect("catalogued segment path"),
        ));
        if let Some(rows) = segment["local_rows"]["path"].as_str() {
            paths.insert(PathBuf::from(rows));
        }
    }
    assert!(
        !paths.is_empty(),
        "catalogue must contain at least one component for {collection}/{field}",
    );
    paths
}

fn base_field_path(manifest: &Value, collection: &str, field: &str) -> PathBuf {
    collection_catalog(manifest, collection)["segments"]
        .as_array()
        .expect("collection catalog segments")
        .iter()
        .find(|segment| {
            segment["field"].as_str() == Some(field) && segment["kind"].as_str() == Some("base")
        })
        .map(|segment| PathBuf::from(segment["path"].as_str().expect("base segment path")))
        .unwrap_or_else(|| {
            panic!("catalogue must contain a base component for {collection}/{field}")
        })
}

fn same_regular_inode(left: &Path, right: &Path) -> io::Result<bool> {
    let left = match fs::symlink_metadata(left) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let right = fs::symlink_metadata(right)?;
    if !left.is_file()
        || left.file_type().is_symlink()
        || !right.is_file()
        || right.file_type().is_symlink()
    {
        return Err(io::Error::other(
            "compared checkpoint components must be real regular files",
        ));
    }
    Ok(left.dev() == right.dev() && left.ino() == right.ino())
}

fn assert_hardlinked(left: &Path, right: &Path, context: &str) {
    assert!(
        same_regular_inode(left, right).expect("inspect hard-link identity"),
        "{context}: immutable inherited payload must preserve its inode: {} <> {}",
        left.display(),
        right.display(),
    );
    assert!(
        fs::symlink_metadata(right)
            .expect("inspect inherited link count")
            .nlink()
            >= 2,
        "{context}: inherited payload must retain at least two hard links: {}",
        right.display(),
    );
}

/// Return the one final private generation that is already catalogued at the
/// durable-library `SyncFile` callback. A compaction scratch directory has no
/// generation manifest, so it cannot satisfy this predicate.
fn final_staging_generation(root: &Path) -> io::Result<PathBuf> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with(".stage-gen-") {
            continue;
        }
        let candidate = entry.path();
        let manifest = candidate.join(GENERATION_MANIFEST);
        let metadata = match fs::symlink_metadata(&manifest) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if metadata.is_file() && !metadata.file_type().is_symlink() {
            candidates.push(candidate);
        }
    }
    candidates.sort();
    match candidates.as_slice() {
        [stage] => Ok(stage.clone()),
        _ => Err(io::Error::other(format!(
            "expected exactly one private generation with {GENERATION_MANIFEST}, found {candidates:?}"
        ))),
    }
}

fn fresh_compaction_component(
    checkpoint_root: &Path,
    predecessor: &Path,
    collection: &str,
    field: &str,
    relative: &Path,
) -> io::Result<bool> {
    let staged = final_staging_generation(checkpoint_root)?;
    let manifest: Value = serde_json::from_slice(&fs::read(staged.join(GENERATION_MANIFEST))?)
        .map_err(io::Error::other)?;
    let candidates = field_component_paths(&manifest, collection, field);
    if !candidates.contains(relative) {
        return Ok(false);
    }
    Ok(!same_regular_inode(
        &predecessor.join(relative),
        &staged.join(relative),
    )?)
}

fn wait_receiver<T>(receiver: mpsc::Receiver<T>, timeout: Duration, phase: &str) -> T {
    receiver.recv_timeout(timeout).unwrap_or_else(|error| {
        panic!("{phase} did not finish before bounded cleanup watchdog {timeout:?}: {error}")
    })
}

fn open_cold(store: &SegmentRdbStore, expected_sequence: u64, phase: &str) -> Arc<Engine> {
    let loaded = store
        .load_current_generation()
        .unwrap_or_else(|error| panic!("{phase}: cold-open CURRENT: {error:#}"))
        .unwrap_or_else(|| panic!("{phase}: CURRENT names a generation"));
    assert_eq!(
        loaded.sequence, expected_sequence,
        "{phase}: cold CURRENT must retain its expected watermark",
    );
    loaded.engine
}

fn run_incremental_child(root: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, INCREMENTAL_CASE).expect("record incremental child entry");
    let injector = Arc::new(SelectiveSyncFailure::default());
    let store = SegmentRdbStore::with_failure_injector_and_merge_observer(
        &root,
        injector.clone(),
        Arc::new(NoopMergeObserver),
    )
    .expect("open public failure-injected segment store");
    let engine = Arc::new(Engine::new());
    engine
        .create_collection(INCREMENTAL_STABLE_COLLECTION, schema(&[STABLE_FIELD]))
        .expect("create public unchanged stable Keyword collection");
    engine
        .create_collection(INCREMENTAL_CHANGED_COLLECTION, schema(&[CHANGE_FIELD]))
        .expect("create public changed Keyword collection");
    index_keyword(
        &engine,
        INCREMENTAL_STABLE_COLLECTION,
        STABLE_FIELD,
        "stable-v1",
    );
    index_keyword(
        &engine,
        INCREMENTAL_CHANGED_COLLECTION,
        CHANGE_FIELD,
        "changed-v1",
    );
    store
        .save(&engine, BASE_SEQUENCE)
        .expect("publish public two-collection base generation");

    let base_name = current_name(&root);
    let base_generation = root.join(&base_name);
    let base_manifest = read_manifest(&base_generation);
    let inherited = base_field_path(&base_manifest, INCREMENTAL_STABLE_COLLECTION, STABLE_FIELD);

    index_keyword(
        &engine,
        INCREMENTAL_CHANGED_COLLECTION,
        CHANGE_FIELD,
        "changed-v2",
    );
    let publication_events = injector.point_count();
    injector.arm_inherited(inherited.clone());
    let incremental = store.save(&engine, INCREMENTAL_SEQUENCE);
    assert_eq!(
        injector.inherited_hits(),
        0,
        "an incremental checkpoint must not SyncFile an already durable unchanged inherited payload: {}",
        inherited.display(),
    );
    incremental.expect("incremental checkpoint must publish its fresh changed field");
    injector.disarm();
    injector.assert_successful_publication_steps(
        publication_events,
        "incremental inherited-file publication",
    );

    let incremental_name = current_name(&root);
    let incremental_generation = root.join(&incremental_name);
    let incremental_manifest = read_manifest(&incremental_generation);
    let inherited_after = base_field_path(
        &incremental_manifest,
        INCREMENTAL_STABLE_COLLECTION,
        STABLE_FIELD,
    );
    assert_eq!(
        inherited_after, inherited,
        "the unchanged field must retain the catalogued relative component path",
    );
    assert_hardlinked(
        &base_generation.join(&inherited),
        &incremental_generation.join(&inherited_after),
        "incremental checkpoint",
    );
    assert_exact(
        &engine,
        INCREMENTAL_STABLE_COLLECTION,
        STABLE_FIELD,
        "stable-v1",
        &[EXTERNAL_ID],
        "live incremental",
    );
    assert_exact(
        &engine,
        INCREMENTAL_CHANGED_COLLECTION,
        CHANGE_FIELD,
        "changed-v1",
        &[],
        "live incremental",
    );
    assert_exact(
        &engine,
        INCREMENTAL_CHANGED_COLLECTION,
        CHANGE_FIELD,
        "changed-v2",
        &[EXTERNAL_ID],
        "live incremental",
    );
    let incremental_cold = open_cold(&store, INCREMENTAL_SEQUENCE, "incremental cold reopen");
    assert_exact(
        &incremental_cold,
        INCREMENTAL_STABLE_COLLECTION,
        STABLE_FIELD,
        "stable-v1",
        &[EXTERNAL_ID],
        "incremental cold reopen",
    );
    assert_exact(
        &incremental_cold,
        INCREMENTAL_CHANGED_COLLECTION,
        CHANGE_FIELD,
        "changed-v2",
        &[EXTERNAL_ID],
        "incremental cold reopen",
    );

    index_keyword(
        &engine,
        INCREMENTAL_CHANGED_COLLECTION,
        CHANGE_FIELD,
        "changed-v3",
    );
    let current_before_fresh_failure =
        fs::read(root.join(CURRENT_FILE)).expect("read CURRENT before fresh manifest fault");
    injector.arm_fresh_manifest();
    let fresh_failure = store.save(&engine, RETRY_SEQUENCE);
    assert!(
        fresh_failure.is_err(),
        "a fresh generation manifest SyncFile fault must stop publication before CURRENT",
    );
    assert_eq!(
        injector.manifest_hits(),
        1,
        "the fresh generation manifest must still pass the SyncFile durability seam",
    );
    assert_eq!(
        fs::read(root.join(CURRENT_FILE)).expect("read CURRENT after fresh manifest fault"),
        current_before_fresh_failure,
        "fresh pre-publication SyncFile failure must retain the exact old CURRENT bytes",
    );
    let old_cold = open_cold(
        &store,
        INCREMENTAL_SEQUENCE,
        "fresh fault old CURRENT reopen",
    );
    assert_exact(
        &old_cold,
        INCREMENTAL_CHANGED_COLLECTION,
        CHANGE_FIELD,
        "changed-v2",
        &[EXTERNAL_ID],
        "fresh fault old CURRENT reopen",
    );
    assert_exact(
        &old_cold,
        INCREMENTAL_CHANGED_COLLECTION,
        CHANGE_FIELD,
        "changed-v3",
        &[],
        "fresh fault old CURRENT reopen",
    );

    injector.disarm();
    store
        .save(&engine, RETRY_SEQUENCE)
        .expect("retry after fresh manifest fault must publish the retained cut");
    let retried_cold = open_cold(&store, RETRY_SEQUENCE, "fresh fault retry cold reopen");
    assert_exact(
        &retried_cold,
        INCREMENTAL_STABLE_COLLECTION,
        STABLE_FIELD,
        "stable-v1",
        &[EXTERNAL_ID],
        "fresh fault retry cold reopen",
    );
    assert_exact(
        &retried_cold,
        INCREMENTAL_CHANGED_COLLECTION,
        CHANGE_FIELD,
        "changed-v2",
        &[],
        "fresh fault retry cold reopen",
    );
    assert_exact(
        &retried_cold,
        INCREMENTAL_CHANGED_COLLECTION,
        CHANGE_FIELD,
        "changed-v3",
        &[EXTERNAL_ID],
        "fresh fault retry cold reopen",
    );
}

fn merge_value(round: u64) -> String {
    format!("merge-v{round}")
}

fn run_merge_child(root: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, MERGE_CASE).expect("record merge child entry");
    let injector = Arc::new(SelectiveSyncFailure::default());
    let observer = Arc::new(PauseBeforePublish::default());
    let store = Arc::new(
        SegmentRdbStore::with_failure_injector_and_merge_observer(
            &root,
            injector.clone(),
            observer.clone(),
        )
        .expect("open public observed segment store"),
    );
    let engine = Arc::new(Engine::new());
    engine
        .create_collection(MERGE_COLLECTION, schema(&[STABLE_FIELD, MERGE_FIELD]))
        .expect("create public merge Keyword collection");
    index_keyword(&engine, MERGE_COLLECTION, STABLE_FIELD, "stable-v1");
    index_keyword(&engine, MERGE_COLLECTION, MERGE_FIELD, "merge-v0");
    store
        .save(&engine, BASE_SEQUENCE)
        .expect("publish public merge base generation");

    let (first_entered, mut first_release) = observer.arm();
    for round in 1..=FIRST_MERGE_DELTA_COUNT {
        index_keyword(&engine, MERGE_COLLECTION, MERGE_FIELD, &merge_value(round));
        store
            .save(&engine, BASE_SEQUENCE + round)
            .expect("publish public sparse merge delta");
    }
    wait_receiver(
        first_entered,
        READY_WATCHDOG,
        "four public deltas reaching one real merge BeforePublish pause",
    );
    let predecessor_name = current_name(&root);
    let predecessor = root.join(&predecessor_name);
    let predecessor_manifest = read_manifest(&predecessor);
    let inherited = base_field_path(&predecessor_manifest, MERGE_COLLECTION, STABLE_FIELD);
    let merge_publication_events = injector.point_count();
    injector.arm_inherited(inherited.clone());
    first_release.release();
    let first_merge = store.wait_for_merges(MERGE_WATCHDOG);
    assert_eq!(
        injector.inherited_hits(),
        0,
        "a one-field background merge must not SyncFile its unchanged durable inherited payload: {}",
        inherited.display(),
    );
    first_merge.expect("one-field background merge must publish after skipping inherited payload");
    injector.disarm();
    injector.assert_successful_publication_steps(
        merge_publication_events,
        "one-field background merge publication",
    );

    let merged_name = current_name(&root);
    let merged_generation = root.join(&merged_name);
    let merged_manifest = read_manifest(&merged_generation);
    let inherited_after = base_field_path(&merged_manifest, MERGE_COLLECTION, STABLE_FIELD);
    assert_eq!(
        inherited_after, inherited,
        "one-field merge must retain the unchanged field's catalogued component",
    );
    assert_hardlinked(
        &predecessor.join(&inherited),
        &merged_generation.join(&inherited_after),
        "one-field background merge",
    );
    assert_exact(
        &engine,
        MERGE_COLLECTION,
        STABLE_FIELD,
        "stable-v1",
        &[EXTERNAL_ID],
        "live first merge",
    );
    assert_exact(
        &engine,
        MERGE_COLLECTION,
        MERGE_FIELD,
        &merge_value(FIRST_MERGE_DELTA_COUNT),
        &[EXTERNAL_ID],
        "live first merge",
    );
    let first_merge_cold = open_cold(
        &store,
        BASE_SEQUENCE + FIRST_MERGE_DELTA_COUNT,
        "first merge cold reopen",
    );
    assert_exact(
        &first_merge_cold,
        MERGE_COLLECTION,
        STABLE_FIELD,
        "stable-v1",
        &[EXTERNAL_ID],
        "first merge cold reopen",
    );
    assert_exact(
        &first_merge_cold,
        MERGE_COLLECTION,
        MERGE_FIELD,
        &merge_value(FIRST_MERGE_DELTA_COUNT),
        &[EXTERNAL_ID],
        "first merge cold reopen",
    );

    let (second_entered, mut second_release) = observer.arm();
    for round in (FIRST_MERGE_DELTA_COUNT + 1)..=8 {
        index_keyword(&engine, MERGE_COLLECTION, MERGE_FIELD, &merge_value(round));
        store
            .save(&engine, BASE_SEQUENCE + round)
            .expect("publish public later sparse merge delta");
    }
    wait_receiver(
        second_entered,
        READY_WATCHDOG,
        "later public deltas reaching fresh-output merge BeforePublish pause",
    );
    let current_before_fresh_failure =
        fs::read(root.join(CURRENT_FILE)).expect("read CURRENT before fresh compaction fault");
    let fresh_predecessor = root.join(current_name(&root));
    injector.arm_fresh_compaction(
        root.clone(),
        fresh_predecessor,
        MERGE_COLLECTION,
        MERGE_FIELD,
    );
    second_release.release();
    let fresh_merge_failure = store.wait_for_merges(MERGE_WATCHDOG);
    assert!(
        fresh_merge_failure.is_err(),
        "a fresh compaction output SyncFile fault must stop merge publication before CURRENT",
    );
    assert_eq!(
        injector.fresh_compaction_hits(),
        1,
        "the selected fresh catalogued compaction component must still pass SyncFile",
    );
    assert_eq!(
        fs::read(root.join(CURRENT_FILE)).expect("read CURRENT after fresh compaction fault"),
        current_before_fresh_failure,
        "fresh compaction SyncFile failure must retain the exact old CURRENT bytes",
    );
    let old_cold = open_cold(
        &store,
        SECOND_MERGE_FINAL_SEQUENCE,
        "fresh compaction fault old CURRENT reopen",
    );
    assert_exact(
        &old_cold,
        MERGE_COLLECTION,
        STABLE_FIELD,
        "stable-v1",
        &[EXTERNAL_ID],
        "fresh compaction fault old CURRENT reopen",
    );
    assert_exact(
        &old_cold,
        MERGE_COLLECTION,
        MERGE_FIELD,
        &merge_value(8),
        &[EXTERNAL_ID],
        "fresh compaction fault old CURRENT reopen",
    );
    assert_exact(
        &engine,
        MERGE_COLLECTION,
        MERGE_FIELD,
        &merge_value(8),
        &[EXTERNAL_ID],
        "live state after fresh compaction fault",
    );
}

fn run_isolated_case(case: &str, test_name: &str) {
    let workspace = tempfile::tempdir().expect("create parent-owned inherited-sync workspace");
    let child_root = workspace.path().join("segments");
    let child_tmp = workspace.path().join("child-tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned inherited-sync child TMPDIR");
    let handshake = workspace.path().join("entered-case");
    let stdout_path = workspace.path().join("child.stdout");
    let stderr_path = workspace.path().join("child.stderr");
    let executable = std::env::current_exe().expect("locate inherited-sync test executable");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, "1")
        .env(CHILD_CASE_ENV, case)
        .env(CHILD_ROOT_ENV, &child_root)
        .env(CHILD_HANDSHAKE_ENV, &handshake)
        .env("TMPDIR", &child_tmp)
        .env("TEMP", &child_tmp)
        .env("TMP", &child_tmp)
        .arg(test_name)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::from(
            File::create(&stdout_path).expect("create inherited-sync child stdout"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("create inherited-sync child stderr"),
        ))
        .spawn()
        .expect("spawn isolated inherited-sync child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + CHILD_WATCHDOG;
    loop {
        match child
            .0
            .as_mut()
            .expect("inherited-sync child remains owned until exit")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                let mut raw = child.0.take().expect("timed-out child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("reap killed inherited-sync child");
                let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
                let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
                panic!(
                    "inherited-sync child exceeded cleanup watchdog {CHILD_WATCHDOG:?}; status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated inherited-sync child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited inherited-sync child remains owned")
        .wait()
        .expect("wait for exited inherited-sync child");
    let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
    let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated inherited-sync child did not enter {test_name}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, case,
        "isolated child must enter its intended behavior body"
    );
    assert!(
        status.success(),
        "isolated inherited-sync child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[test]
fn incremental_checkpoint_skips_only_durable_inherited_payloads() {
    if std::env::var_os(CHILD_MODE_ENV).is_some()
        && std::env::var(CHILD_CASE_ENV).ok().as_deref() == Some(INCREMENTAL_CASE)
    {
        let root = PathBuf::from(
            std::env::var_os(CHILD_ROOT_ENV)
                .expect("incremental child must receive parent-owned segment root"),
        );
        let handshake = PathBuf::from(
            std::env::var_os(CHILD_HANDSHAKE_ENV)
                .expect("incremental child must receive parent-owned handshake path"),
        );
        run_incremental_child(root, handshake);
        return;
    }
    run_isolated_case(INCREMENTAL_CASE, INCREMENTAL_TEST);
}

#[test]
fn one_field_background_merge_skips_only_durable_inherited_payloads() {
    if std::env::var_os(CHILD_MODE_ENV).is_some()
        && std::env::var(CHILD_CASE_ENV).ok().as_deref() == Some(MERGE_CASE)
    {
        let root = PathBuf::from(
            std::env::var_os(CHILD_ROOT_ENV)
                .expect("merge child must receive parent-owned segment root"),
        );
        let handshake = PathBuf::from(
            std::env::var_os(CHILD_HANDSHAKE_ENV)
                .expect("merge child must receive parent-owned handshake path"),
        );
        run_merge_child(root, handshake);
        return;
    }
    run_isolated_case(MERGE_CASE, MERGE_TEST);
}
