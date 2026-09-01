// CODEGEN-BEGIN
//! Durable segment-checkpoint generations.
//!
//! A checkpoint stores one directory per collection. Each collection contains
//! mmap segments and `_schema.json`. New checkpoints also contain one
//! top-level `_generation.json` manifest. The shared `storage-durable`
//! generation store fsyncs the complete tree and changes `CURRENT` atomically.
//! `CURRENT` is the only source of truth. A complete but unpointed directory is
//! never selected during restart.
//!
//! New directory names are `gen-<seq>-rev-<revision>`. A same-sequence save
//! creates a new immutable revision. A background save below the active
//! sequence is a no-op. Exact 0.4.28 `gen-<seq>` directories remain readable:
//! on the first 0.4.29 restart, Lumen validates the exact highest legacy
//! directory and then writes `CURRENT` once. It never falls back from a corrupt
//! highest legacy directory.

use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use storage_durable::{
    CurrentReadErrorKind, CurrentTarget, FailureInjector, GenerationName, GenerationStore,
    NoFailures, StagedGeneration,
};

use crate::storage::Engine;

const GENERATION_MANIFEST_FILE: &str = "_generation.json";
const GENERATION_MANIFEST_SCHEMA_VERSION: u32 = 1;
const CHECKPOINT_SCHEMA_FILE: &str = "_schema.json";
const CURRENT_FILE: &str = "CURRENT";
const CURRENT_TEMP_FILE: &str = "CURRENT.tmp";
const AOF_FILE: &str = "aof.log";
const AOF_COMPACT_TEMP_FILE: &str = "aof.log.compact.tmp";
// The shipped image pre-populates its declared volume with this inert regular
// file so Docker recognizes a non-empty data directory. It carries no Lumen
// state and is part of a semantically new root.
const CONTAINER_VOLUME_SEED_FILE: &str = ".lumen-volume-seed";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SegmentGenerationManifest {
    schema_version: u32,
    sequence: u64,
    revision: u64,
    previous: Option<String>,
}

#[derive(Debug, Clone)]
struct GenerationRecord {
    name: GenerationName,
    path: PathBuf,
    sequence: u64,
    revision: u64,
    legacy: bool,
    previous: Option<GenerationName>,
}

/// The durable checkpoint decision made before Lumen starts accepting work.
///
/// This is intentionally about the checkpoint root only. The binary logs the
/// separate AOF replay decision after it applies the checkpoint baseline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentStartupDecision {
    InitializedEmptyRoot,
    RecoveredUncommittedEmpty,
    RestoredCurrentEmpty,
    RestoredCurrentGeneration,
    AdoptedLegacy0428,
}

impl SegmentStartupDecision {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InitializedEmptyRoot => "initialized_empty_root",
            Self::RecoveredUncommittedEmpty => "recovered_uncommitted_empty",
            Self::RestoredCurrentEmpty => "restored_current_empty",
            Self::RestoredCurrentGeneration => "restored_current_generation",
            Self::AdoptedLegacy0428 => "adopted_legacy_0428",
        }
    }
}

/// The exact successful checkpoint-root state selected during cold start.
#[derive(Clone, Debug)]
pub struct SegmentStartupOutcome {
    pub decision: SegmentStartupDecision,
    pub checkpoint_sequence: Option<u64>,
    pub generation: Option<GenerationName>,
    pub recovered_legacy_aside: bool,
    pub staging_cleaned: usize,
}

#[derive(Clone, Copy, Debug)]
enum StartupBootstrap {
    ExistingCurrent {
        staging_cleaned: usize,
    },
    InitializedEmpty {
        recovered_uncommitted: bool,
        staging_cleaned: usize,
    },
    Legacy {
        recovered_legacy_aside: bool,
        staging_cleaned: usize,
    },
}

#[derive(Debug, Default)]
struct RootInventory {
    non_seed_entries: usize,
    revision_generations: Vec<String>,
    has_aof_log: bool,
    has_aof_compact_temp: bool,
}

/// The exact generation selected by `CURRENT`, reopened into a fresh engine.
#[derive(Clone)]
pub struct LoadedSegmentGeneration {
    pub name: GenerationName,
    pub sequence: u64,
    pub engine: Arc<Engine>,
}

impl GenerationRecord {
    fn order_key(&self) -> (u8, u64) {
        if self.legacy {
            (0, self.sequence)
        } else {
            (1, self.revision)
        }
    }
}

/// Filesystem-backed segment checkpoints selected through one durable
/// `CURRENT` pointer.
///
/// Clones and independently opened handles for the same canonical root share
/// `save_lock` inside one process. The lock covers preparation, abandoned-stage
/// cleanup, activation, reopen, and prune. As required by `GenerationStore`, one
/// process must own all mutations for a root; cross-process writers are not
/// supported.
#[derive(Clone)]
pub struct SegmentRdbStore {
    root: PathBuf,
    save_lock: Arc<Mutex<()>>,
    generations: GenerationStore,
    bootstrap: StartupBootstrap,
}

impl fmt::Debug for SegmentRdbStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SegmentRdbStore")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl SegmentRdbStore {
    /// Open or create the checkpoint root.
    ///
    /// A genuinely empty root receives the explicit empty `CURRENT` sentinel.
    /// A root with exact 0.4.28 legacy generations remains uninitialized until
    /// [`Self::reopen_into_with_outcome`] validates and adopts the highest one.
    /// A non-empty root with an unknown layout fails before this method writes
    /// `CURRENT` or removes any entry.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        Self::open_with_injector(root, Arc::new(NoFailures))
    }

    fn open_with_injector(
        root: impl Into<PathBuf>,
        injector: Arc<dyn FailureInjector>,
    ) -> Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)
            .with_context(|| format!("create segment-checkpoint dir {}", root.display()))?;
        let generations = GenerationStore::open_with_injector(&root, injector)
            .with_context(|| format!("open generation store {}", root.display()))?;
        let root = std::fs::canonicalize(&root)
            .with_context(|| format!("canonicalize checkpoint root {}", root.display()))?;
        let store = Self {
            save_lock: shared_save_lock(&root)?,
            root,
            generations,
            bootstrap: StartupBootstrap::ExistingCurrent { staging_cleaned: 0 },
        };
        let _guard = store
            .save_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let bootstrap = store.prepare_startup_root()?;
        drop(_guard);
        Ok(Self { bootstrap, ..store })
    }

    /// Open a store with deterministic filesystem failures for restore tests.
    #[cfg(test)]
    pub(crate) fn new_with_failure_injector(
        root: impl Into<PathBuf>,
        injector: Arc<dyn FailureInjector>,
    ) -> Result<Self> {
        Self::open_with_injector(root, injector)
    }

    /// Checkpoint `engine` through a new immutable generation.
    ///
    /// Same-sequence saves remain meaningful because reshard operations can
    /// change state without advancing `applied_seq`. A lower sequence can only
    /// be a stale background caller, so it returns without moving `CURRENT`.
    pub fn save(&self, engine: &Arc<Engine>, up_to_seq: u64) -> Result<()> {
        self.save_inner(engine, up_to_seq, false).map(|_| ())
    }

    /// Save a generation for a restore operation.
    ///
    /// Unlike [`Self::save`], this never silently ignores a stale sequence and
    /// always creates a new revision, including when the sequence is unchanged.
    pub fn save_required(&self, engine: &Arc<Engine>, up_to_seq: u64) -> Result<GenerationName> {
        self.save_inner(engine, up_to_seq, true)
    }

    fn save_inner(
        &self,
        engine: &Arc<Engine>,
        up_to_seq: u64,
        required: bool,
    ) -> Result<GenerationName> {
        let _guard = self
            .save_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.inventory_root()?;
        self.sweep_abandoned_staging()?;

        let current = self.current_record()?;
        if let Some(current) = &current {
            if up_to_seq < current.sequence {
                if required {
                    bail!(
                        "required segment generation sequence {up_to_seq} is below CURRENT sequence {}",
                        current.sequence
                    );
                }
                return Ok(current.name.clone());
            }
        }

        let (revision, staged) = self.begin_next_generation(up_to_seq)?;
        let staging_path = staged.path().to_path_buf();
        if let Err(error) = engine.flush_to_segments(&staging_path, up_to_seq) {
            let _ = std::fs::remove_dir_all(&staging_path);
            return Err(error).context("flush collections to segment checkpoint");
        }

        let previous = current.as_ref().map(|record| record.name.clone());
        let manifest = SegmentGenerationManifest {
            schema_version: GENERATION_MANIFEST_SCHEMA_VERSION,
            sequence: up_to_seq,
            revision,
            previous: previous.as_ref().map(|name| name.as_str().to_owned()),
        };
        if let Err(error) = write_generation_manifest(&staging_path, &manifest) {
            let _ = std::fs::remove_dir_all(&staging_path);
            return Err(error);
        }

        let staged_record = GenerationRecord {
            name: staged.generation().clone(),
            path: staging_path.clone(),
            sequence: up_to_seq,
            revision,
            legacy: false,
            previous,
        };
        if let Err(error) = self.validate_record(&staged_record) {
            let _ = std::fs::remove_dir_all(&staging_path);
            return Err(error).context("validate staged segment generation");
        }

        self.generations
            .commit(staged)
            .map_err(anyhow::Error::new)
            .with_context(|| {
                format!("activate segment generation seq {up_to_seq} revision {revision}")
            })?;
        Ok(staged_record.name)
    }

    /// Reopen the exact active checkpoint into a fresh engine.
    pub fn load_latest(&self) -> Result<Option<(Arc<Engine>, u64)>> {
        let engine = Arc::new(Engine::new());
        match self.reopen_into(&engine)? {
            Some(seq) => Ok(Some((engine, seq))),
            None => Ok(None),
        }
    }

    /// Load exactly the generation named by `CURRENT`.
    ///
    /// This method never performs legacy adoption and never searches for a
    /// higher, unpointed generation.
    pub fn load_current_generation(&self) -> Result<Option<LoadedSegmentGeneration>> {
        let _guard = self
            .save_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let CurrentTarget::Generation(name) = self
            .generations
            .read_current()
            .map_err(|error| anyhow::Error::new(error).context("read CURRENT"))?
        else {
            return Ok(None);
        };
        let record = self.record_for_name(name.clone())?;
        let engine = Arc::new(Engine::new());
        self.reopen_record(&engine, &record)?;
        Ok(Some(LoadedSegmentGeneration {
            name,
            sequence: record.sequence,
            engine,
        }))
    }

    /// Reopen the cold-start checkpoint and return the decision that selected
    /// it. A missing `CURRENT` can adopt only an exact 0.4.28 generation that
    /// the open-time inventory already accepted. It never selects an unpointed
    /// revision or falls back from a corrupt highest legacy generation.
    pub fn reopen_into_with_outcome(&self, engine: &Arc<Engine>) -> Result<SegmentStartupOutcome> {
        let _guard = self
            .save_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match self.generations.read_current() {
            Ok(CurrentTarget::Empty) => match self.bootstrap {
                StartupBootstrap::InitializedEmpty {
                    recovered_uncommitted,
                    staging_cleaned,
                } => Ok(SegmentStartupOutcome {
                    decision: if recovered_uncommitted {
                        SegmentStartupDecision::RecoveredUncommittedEmpty
                    } else {
                        SegmentStartupDecision::InitializedEmptyRoot
                    },
                    checkpoint_sequence: None,
                    generation: None,
                    recovered_legacy_aside: false,
                    staging_cleaned,
                }),
                StartupBootstrap::ExistingCurrent { staging_cleaned } => {
                    Ok(SegmentStartupOutcome {
                        decision: SegmentStartupDecision::RestoredCurrentEmpty,
                        checkpoint_sequence: None,
                        generation: None,
                        recovered_legacy_aside: false,
                        staging_cleaned,
                    })
                }
                StartupBootstrap::Legacy { .. } => {
                    bail!("CURRENT became empty before legacy segment generation adoption")
                }
            },
            Ok(CurrentTarget::Generation(name)) => {
                let record = self.record_for_name(name)?;
                let seq = self.reopen_record(engine, &record)?;
                let staging_cleaned = match self.bootstrap {
                    StartupBootstrap::ExistingCurrent { staging_cleaned } => staging_cleaned,
                    StartupBootstrap::InitializedEmpty {
                        staging_cleaned, ..
                    }
                    | StartupBootstrap::Legacy {
                        staging_cleaned, ..
                    } => staging_cleaned,
                };
                Ok(SegmentStartupOutcome {
                    decision: SegmentStartupDecision::RestoredCurrentGeneration,
                    checkpoint_sequence: Some(seq),
                    generation: Some(record.name),
                    recovered_legacy_aside: false,
                    staging_cleaned,
                })
            }
            Err(error) if error.kind == CurrentReadErrorKind::Missing => {
                let StartupBootstrap::Legacy {
                    recovered_legacy_aside,
                    staging_cleaned,
                } = self.bootstrap
                else {
                    bail!("CURRENT disappeared after segment root initialization");
                };
                let inventory = self.inventory_root()?;
                if let Some(name) = inventory.revision_generations.first() {
                    bail!(
                        "CURRENT is missing but root contains unpointed revision generation `{name}`; refusing to select or initialize it"
                    );
                }
                let Some(record) = self.legacy_records()?.into_iter().next_back() else {
                    bail!("CURRENT is missing and no exact 0.4.28 generation can be adopted");
                };
                let seq = self.reopen_record(engine, &record)?;
                self.generations
                    .adopt_legacy(record.name.clone())
                    .map_err(anyhow::Error::new)
                    .with_context(|| format!("adopt legacy segment generation {}", record.name))?;
                Ok(SegmentStartupOutcome {
                    decision: SegmentStartupDecision::AdoptedLegacy0428,
                    checkpoint_sequence: Some(seq),
                    generation: Some(record.name),
                    recovered_legacy_aside,
                    staging_cleaned,
                })
            }
            Err(error) => Err(anyhow::Error::new(error).context("read CURRENT")),
        }
    }

    /// Reopen the cold-start checkpoint without exposing the startup decision.
    pub fn reopen_into(&self, engine: &Arc<Engine>) -> Result<Option<u64>> {
        Ok(self.reopen_into_with_outcome(engine)?.checkpoint_sequence)
    }

    /// Retain the active generation and up to `keep - 1` prior generations.
    ///
    /// `keep=0` still retains the active generation. Complete revisions newer
    /// than `CURRENT` are failed pre-commit attempts and are removed. The method
    /// never removes the directory named by `CURRENT`. If a prior prune already
    /// removed the predecessor named by an immutable manifest, this call keeps
    /// any unlinked older directories whose lineage it can no longer prove.
    pub fn prune(&self, keep: usize) -> Result<usize> {
        let _guard = self
            .save_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.inventory_root()?;
        self.sweep_abandoned_staging()?;
        let (history, truncated) = self.active_history()?;
        let all = self.generation_entries()?;

        let retain: BTreeSet<_> = history
            .iter()
            .take(keep.max(1))
            .map(|record| record.name.as_str().to_owned())
            .collect();
        let active_chain: BTreeSet<_> = history
            .iter()
            .map(|record| record.name.as_str().to_owned())
            .collect();
        let current = history.first();

        let mut removed = 0usize;
        for (name, path) in all {
            if retain.contains(&name) {
                continue;
            }
            if truncated
                && !active_chain.contains(&name)
                && !current.is_some_and(|current| definitely_unpointed_after(&name, current))
            {
                continue;
            }
            std::fs::remove_dir_all(&path)
                .with_context(|| format!("remove segment generation {}", path.display()))?;
            removed += 1;
        }
        if removed > 0 {
            sync_directory(&self.root).context("fsync checkpoint root after prune")?;
        }
        Ok(removed)
    }

    /// Activated predecessor-chain sequences, ascending and de-duplicated.
    pub fn generation_seqs(&self) -> Result<Vec<u64>> {
        Ok(self
            .active_history()?
            .0
            .into_iter()
            .map(|record| record.sequence)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect())
    }

    /// Inspect every direct child before any root mutation. The segment root is
    /// Lumen-owned, but a wrong mount or an older unsupported layout must still
    /// fail loudly instead of being converted into a fresh empty store.
    fn inventory_root(&self) -> Result<RootInventory> {
        let mut inventory = RootInventory::default();
        let mut children = Vec::new();
        let mut violations = Vec::new();
        let mut entries = std::fs::read_dir(&self.root)
            .with_context(|| format!("read checkpoint root {}", self.root.display()))?
            .collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .with_context(|| format!("inspect checkpoint root entry {}", path.display()))?;
            let kind = root_entry_kind(&metadata);
            let raw = match entry.file_name().into_string() {
                Ok(raw) => raw,
                Err(name) => {
                    let display = format!("{name:?}");
                    children.push(format!("{display} ({kind})"));
                    violations.push(format!("checkpoint root has non-UTF-8 entry {display}"));
                    continue;
                }
            };
            children.push(format!("{raw} ({kind})"));
            let regular_file = !metadata.file_type().is_symlink() && metadata.is_file();
            let real_directory = !metadata.file_type().is_symlink() && metadata.is_dir();

            if raw != CONTAINER_VOLUME_SEED_FILE {
                inventory.non_seed_entries += 1;
            }

            if matches!(
                raw.as_str(),
                CURRENT_FILE
                    | CURRENT_TEMP_FILE
                    | AOF_FILE
                    | AOF_COMPACT_TEMP_FILE
                    | CONTAINER_VOLUME_SEED_FILE
            ) {
                if !regular_file {
                    violations.push(format!(
                        "checkpoint root entry must be a regular file: {}",
                        path.display()
                    ));
                } else if raw == AOF_FILE {
                    inventory.has_aof_log = true;
                } else if raw == AOF_COMPACT_TEMP_FILE {
                    inventory.has_aof_compact_temp = true;
                }
                continue;
            }
            if parse_legacy_name(&raw).is_some() {
                if !real_directory {
                    violations.push(format!(
                        "legacy checkpoint must be a real directory: {}",
                        path.display()
                    ));
                }
                continue;
            }
            if parse_revision_name(&raw).is_some() {
                if !real_directory {
                    violations.push(format!(
                        "segment generation must be a real directory: {}",
                        path.display()
                    ));
                } else {
                    inventory.revision_generations.push(raw);
                }
                continue;
            }
            if is_legacy_aside_name(&raw) {
                if !real_directory {
                    violations.push(format!(
                        "legacy aside must be a real directory: {}",
                        path.display()
                    ));
                }
                continue;
            }
            if is_known_staging_name(&raw) {
                if !real_directory {
                    violations.push(format!(
                        "checkpoint staging must be a real directory: {}",
                        path.display()
                    ));
                }
                continue;
            }
            violations.push(format!(
                "unrecognized non-empty segment checkpoint root entry `{raw}` at {}",
                path.display()
            ));
        }

        if inventory.has_aof_compact_temp && !inventory.has_aof_log {
            violations.push(format!(
                "{AOF_COMPACT_TEMP_FILE} requires regular {AOF_FILE} beside it"
            ));
        }
        if !violations.is_empty() {
            bail!(
                "invalid segment checkpoint root inventory [{}]; refusing to initialize CURRENT: {}",
                children.join(", "),
                violations.join("; ")
            );
        }
        Ok(inventory)
    }

    /// Establish the one permitted missing-`CURRENT` state before a caller can
    /// save or reopen. This runs under `save_lock` and mutates only after the
    /// full direct-child inventory has accepted the root.
    fn prepare_startup_root(&self) -> Result<StartupBootstrap> {
        let inventory = self.inventory_root()?;
        match self.generations.read_current() {
            Ok(_) => Ok(StartupBootstrap::ExistingCurrent {
                staging_cleaned: self.sweep_abandoned_staging()?,
            }),
            Err(error) if error.kind == CurrentReadErrorKind::Missing => {
                if let Some(name) = inventory.revision_generations.first() {
                    bail!(
                        "CURRENT is missing but root contains unpointed revision generation `{name}`; refusing to select or initialize it"
                    );
                }
                let recovered_legacy_aside = self.reconcile_legacy_asides()?;
                let staging_cleaned = self.sweep_abandoned_staging()?;
                if !self.legacy_records()?.is_empty() {
                    return Ok(StartupBootstrap::Legacy {
                        recovered_legacy_aside,
                        staging_cleaned,
                    });
                }
                self.generations
                    .initialize_empty()
                    .map_err(anyhow::Error::new)
                    .context("initialize empty segment generation store")?;
                Ok(StartupBootstrap::InitializedEmpty {
                    recovered_uncommitted: inventory.non_seed_entries > 0,
                    staging_cleaned,
                })
            }
            Err(error) => Err(anyhow::Error::new(error).context("read CURRENT")),
        }
    }

    fn current_record(&self) -> Result<Option<GenerationRecord>> {
        match self.generations.read_current() {
            Ok(CurrentTarget::Empty) => Ok(None),
            Ok(CurrentTarget::Generation(name)) => self.record_for_name(name).map(Some),
            Err(error) => Err(anyhow::Error::new(error).context("read CURRENT")),
        }
    }

    /// Return the exact activated predecessor chain, newest first. A missing
    /// predecessor is an allowed retention boundary because prune never mutates
    /// an immutable manifest merely to truncate its link.
    fn active_history(&self) -> Result<(Vec<GenerationRecord>, bool)> {
        let Some(mut record) = self.current_record()? else {
            return Ok((Vec::new(), false));
        };
        let mut visited = BTreeSet::new();
        let mut history = Vec::new();
        let mut truncated = false;

        loop {
            let name = record.name.as_str().to_owned();
            if !visited.insert(name.clone()) {
                bail!("segment generation predecessor cycle at `{name}`");
            }
            let next = if let Some(previous) = &record.previous {
                if let Some(previous_record) = self.record_if_present(previous.clone())? {
                    if previous_record.sequence > record.sequence
                        || previous_record.order_key() >= record.order_key()
                    {
                        bail!(
                            "generation {} has non-predecessor link {}",
                            record.name,
                            previous
                        );
                    }
                    Some(previous_record)
                } else {
                    truncated = true;
                    None
                }
            } else if record.legacy {
                self.legacy_records()?
                    .into_iter()
                    .filter(|candidate| candidate.legacy && candidate.sequence < record.sequence)
                    .max_by_key(|candidate| candidate.sequence)
            } else {
                None
            };
            history.push(record);
            let Some(next) = next else {
                break;
            };
            record = next;
        }
        Ok((history, truncated))
    }

    fn begin_next_generation(&self, sequence: u64) -> Result<(u64, StagedGeneration)> {
        let mut revision = self
            .generation_entries()?
            .into_iter()
            .filter_map(|(name, _)| parse_revision_name(&name).map(|(_, revision)| revision))
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| anyhow!("segment generation revision exhausted"))?;

        loop {
            let name = GenerationName::parse(format!("gen-{sequence}-rev-{revision}"))
                .map_err(anyhow::Error::new)
                .context("build segment generation name")?;
            match self.generations.begin(name) {
                Ok(staged) => return Ok((revision, staged)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    revision = revision
                        .checked_add(1)
                        .ok_or_else(|| anyhow!("segment generation revision exhausted"))?;
                }
                Err(error) => return Err(error).context("create segment generation staging"),
            }
        }
    }

    fn record_for_name(&self, name: GenerationName) -> Result<GenerationRecord> {
        let path = self.generations.generation_path(&name);
        let metadata = std::fs::symlink_metadata(&path)
            .with_context(|| format!("inspect segment generation {}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            bail!(
                "segment generation must be a real directory: {}",
                path.display()
            );
        }
        if let Some(sequence) = parse_legacy_name(name.as_str()) {
            if path.join(GENERATION_MANIFEST_FILE).exists() {
                bail!(
                    "legacy generation {} unexpectedly contains {}",
                    name,
                    GENERATION_MANIFEST_FILE
                );
            }
            return Ok(GenerationRecord {
                name,
                path,
                sequence,
                revision: 0,
                legacy: true,
                previous: None,
            });
        }

        let (sequence, revision) = parse_revision_name(name.as_str())
            .ok_or_else(|| anyhow!("CURRENT names an unsupported generation `{name}`"))?;
        let manifest = read_generation_manifest(&path)?;
        if manifest.schema_version != GENERATION_MANIFEST_SCHEMA_VERSION {
            bail!(
                "generation {} has unsupported manifest schema {}",
                name,
                manifest.schema_version
            );
        }
        if manifest.sequence != sequence || manifest.revision != revision {
            bail!(
                "generation {} manifest does not match its directory name",
                name
            );
        }
        let previous = match manifest.previous {
            Some(raw) => {
                if !is_supported_generation_name(&raw) {
                    bail!("generation {name} has unsupported predecessor `{raw}`");
                }
                if !is_older_predecessor(&raw, sequence, revision) {
                    bail!("generation {name} has non-predecessor link `{raw}`");
                }
                Some(
                    GenerationName::parse(raw)
                        .map_err(anyhow::Error::new)
                        .context("parse previous segment generation")?,
                )
            }
            None => None,
        };
        if previous.as_ref() == Some(&name) {
            bail!("generation {name} points to itself as predecessor");
        }
        Ok(GenerationRecord {
            name,
            path,
            sequence,
            revision,
            legacy: false,
            previous,
        })
    }

    fn record_if_present(&self, name: GenerationName) -> Result<Option<GenerationRecord>> {
        let path = self.generations.generation_path(&name);
        match std::fs::symlink_metadata(&path) {
            Ok(_) => self.record_for_name(name).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => {
                Err(error).with_context(|| format!("inspect segment generation {}", path.display()))
            }
        }
    }

    fn generation_entries(&self) -> Result<Vec<(String, PathBuf)>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&self.root)
            .with_context(|| format!("read checkpoint root {}", self.root.display()))?
        {
            let entry = entry?;
            let Some(raw) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if parse_legacy_name(&raw).is_none() && parse_revision_name(&raw).is_none() {
                continue;
            }
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .with_context(|| format!("inspect segment generation {}", path.display()))?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!(
                    "segment generation must be a real directory: {}",
                    path.display()
                );
            }
            entries.push((raw, path));
        }
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(entries)
    }

    fn legacy_records(&self) -> Result<Vec<GenerationRecord>> {
        let mut records = Vec::new();
        for (raw, _) in self.generation_entries()? {
            if parse_legacy_name(&raw).is_none() {
                continue;
            }
            let name = GenerationName::parse(raw)
                .map_err(anyhow::Error::new)
                .context("parse legacy segment generation")?;
            records.push(self.record_for_name(name)?);
        }
        records.sort_by_key(|record| record.sequence);
        Ok(records)
    }

    fn reopen_record(&self, engine: &Arc<Engine>, record: &GenerationRecord) -> Result<u64> {
        let collections = self.validate_record(record)?;
        self.reopen_once(engine.as_ref(), record, collections)?;
        Ok(record.sequence)
    }

    /// Fully validate and reopen into a disposable engine before any caller
    /// engine can change. Generations are immutable after activation, and the
    /// serving process is their sole writer, so the second reopen sees the same
    /// bytes without exposing a partly installed collection set on corruption.
    fn validate_record(&self, record: &GenerationRecord) -> Result<usize> {
        let collections = validate_generation_layout(record)?;
        let verifier = Engine::new();
        self.reopen_once(&verifier, record, collections)?;
        Ok(collections)
    }

    fn reopen_once(
        &self,
        engine: &Engine,
        record: &GenerationRecord,
        collections: usize,
    ) -> Result<()> {
        let reopened = engine
            .reopen_from_segment_dir(&record.path)
            .with_context(|| format!("reopen checkpoint {}", record.path.display()))?;
        if collections == 0 {
            if reopened != 0 {
                bail!(
                    "empty generation {} reopened with unexpected sequence {reopened}",
                    record.name
                );
            }
        } else if reopened != record.sequence {
            bail!(
                "generation {} expected sequence {} but reopened {reopened}",
                record.name,
                record.sequence
            );
        }
        Ok(())
    }

    fn sweep_abandoned_staging(&self) -> Result<usize> {
        let mut removed = 0usize;
        for entry in std::fs::read_dir(&self.root)
            .with_context(|| format!("read checkpoint root {}", self.root.display()))?
        {
            let entry = entry?;
            let Some(raw) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if !is_known_staging_name(&raw) {
                continue;
            }
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .with_context(|| format!("inspect checkpoint staging {}", path.display()))?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!(
                    "checkpoint staging must be a real directory: {}",
                    path.display()
                );
            }
            std::fs::remove_dir_all(&path)
                .with_context(|| format!("remove abandoned staging {}", path.display()))?;
            removed += 1;
        }
        if removed > 0 {
            sync_directory(&self.root).context("fsync root after staging cleanup")?;
        }
        Ok(removed)
    }

    fn reconcile_legacy_asides(&self) -> Result<bool> {
        let mut asides = Vec::new();
        for entry in std::fs::read_dir(&self.root)
            .with_context(|| format!("read checkpoint root {}", self.root.display()))?
        {
            let entry = entry?;
            let Some(raw) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Some(sequence) = parse_legacy_aside_name(&raw) else {
                continue;
            };
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .with_context(|| format!("inspect legacy aside {}", path.display()))?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!("legacy aside must be a real directory: {}", path.display());
            }
            asides.push((sequence, path));
        }
        asides.sort_by_key(|(sequence, _)| *sequence);
        let mut changed = false;
        for (sequence, aside) in asides {
            let committed = self.root.join(format!("gen-{sequence}"));
            match std::fs::symlink_metadata(&committed) {
                Ok(metadata) if !metadata.file_type().is_symlink() && metadata.is_dir() => {
                    std::fs::remove_dir_all(&aside).with_context(|| {
                        format!("remove stale legacy aside {}", aside.display())
                    })?;
                }
                Ok(_) => {
                    bail!(
                        "legacy checkpoint target must be a real directory: {}",
                        committed.display()
                    );
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    std::fs::rename(&aside, &committed).with_context(|| {
                        format!(
                            "restore legacy checkpoint {} -> {}",
                            aside.display(),
                            committed.display()
                        )
                    })?;
                }
                Err(error) => {
                    return Err(error).with_context(|| {
                        format!("inspect legacy checkpoint {}", committed.display())
                    });
                }
            }
            changed = true;
        }
        if changed {
            sync_directory(&self.root).context("fsync root after legacy aside recovery")?;
        }
        Ok(changed)
    }
}

fn parse_canonical_u64(raw: &str) -> Option<u64> {
    let value = raw.parse::<u64>().ok()?;
    (value.to_string() == raw).then_some(value)
}

fn parse_legacy_name(raw: &str) -> Option<u64> {
    parse_canonical_u64(raw.strip_prefix("gen-")?)
}

fn parse_legacy_aside_name(raw: &str) -> Option<u64> {
    raw.strip_prefix("gen-")?
        .strip_suffix(".old")
        .and_then(parse_canonical_u64)
}

fn root_entry_kind(metadata: &std::fs::Metadata) -> &'static str {
    if metadata.file_type().is_symlink() {
        "symlink"
    } else if metadata.is_file() {
        "regular file"
    } else if metadata.is_dir() {
        "directory"
    } else {
        "special file"
    }
}

fn is_legacy_aside_name(raw: &str) -> bool {
    parse_legacy_aside_name(raw).is_some()
}

fn is_known_staging_name(raw: &str) -> bool {
    raw.strip_prefix(".gen-")
        .and_then(|name| name.strip_suffix(".tmp"))
        .and_then(parse_canonical_u64)
        .is_some()
        || raw
            .strip_prefix(".stage-")
            .and_then(parse_revision_name)
            .is_some()
}

fn parse_revision_name(raw: &str) -> Option<(u64, u64)> {
    let rest = raw.strip_prefix("gen-")?;
    let (sequence, revision) = rest.rsplit_once("-rev-")?;
    Some((
        parse_canonical_u64(sequence)?,
        parse_canonical_u64(revision)?,
    ))
}

fn is_supported_generation_name(raw: &str) -> bool {
    parse_legacy_name(raw).is_some() || parse_revision_name(raw).is_some()
}

fn is_older_predecessor(raw: &str, current_sequence: u64, current_revision: u64) -> bool {
    if let Some(sequence) = parse_legacy_name(raw) {
        sequence <= current_sequence
    } else if let Some((sequence, revision)) = parse_revision_name(raw) {
        sequence <= current_sequence && revision < current_revision
    } else {
        false
    }
}

fn definitely_unpointed_after(raw: &str, current: &GenerationRecord) -> bool {
    if let Some(sequence) = parse_legacy_name(raw) {
        sequence > current.sequence
    } else if let Some((sequence, revision)) = parse_revision_name(raw) {
        current.legacy || sequence > current.sequence || revision >= current.revision
    } else {
        false
    }
}

fn write_generation_manifest(path: &Path, manifest: &SegmentGenerationManifest) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(manifest).context("encode generation manifest")?;
    bytes.push(b'\n');
    std::fs::write(path.join(GENERATION_MANIFEST_FILE), bytes)
        .with_context(|| format!("write generation manifest under {}", path.display()))
}

fn read_generation_manifest(path: &Path) -> Result<SegmentGenerationManifest> {
    let manifest_path = path.join(GENERATION_MANIFEST_FILE);
    let metadata = std::fs::symlink_metadata(&manifest_path)
        .with_context(|| format!("inspect generation manifest {}", manifest_path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!(
            "generation manifest must be a regular file: {}",
            manifest_path.display()
        );
    }
    if metadata.len() > 4096 {
        bail!(
            "generation manifest is too large: {}",
            manifest_path.display()
        );
    }
    let bytes = std::fs::read(&manifest_path)
        .with_context(|| format!("read generation manifest {}", manifest_path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("decode generation manifest {}", manifest_path.display()))
}

fn validate_generation_layout(record: &GenerationRecord) -> Result<usize> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&record.path)
        .with_context(|| format!("read generation {}", record.path.display()))?
    {
        entries.push(entry?.path());
    }
    entries.sort();

    let mut collections = 0usize;
    for path in entries {
        let metadata = std::fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            bail!("generation contains a symlink: {}", path.display());
        }
        if path.file_name().and_then(|name| name.to_str()) == Some(GENERATION_MANIFEST_FILE) {
            if record.legacy || !metadata.is_file() {
                bail!("invalid generation manifest entry: {}", path.display());
            }
            continue;
        }
        if !metadata.is_dir() {
            bail!("unexpected generation entry: {}", path.display());
        }
        validate_real_tree(&path)?;

        let schema_path = path.join(CHECKPOINT_SCHEMA_FILE);
        let schema_metadata = std::fs::symlink_metadata(&schema_path)
            .with_context(|| format!("inspect checkpoint schema {}", schema_path.display()))?;
        if schema_metadata.file_type().is_symlink() || !schema_metadata.is_file() {
            bail!(
                "checkpoint schema must be a regular file: {}",
                schema_path.display()
            );
        }
        let schema: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&schema_path)
                .with_context(|| format!("read checkpoint schema {}", schema_path.display()))?,
        )
        .with_context(|| format!("decode checkpoint schema {}", schema_path.display()))?;
        let applied_seq = schema
            .get("applied_seq")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                anyhow!(
                    "checkpoint schema has no applied_seq: {}",
                    schema_path.display()
                )
            })?;
        if applied_seq != record.sequence {
            bail!(
                "checkpoint schema {} has sequence {applied_seq}, expected {}",
                schema_path.display(),
                record.sequence
            );
        }
        collections += 1;
    }

    if !record.legacy {
        if let Some(previous) = &record.previous {
            if !is_older_predecessor(previous.as_str(), record.sequence, record.revision) {
                bail!(
                    "generation {} has non-predecessor link {}",
                    record.name,
                    previous
                );
            }
        }
        let manifest = read_generation_manifest(&record.path)?;
        let expected_previous = record.previous.as_ref().map(GenerationName::as_str);
        if manifest.schema_version != GENERATION_MANIFEST_SCHEMA_VERSION
            || manifest.sequence != record.sequence
            || manifest.revision != record.revision
            || manifest.previous.as_deref() != expected_previous
        {
            bail!(
                "generation {} manifest does not match its validated record",
                record.name
            );
        }
    }
    Ok(collections)
}

fn validate_real_tree(root: &Path) -> Result<()> {
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let metadata = std::fs::symlink_metadata(&directory)
            .with_context(|| format!("inspect checkpoint path {}", directory.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            bail!(
                "checkpoint directory must be a real directory: {}",
                directory.display()
            );
        }

        let mut entries = std::fs::read_dir(&directory)
            .with_context(|| format!("read checkpoint directory {}", directory.display()))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        entries.sort();
        for path in entries {
            let metadata = std::fs::symlink_metadata(&path)
                .with_context(|| format!("inspect checkpoint path {}", path.display()))?;
            if metadata.file_type().is_symlink() {
                bail!("checkpoint contains a symlink: {}", path.display());
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if !metadata.is_file() {
                bail!(
                    "checkpoint contains a special filesystem entry: {}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

fn shared_save_lock(root: &Path) -> Result<Arc<Mutex<()>>> {
    static ROOT_LOCKS: OnceLock<Mutex<HashMap<PathBuf, Weak<Mutex<()>>>>> = OnceLock::new();

    let registry = ROOT_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut registry = registry
        .lock()
        .map_err(|_| anyhow!("segment root-lock registry poisoned"))?;
    registry.retain(|_, lock| lock.strong_count() > 0);
    if let Some(lock) = registry.get(root).and_then(Weak::upgrade) {
        return Ok(lock);
    }

    let lock = Arc::new(Mutex::new(()));
    registry.insert(root.to_path_buf(), Arc::downgrade(&lock));
    Ok(lock)
}

fn sync_directory(path: &Path) -> std::io::Result<()> {
    OpenOptions::new().read(true).open(path)?.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    };
    use std::collections::BTreeMap;

    fn kw_schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert(
            "email".to_string(),
            FieldSpec {
                field_type: FieldType::Keyword,
                analyzer: None,
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        );
        CreateCollectionRequest { fields }
    }

    fn index_kw(e: &Engine, eid: &str, v: &str) {
        index_kw_in(e, "u", eid, v);
    }

    fn index_kw_in(e: &Engine, collection: &str, eid: &str, v: &str) {
        e.index(
            collection,
            IndexRequest {
                items: vec![IndexItem {
                    external_id: eid.into(),
                    field: "email".into(),
                    value: FieldValue::String(v.into()),
                    version: None,
                }],
                request_id: None,
            },
        )
        .unwrap();
    }

    fn current_generation(store: &SegmentRdbStore) -> GenerationName {
        match store.generations.read_current().unwrap() {
            CurrentTarget::Generation(name) => name,
            CurrentTarget::Empty => panic!("expected an active generation"),
        }
    }

    fn install_unpointed_generation(
        store: &SegmentRdbStore,
        engine: &Arc<Engine>,
        sequence: u64,
        previous: Option<&GenerationName>,
    ) -> GenerationName {
        let (revision, staged) = store.begin_next_generation(sequence).unwrap();
        let staging_path = staged.path().to_path_buf();
        engine.flush_to_segments(&staging_path, sequence).unwrap();
        write_generation_manifest(
            &staging_path,
            &SegmentGenerationManifest {
                schema_version: GENERATION_MANIFEST_SCHEMA_VERSION,
                sequence,
                revision,
                previous: previous.map(|name| name.as_str().to_owned()),
            },
        )
        .unwrap();
        let name = staged.generation().clone();
        let target = store.generations.generation_path(&name);
        drop(staged);
        std::fs::rename(staging_path, &target).unwrap();
        sync_directory(&store.root).unwrap();
        name
    }

    fn first_segment_file(root: &Path) -> PathBuf {
        let mut pending = vec![root.to_path_buf()];
        while let Some(directory) = pending.pop() {
            let mut entries = std::fs::read_dir(directory)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            entries.sort();
            for path in entries {
                if path.is_dir() {
                    pending.push(path);
                } else if path.extension().and_then(|extension| extension.to_str()) == Some("lseg")
                {
                    return path;
                }
            }
        }
        panic!("expected a segment file under {}", root.display());
    }

    #[test]
    fn save_then_load_round_trips_at_seq() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();

        let src = Arc::new(Engine::new());
        src.create_collection("u", kw_schema()).unwrap();
        index_kw(&src, "u1", "a@x.com");
        store.save(&src, 42).unwrap();

        let (eng, seq) = store.load_latest().unwrap().expect("a checkpoint");
        assert_eq!(seq, 42);
        assert_eq!(eng.stats("u").unwrap().documents_indexed, 1);
    }

    #[test]
    fn adopts_exact_0428_generation_once_and_writes_exact_current() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(!dir.path().join("CURRENT").exists());
        let loaded = Arc::new(Engine::new());
        let outcome = store.reopen_into_with_outcome(&loaded).unwrap();
        assert_eq!(outcome.decision, SegmentStartupDecision::AdoptedLegacy0428);
        assert_eq!(outcome.checkpoint_sequence, Some(42));
        assert_eq!(
            outcome.generation.as_ref().map(GenerationName::as_str),
            Some("gen-42")
        );
        assert_eq!(loaded.stats("u").unwrap().documents_indexed, 1);
        assert_eq!(
            std::fs::read(dir.path().join("CURRENT")).unwrap(),
            b"generation:gen-42\n"
        );

        let restarted = SegmentRdbStore::new(dir.path()).unwrap();
        assert_eq!(restarted.load_latest().unwrap().unwrap().1, 42);
    }

    #[test]
    fn adopts_empty_0428_generation_without_losing_sequence() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("gen-42")).unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let (_, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 42);
        assert_eq!(
            std::fs::read(dir.path().join("CURRENT")).unwrap(),
            b"generation:gen-42\n"
        );
    }

    #[test]
    fn corrupt_highest_legacy_generation_blocks_fallback_and_adoption() {
        let dir = tempfile::tempdir().unwrap();
        let valid = dir.path().join("gen-42");
        std::fs::create_dir(&valid).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&valid, 42).unwrap();
        let corrupt = dir.path().join("gen-99");
        std::fs::create_dir(&corrupt).unwrap();
        std::fs::write(corrupt.join("not-a-collection"), b"corrupt").unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(store.load_latest().is_err());
        assert!(!dir.path().join("CURRENT").exists());
    }

    #[cfg(unix)]
    #[test]
    fn legacy_nested_symlink_blocks_adoption() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(outside.path(), b"outside").unwrap();
        let collection = std::fs::read_dir(&legacy)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.is_dir())
            .unwrap();
        symlink(outside.path(), collection.join("nested-link")).unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(store.load_latest().is_err());
        assert!(!dir.path().join("CURRENT").exists());
    }

    #[cfg(unix)]
    #[test]
    fn parseable_legacy_aside_symlink_blocks_empty_initialization() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        symlink(target.path(), dir.path().join("gen-7.old")).unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
        assert!(!dir.path().join("CURRENT").exists());
    }

    #[test]
    fn empty_new_generation_preserves_sequence_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        store.save(&Arc::new(Engine::new()), 42).unwrap();

        let restarted = SegmentRdbStore::new(dir.path()).unwrap();
        let (_, sequence) = restarted.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 42);
    }

    #[test]
    fn lower_sequence_save_is_a_noop() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 9).unwrap();
        let current_before = std::fs::read(dir.path().join("CURRENT")).unwrap();

        index_kw(&engine, "u2", "b@x.com");
        store.save(&engine, 8).unwrap();

        assert_eq!(
            std::fs::read(dir.path().join("CURRENT")).unwrap(),
            current_before
        );
        let (loaded, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 9);
        assert_eq!(loaded.stats("u").unwrap().documents_indexed, 1);
    }

    #[test]
    fn required_lower_sequence_rejects_without_changing_current() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        store.save(&engine, 9).unwrap();
        let before = std::fs::read(dir.path().join("CURRENT")).unwrap();

        let error = store.save_required(&engine, 8).unwrap_err();
        assert!(error.to_string().contains("below CURRENT sequence 9"));
        assert_eq!(std::fs::read(dir.path().join("CURRENT")).unwrap(), before);
    }

    #[test]
    fn required_same_sequence_creates_distinct_revision_and_exact_loader_matches() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        let first = store.save_required(&engine, 9).unwrap();
        index_kw(&engine, "u2", "b@x.com");
        let second = store.save_required(&engine, 9).unwrap();
        assert_ne!(first, second);

        let loaded = store.load_current_generation().unwrap().unwrap();
        assert_eq!(loaded.name, second);
        assert_eq!(loaded.sequence, 9);
        assert_eq!(loaded.engine.stats("u").unwrap().documents_indexed, 2);
    }

    #[test]
    fn exact_loader_never_selects_unpointed_higher_generation() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        let current = store.save_required(&engine, 9).unwrap();
        let unpointed = install_unpointed_generation(&store, &engine, 99, Some(&current));

        let loaded = store.load_current_generation().unwrap().unwrap();
        assert_eq!(loaded.name, current);
        assert_eq!(loaded.sequence, 9);
        assert_ne!(loaded.name, unpointed);
    }

    #[test]
    fn exact_loader_never_adopts_a_legacy_generation_when_current_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(!dir.path().join("CURRENT").exists());
        assert!(store.load_current_generation().is_err());
        assert!(
            !dir.path().join("CURRENT").exists(),
            "exact restore reload must not perform the 0.4.28 startup adoption"
        );
    }

    #[test]
    fn injected_store_commit_is_deterministic() {
        #[derive(Default)]
        struct FailRenameCurrent(Mutex<Vec<storage_durable::FailurePoint>>);

        impl FailureInjector for FailRenameCurrent {
            fn check(&self, point: &storage_durable::FailurePoint) -> std::io::Result<()> {
                self.0.lock().unwrap().push(point.clone());
                if point.step == storage_durable::CommitStep::RenameCurrent {
                    return Err(std::io::Error::other("injected rename failure"));
                }
                Ok(())
            }
        }

        let dir = tempfile::tempdir().unwrap();
        SegmentRdbStore::new(dir.path()).unwrap();
        let injector = Arc::new(FailRenameCurrent::default());
        let store =
            SegmentRdbStore::new_with_failure_injector(dir.path(), injector.clone()).unwrap();
        let error = store
            .save_required(&Arc::new(Engine::new()), 1)
            .unwrap_err();
        assert!(error.to_string().contains("activate segment generation"));
        assert!(matches!(store.load_current_generation().unwrap(), None));
        assert!(injector
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|point| point.step == storage_durable::CommitStep::RenameCurrent));
    }

    #[test]
    fn load_latest_picks_highest_seq() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let e = Arc::new(Engine::new());
        e.create_collection("u", kw_schema()).unwrap();
        index_kw(&e, "u1", "a@x.com");
        for seq in [10u64, 5, 99, 50] {
            store.save(&e, seq).unwrap();
        }
        assert_eq!(store.load_latest().unwrap().unwrap().1, 99);
    }

    #[test]
    fn prune_keeps_newest() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let e = Arc::new(Engine::new());
        e.create_collection("u", kw_schema()).unwrap();
        index_kw(&e, "u1", "a@x.com");
        for seq in 1..=5u64 {
            store.save(&e, seq).unwrap();
        }
        let removed = store.prune(2).unwrap();
        assert_eq!(removed, 3);
        assert_eq!(store.generation_seqs().unwrap(), vec![4, 5]);
        assert_eq!(store.load_latest().unwrap().unwrap().1, 5);
    }

    #[test]
    fn torn_staging_dir_is_ignored_and_swept() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let e = Arc::new(Engine::new());
        e.create_collection("u", kw_schema()).unwrap();
        index_kw(&e, "u1", "a@x.com");
        store.save(&e, 7).unwrap();

        // Simulate a crash mid-stage: a leftover `.gen-<seq>.tmp` dir.
        std::fs::create_dir_all(dir.path().join(".gen-9.tmp")).unwrap();
        // load_latest still returns the good committed generation, not the torn one.
        assert_eq!(store.load_latest().unwrap().unwrap().1, 7);
        // A subsequent save sweeps the torn staging dir.
        store.save(&e, 8).unwrap();
        assert!(!dir.path().join(".gen-9.tmp").exists());
        assert_eq!(store.load_latest().unwrap().unwrap().1, 8);
    }

    #[test]
    fn abandoned_durable_staging_is_swept_on_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let _first = SegmentRdbStore::new(dir.path()).unwrap();
        let staging = dir.path().join(".stage-gen-9-rev-99");
        std::fs::create_dir(&staging).unwrap();
        std::fs::write(staging.join("partial"), b"partial").unwrap();

        let _reopened = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(!staging.exists());
    }

    #[test]
    fn unrelated_legacy_like_directory_is_not_swept() {
        let dir = tempfile::tempdir().unwrap();
        let unrelated = dir.path().join(".gen-user.tmp");
        std::fs::create_dir(&unrelated).unwrap();
        std::fs::write(unrelated.join("owned-by-user"), b"keep").unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
        assert!(
            !dir.path().join("CURRENT").exists(),
            "an unrecognized non-empty root must not become an empty store"
        );
        assert!(unrelated.is_dir());
        assert_eq!(
            std::fs::read(unrelated.join("owned-by-user")).unwrap(),
            b"keep"
        );
    }

    #[test]
    fn mixed_legacy_and_unknown_root_fails_before_legacy_adoption() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();
        std::fs::create_dir(dir.path().join("foreign-layout")).unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
        assert!(
            !dir.path().join("CURRENT").exists(),
            "unknown content must block legacy adoption before it writes CURRENT"
        );
        assert!(legacy.is_dir());
        assert!(dir.path().join("foreign-layout").is_dir());
    }

    #[test]
    fn unknown_entry_beside_valid_current_fails_before_any_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let first = SegmentRdbStore::new(dir.path()).unwrap();
        let current = std::fs::read(dir.path().join("CURRENT")).unwrap();
        drop(first);
        let foreign = dir.path().join("foreign-layout");
        std::fs::create_dir(&foreign).unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
        assert_eq!(std::fs::read(dir.path().join("CURRENT")).unwrap(), current);
        assert!(foreign.is_dir());
    }

    #[test]
    fn unpointed_revision_without_current_has_a_specific_fail_closed_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("gen-7-rev-1")).unwrap();

        let error = SegmentRdbStore::new(dir.path()).unwrap_err();
        assert!(error.to_string().contains("unpointed revision generation"));
        assert!(!dir.path().join("CURRENT").exists());
    }

    #[test]
    fn aof_only_root_remains_a_supported_empty_checkpoint_baseline() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("aof.log"), b"").unwrap();
        std::fs::write(dir.path().join("aof.log.compact.tmp"), b"").unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let outcome = store
            .reopen_into_with_outcome(&Arc::new(Engine::new()))
            .unwrap();
        assert_eq!(
            outcome.decision,
            SegmentStartupDecision::RecoveredUncommittedEmpty
        );
        assert_eq!(outcome.checkpoint_sequence, None);
        assert_eq!(
            std::fs::read(dir.path().join("CURRENT")).unwrap(),
            b"empty\n"
        );
    }

    #[test]
    fn compact_aof_temp_without_aof_is_rejected_before_current_is_written() {
        let dir = tempfile::tempdir().unwrap();
        let compact = dir.path().join(AOF_COMPACT_TEMP_FILE);
        let bytes = b"uncommitted compact output";
        std::fs::write(&compact, bytes).unwrap();

        let error = SegmentRdbStore::new(dir.path()).unwrap_err();
        assert!(error
            .to_string()
            .contains("aof.log.compact.tmp requires regular aof.log beside it"));
        assert!(!dir.path().join(CURRENT_FILE).exists());
        assert_eq!(std::fs::read(compact).unwrap(), bytes);
    }

    #[test]
    fn invalid_root_inventory_lists_every_child_name_and_kind_in_sorted_order() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(AOF_FILE)).unwrap();
        std::fs::write(dir.path().join("alpha-foreign"), b"foreign").unwrap();
        std::fs::create_dir(dir.path().join("zeta-foreign")).unwrap();

        let error = SegmentRdbStore::new(dir.path()).unwrap_err();
        let rendered = error.to_string();
        assert!(rendered.contains(
            "invalid segment checkpoint root inventory [alpha-foreign (regular file), aof.log (directory), zeta-foreign (directory)]"
        ));
        assert!(rendered.contains("checkpoint root entry must be a regular file"));
        assert!(rendered
            .contains("unrecognized non-empty segment checkpoint root entry `alpha-foreign`"));
        assert!(rendered
            .contains("unrecognized non-empty segment checkpoint root entry `zeta-foreign`"));
        assert!(!dir.path().join(CURRENT_FILE).exists());
    }

    #[test]
    fn current_empty_remains_authoritative_over_an_unpointed_revision() {
        let dir = tempfile::tempdir().unwrap();
        let _first = SegmentRdbStore::new(dir.path()).unwrap();
        let current = std::fs::read(dir.path().join(CURRENT_FILE)).unwrap();
        std::fs::create_dir(dir.path().join("gen-7-rev-1")).unwrap();

        let reopened = SegmentRdbStore::new(dir.path()).unwrap();
        let outcome = reopened
            .reopen_into_with_outcome(&Arc::new(Engine::new()))
            .unwrap();
        assert_eq!(
            outcome.decision,
            SegmentStartupDecision::RestoredCurrentEmpty
        );
        assert_eq!(
            std::fs::read(dir.path().join(CURRENT_FILE)).unwrap(),
            current
        );
    }

    #[test]
    fn genuinely_empty_root_reports_initialization_once_then_current_empty() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CONTAINER_VOLUME_SEED_FILE), b"").unwrap();
        let first = SegmentRdbStore::new(dir.path()).unwrap();
        assert_eq!(
            first
                .reopen_into_with_outcome(&Arc::new(Engine::new()))
                .unwrap()
                .decision,
            SegmentStartupDecision::InitializedEmptyRoot
        );

        let reopened = SegmentRdbStore::new(dir.path()).unwrap();
        assert_eq!(
            reopened
                .reopen_into_with_outcome(&Arc::new(Engine::new()))
                .unwrap()
                .decision,
            SegmentStartupDecision::RestoredCurrentEmpty
        );
    }

    #[test]
    fn legacy_aside_is_recovered_and_reported_before_adoption() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42.old");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let outcome = store
            .reopen_into_with_outcome(&Arc::new(Engine::new()))
            .unwrap();
        assert_eq!(outcome.decision, SegmentStartupDecision::AdoptedLegacy0428);
        assert!(outcome.recovered_legacy_aside);
        assert!(dir.path().join("gen-42").is_dir());
        assert!(!dir.path().join("gen-42.old").exists());
    }

    #[cfg(unix)]
    #[test]
    fn unknown_root_symlink_fails_before_current_is_written() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), dir.path().join("foreign-layout")).unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
        assert!(!dir.path().join("CURRENT").exists());
        assert!(dir.path().join("foreign-layout").is_symlink());
    }

    #[test]
    fn staged_corruption_is_rejected_before_current_moves() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        let (revision, staged) = store.begin_next_generation(7).unwrap();
        let staging_path = staged.path().to_path_buf();
        engine.flush_to_segments(&staging_path, 7).unwrap();
        write_generation_manifest(
            &staging_path,
            &SegmentGenerationManifest {
                schema_version: GENERATION_MANIFEST_SCHEMA_VERSION,
                sequence: 7,
                revision,
                previous: None,
            },
        )
        .unwrap();
        std::fs::write(first_segment_file(&staging_path), b"corrupt").unwrap();
        let record = GenerationRecord {
            name: staged.generation().clone(),
            path: staging_path,
            sequence: 7,
            revision,
            legacy: false,
            previous: None,
        };

        assert!(store.validate_record(&record).is_err());
        assert_eq!(
            store.generations.read_current().unwrap(),
            CurrentTarget::Empty
        );
    }

    #[test]
    fn staged_manifest_must_parse_and_match_the_validated_record() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        let (revision, staged) = store.begin_next_generation(7).unwrap();
        let staging_path = staged.path().to_path_buf();
        engine.flush_to_segments(&staging_path, 7).unwrap();
        let record = GenerationRecord {
            name: staged.generation().clone(),
            path: staging_path.clone(),
            sequence: 7,
            revision,
            legacy: false,
            previous: None,
        };

        std::fs::write(staging_path.join(GENERATION_MANIFEST_FILE), b"{\"").unwrap();
        assert!(store.validate_record(&record).is_err());

        write_generation_manifest(
            &staging_path,
            &SegmentGenerationManifest {
                schema_version: GENERATION_MANIFEST_SCHEMA_VERSION,
                sequence: 8,
                revision,
                previous: None,
            },
        )
        .unwrap();
        assert!(store.validate_record(&record).is_err());
        assert_eq!(
            store.generations.read_current().unwrap(),
            CurrentTarget::Empty
        );
    }

    /// Keep the historical test name because the release gate calls it by
    /// exact name. The 0.4.29 model no longer moves the predecessor aside.
    /// It installs a complete immutable replacement first. `CURRENT` remains
    /// the sole commit point, so a crash before that pointer rename must reopen
    /// the predecessor and ignore the complete replacement.
    #[test]
    fn same_seq_resave_crash_between_aside_and_commit_recovers_predecessor() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();

        // The predecessor: a first successful save at seq 7.
        let engine_a = Arc::new(Engine::new());
        engine_a.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine_a, "a1", "a1@x.com");
        store.save(&engine_a, 7).unwrap();
        assert_eq!(store.load_latest().unwrap().unwrap().1, 7);
        let predecessor = current_generation(&store);

        // Prepare the complete replacement and perform the generation rename.
        // Do not change CURRENT. This is the exact pre-commit crash state.
        let engine_b = Arc::new(Engine::new());
        engine_b.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine_b, "b1", "b1@x.com");
        index_kw(&engine_b, "b2", "b2@x.com");
        let replacement = install_unpointed_generation(&store, &engine_b, 7, Some(&predecessor));
        assert!(store.generations.generation_path(&replacement).is_dir());
        assert_eq!(current_generation(&store), predecessor);

        // Cold start from scratch, as a restarted pod does.
        let cold_store = SegmentRdbStore::new(dir.path()).unwrap();
        let (reloaded, seq) = cold_store
            .load_latest()
            .unwrap()
            .expect("a complete generation survives the crash window");
        assert_eq!(seq, 7);
        assert_eq!(
            reloaded.stats("u").unwrap().documents_indexed,
            1,
            "recovered generation must be the predecessor (1 doc), not the \
             never-committed replacement (2 docs) or nothing"
        );

        // A normal same-sequence save activates a new immutable revision.
        cold_store.save(&engine_b, 7).unwrap();
        assert_ne!(current_generation(&cold_store), replacement);
        assert_eq!(
            cold_store
                .load_latest()
                .unwrap()
                .unwrap()
                .0
                .stats("u")
                .unwrap()
                .documents_indexed,
            2
        );
    }

    #[test]
    fn complete_unpointed_higher_generation_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let active_engine = Arc::new(Engine::new());
        active_engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&active_engine, "u1", "a@x.com");
        store.save(&active_engine, 7).unwrap();
        let active = current_generation(&store);

        let later_engine = Arc::new(Engine::new());
        later_engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&later_engine, "u1", "a@x.com");
        index_kw(&later_engine, "u2", "b@x.com");
        install_unpointed_generation(&store, &later_engine, 99, Some(&active));

        let restarted = SegmentRdbStore::new(dir.path()).unwrap();
        let (loaded, sequence) = restarted.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 7);
        assert_eq!(loaded.stats("u").unwrap().documents_indexed, 1);
        assert_eq!(current_generation(&restarted), active);
    }

    #[test]
    fn corrupt_current_manifest_fails_without_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 7).unwrap();
        store.save(&engine, 8).unwrap();
        let current = current_generation(&store);
        let current_path = store.generations.generation_path(&current);
        let mut manifest = read_generation_manifest(&current_path).unwrap();
        manifest.sequence = 999;
        write_generation_manifest(&current_path, &manifest).unwrap();

        let restarted = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(restarted.load_latest().is_err());
        assert_eq!(current_generation(&restarted), current);
    }

    #[test]
    fn unsupported_predecessor_name_blocks_prune_without_deleting_history() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 1).unwrap();
        store.save(&engine, 2).unwrap();
        let current = current_generation(&store);
        let current_path = store.generations.generation_path(&current);
        let mut manifest = read_generation_manifest(&current_path).unwrap();
        let predecessor = manifest.previous.clone().unwrap();
        manifest.previous = Some("missing-safe-name".to_owned());
        write_generation_manifest(&current_path, &manifest).unwrap();

        assert!(store.prune(1).is_err());
        assert!(current_path.is_dir());
        assert!(dir.path().join(predecessor).is_dir());
    }

    #[test]
    fn missing_future_predecessor_blocks_prune_without_deleting_history() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 1).unwrap();
        store.save(&engine, 2).unwrap();
        let current = current_generation(&store);
        let current_path = store.generations.generation_path(&current);
        let mut manifest = read_generation_manifest(&current_path).unwrap();
        let predecessor = manifest.previous.clone().unwrap();
        manifest.previous = Some("gen-999-rev-1".to_owned());
        write_generation_manifest(&current_path, &manifest).unwrap();

        assert!(store.prune(1).is_err());
        assert!(current_path.is_dir());
        assert!(dir.path().join(predecessor).is_dir());
    }

    #[test]
    fn missing_older_predecessor_makes_prune_conservative() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 1).unwrap();
        store.save(&engine, 2).unwrap();
        let current = current_generation(&store);
        let current_path = store.generations.generation_path(&current);
        let mut manifest = read_generation_manifest(&current_path).unwrap();
        let real_predecessor = manifest.previous.clone().unwrap();
        manifest.previous = Some("gen-0-rev-0".to_owned());
        write_generation_manifest(&current_path, &manifest).unwrap();

        assert_eq!(store.prune(1).unwrap(), 0);
        assert!(current_path.is_dir());
        assert!(dir.path().join(real_predecessor).is_dir());
    }

    #[test]
    fn full_reopen_validation_leaves_target_engine_unchanged_on_corruption() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("a", kw_schema()).unwrap();
        source.create_collection("z", kw_schema()).unwrap();
        index_kw_in(&source, "a", "a1", "a@x.com");
        index_kw_in(&source, "z", "z1", "z@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();
        std::fs::write(first_segment_file(&legacy.join("7a")), b"corrupt").unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let target = Arc::new(Engine::new());
        target.create_collection("existing", kw_schema()).unwrap();
        index_kw_in(&target, "existing", "e1", "existing@x.com");

        assert!(store.reopen_into(&target).is_err());
        assert_eq!(
            target.stats("existing").unwrap().documents_indexed,
            1,
            "validation failure must not replace or partly extend the caller engine"
        );
        assert!(target.stats("a").is_err());
        assert!(target.stats("z").is_err());
        assert!(!dir.path().join("CURRENT").exists());
    }

    #[test]
    fn malformed_unpointed_revision_does_not_block_save_or_prune() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 1).unwrap();

        let malformed = dir.path().join("gen-200-rev-999");
        std::fs::create_dir(&malformed).unwrap();
        std::fs::write(malformed.join(GENERATION_MANIFEST_FILE), b"not-json").unwrap();

        store.save(&engine, 2).unwrap();
        assert_eq!(store.load_latest().unwrap().unwrap().1, 2);
        assert_eq!(store.prune(1).unwrap(), 2);
        assert!(!malformed.exists());
        assert_eq!(store.generation_seqs().unwrap(), vec![2]);
    }

    #[test]
    fn prune_follows_active_chain_and_removes_aborted_revision() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 1).unwrap();
        store.save(&engine, 2).unwrap();
        let active = current_generation(&store);
        let aborted = install_unpointed_generation(&store, &engine, 200, Some(&active));
        let aborted_path = store.generations.generation_path(&aborted);
        store.save(&engine, 3).unwrap();

        assert_eq!(store.prune(2).unwrap(), 2);
        assert!(!aborted_path.exists());
        assert_eq!(store.generation_seqs().unwrap(), vec![2, 3]);
        assert_eq!(store.load_latest().unwrap().unwrap().1, 3);
    }

    #[cfg(unix)]
    #[test]
    fn parseable_generation_symlink_fails_closed() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        symlink(target.path(), dir.path().join("gen-99")).unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
    }

    /// #1389 AC1: a `reshard:apply` batch applied to a target shard, and a
    /// `reshard:evict` on a source shard, both survive a cold start from a
    /// checkpoint written after those mutations — independent of any
    /// periodic-snapshot cadence, closing the restart gap `#1387`'s embedded
    /// persistence left open for reshard's direct-state-mutation admin verbs
    /// (`Engine::apply_reshard_batch` / `Engine::evict_not_owned`, added by
    /// `#1380`). This is the engine-level half of `#1389`'s proof; the
    /// driver-level half (cutover cannot fire before every touched shard's
    /// checkpoint completes) lives in `e2e/reshard_driver_e2e.rs`.
    #[test]
    fn reshard_apply_and_evict_survive_checkpoint_and_cold_start() {
        use crate::routing::VirtualBucketShardMap;

        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();

        // Target shard: receives a reshard:apply batch on top of its own
        // pre-existing data — mirrors what a shard actually looks like
        // mid-migration.
        let target = Arc::new(Engine::new());
        target.create_collection("u", kw_schema()).unwrap();
        index_kw(&target, "t-existing", "existing@x.com");

        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "migrated-1", "migrated1@x.com");
        let batch = source.snapshot().unwrap();
        let apply_outcome = target.apply_reshard_batch(batch, None).unwrap();
        assert_eq!(apply_outcome.documents_upserted, 1);
        assert_eq!(target.stats("u").unwrap().documents_indexed, 2);

        // Source shard: post-cutover eviction of the bucket that just moved
        // off of it, under a 2-shard map where bucket 0 now belongs to shard
        // 1 (mirrors `reshard_evict_removes_only_moved_bucket_docs`).
        let source_after_cutover = Arc::new(Engine::new());
        source_after_cutover
            .create_collection("u", kw_schema())
            .unwrap();
        let ids: Vec<String> = (0..8).map(|i| format!("s-{i:02}")).collect();
        for id in &ids {
            index_kw(&source_after_cutover, id, &format!("{id}@x.com"));
        }
        let mut assignments = vec![0u32; 4];
        assignments[0] = 1;
        let new_map = VirtualBucketShardMap::new(1, assignments, 2).unwrap();
        let evict_outcome = source_after_cutover.evict_not_owned(&new_map, 0).unwrap();
        assert!(evict_outcome.documents_evicted > 0);
        let remaining_before_checkpoint =
            source_after_cutover.stats("u").unwrap().documents_indexed;
        assert!(remaining_before_checkpoint < ids.len() as u64);

        // Checkpoint both post-mutation states, exactly like
        // `checkpoint_touched_shards` (#1389) drives per shard before
        // cutover — this is the synchronous, awaited durability step, not a
        // background snapshot the driver has no visibility into.
        store.save(&target, 100).unwrap();
        let target_docs_before_drop = target.stats("u").unwrap().documents_indexed;
        drop(target);

        let store2 = SegmentRdbStore::new(dir.path().join("source")).unwrap();
        store2.save(&source_after_cutover, 100).unwrap();
        drop(source_after_cutover);

        // Cold start: reload from the checkpoint alone, as a restarted pod
        // would (WAL replay from `seq + 1` is orthogonal to this proof —
        // there are no un-checkpointed writes here).
        let (reloaded_target, seq) = store.load_latest().unwrap().expect("target checkpoint");
        assert_eq!(seq, 100);
        assert_eq!(
            reloaded_target.stats("u").unwrap().documents_indexed,
            target_docs_before_drop
        );

        let (reloaded_source, seq2) = store2.load_latest().unwrap().expect("source checkpoint");
        assert_eq!(seq2, 100);
        assert_eq!(
            reloaded_source.stats("u").unwrap().documents_indexed,
            remaining_before_checkpoint
        );
    }

    /// #1397 AC1: `POST /admin/checkpoint` (the checkpoint sink) and the
    /// periodic snapshotter share one `SegmentRdbStore` and can both fire at
    /// an unchanged `applied_seq` (reshard apply/evict mutate engine state
    /// without advancing `applied_seq`, so this is a routine, not a rare,
    /// interleaving). Loop the interleaving many rounds with several
    /// concurrent `save` callers per round: every round must cold-start to a
    /// complete engine, never a torn one — proving `save_lock` actually
    /// prevents `sweep_staging`/`rename` races rather than merely narrowing
    /// them.
    #[test]
    fn concurrent_saves_at_same_seq_never_produce_torn_checkpoint() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        for i in 0..20 {
            index_kw(&engine, &format!("u{i:02}"), &format!("u{i:02}@x.com"));
        }
        let expected_docs = engine.stats("u").unwrap().documents_indexed;

        for round in 0..50u64 {
            // Same `up_to_seq` across every concurrent caller this round,
            // mirroring a quiet cutover where `applied_seq` hasn't moved.
            let seq = round;
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let store = store.clone();
                    let engine = engine.clone();
                    std::thread::spawn(move || store.save(&engine, seq))
                })
                .collect();
            for h in handles {
                h.join().unwrap().unwrap();
            }

            // Cold-start from scratch after the interleaving: the committed
            // generation must always be complete and loadable, never torn.
            let (reloaded, loaded_seq) = store.load_latest().unwrap().expect("a checkpoint");
            assert_eq!(loaded_seq, seq);
            assert_eq!(
                reloaded.stats("u").unwrap().documents_indexed,
                expected_docs,
                "round {round}: cold start after concurrent saves must be complete"
            );
        }
    }

    #[test]
    fn independently_opened_handles_share_checkpoint_preparation_lock() {
        let dir = tempfile::tempdir().unwrap();
        let first = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let second = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");

        for sequence in 1..=10 {
            let left = {
                let store = first.clone();
                let engine = engine.clone();
                std::thread::spawn(move || store.save(&engine, sequence))
            };
            let right = {
                let store = second.clone();
                let engine = engine.clone();
                std::thread::spawn(move || store.save(&engine, sequence))
            };
            left.join().unwrap().unwrap();
            right.join().unwrap().unwrap();
        }

        assert_eq!(first.load_latest().unwrap().unwrap().1, 10);
        assert!(!std::fs::read_dir(dir.path()).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with(".stage-"))
        }));
    }
}
// CODEGEN-END
