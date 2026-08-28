// CODEGEN-BEGIN
//! Durable activation for immutable directory generations.
//!
//! The caller owns the bytes and their domain validation. This module owns the
//! filesystem transaction that makes one complete generation current. The
//! rename of `CURRENT.tmp` over `CURRENT` is the only activation commit point.

use std::collections::HashMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};

/// The durable pointer file.
pub const CURRENT_FILE_NAME: &str = "CURRENT";
/// The sibling used to prepare a new durable pointer.
pub const CURRENT_TEMP_FILE_NAME: &str = "CURRENT.tmp";
/// The exact pointer bytes for an initialized store with no generation.
pub const EMPTY_CURRENT_BYTES: &[u8] = b"empty\n";

const CURRENT_PREFIX: &[u8] = b"generation:";
const MAX_GENERATION_NAME_BYTES: usize = 128;
const MAX_CURRENT_BYTES: usize = 256;

/// One safe direct-child generation directory name.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GenerationName(String);

impl GenerationName {
    /// Parse a generation name that cannot escape the store root.
    pub fn parse(value: impl Into<String>) -> Result<Self, GenerationNameError> {
        let value = value.into();
        let bytes = value.as_bytes();
        let kind = if bytes.is_empty() {
            Some(GenerationNameErrorKind::Empty)
        } else if bytes.len() > MAX_GENERATION_NAME_BYTES {
            Some(GenerationNameErrorKind::TooLong)
        } else if value == CURRENT_FILE_NAME || value == CURRENT_TEMP_FILE_NAME {
            Some(GenerationNameErrorKind::Reserved)
        } else if !bytes[0].is_ascii_alphanumeric()
            || !bytes[1..]
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'_' | b'-'))
        {
            Some(GenerationNameErrorKind::InvalidCharacter)
        } else {
            None
        };

        match kind {
            Some(kind) => Err(GenerationNameError { kind }),
            None => Ok(Self(value)),
        }
    }

    /// Return the validated name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GenerationName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Why a generation name is unsafe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationNameErrorKind {
    Empty,
    TooLong,
    InvalidCharacter,
    Reserved,
}

/// A rejected generation name.
#[derive(Debug)]
pub struct GenerationNameError {
    pub kind: GenerationNameErrorKind,
}

impl fmt::Display for GenerationNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid generation name: {:?}", self.kind)
    }
}

impl std::error::Error for GenerationNameError {}

/// The target selected by `CURRENT`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrentTarget {
    Empty,
    Generation(GenerationName),
}

/// Stable classification for a rejected `CURRENT` pointer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrentReadErrorKind {
    Missing,
    PointerNotRegular,
    PointerTooLarge,
    Malformed,
    UnsafeTarget,
    TargetMissing,
    TargetNotDirectory,
    TargetSymlink,
    Io,
}

/// A `CURRENT` read failure. It never triggers generation auto-selection.
#[derive(Debug)]
pub struct CurrentReadError {
    pub kind: CurrentReadErrorKind,
    pub path: PathBuf,
    pub source: Option<io::Error>,
}

impl fmt::Display for CurrentReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "read durable current pointer {}: {:?}",
            self.path.display(),
            self.kind
        )
    }
}

impl std::error::Error for CurrentReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source as &(dyn std::error::Error + 'static))
    }
}

/// One load-bearing operation in a generation mutation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CommitStep {
    ValidateCurrent,
    ValidateStaging,
    SyncFile,
    SyncDirectory,
    RenameGeneration,
    SyncRootAfterGeneration,
    RemoveStaleCurrentTemp,
    SyncRootAfterTempCleanup,
    CreateCurrentTemp,
    WriteCurrentTemp,
    SyncCurrentTemp,
    RenameCurrent,
    SyncRootAfterCurrent,
}

/// Whether the activation commit point definitely ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitFailureClass {
    PreCommit,
    CommitUncertain,
}

/// One deterministic injection point.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailurePoint {
    pub step: CommitStep,
    pub occurrence: usize,
    pub relative_path: PathBuf,
}

/// Test seam for failures immediately before load-bearing filesystem calls.
pub trait FailureInjector: Send + Sync {
    fn check(&self, point: &FailurePoint) -> io::Result<()>;
}

/// Production injector that never fails.
#[derive(Debug, Default)]
pub struct NoFailures;

impl FailureInjector for NoFailures {
    fn check(&self, _: &FailurePoint) -> io::Result<()> {
        Ok(())
    }
}

/// A durable generation mutation failure.
#[derive(Debug)]
pub struct CommitError {
    class: CommitFailureClass,
    step: CommitStep,
    target: CurrentTarget,
    path: PathBuf,
    source: io::Error,
}

impl CommitError {
    pub fn class(&self) -> CommitFailureClass {
        self.class
    }

    pub fn step(&self) -> CommitStep {
        self.step
    }

    pub fn target(&self) -> &CurrentTarget {
        &self.target
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn io_error(&self) -> &io::Error {
        &self.source
    }
}

impl fmt::Display for CommitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "durable generation {:?} failure at {:?} for {}: {}",
            self.class,
            self.step,
            self.path.display(),
            self.source
        )
    }
}

impl std::error::Error for CommitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// A caller-owned staging directory. It is consumed by `commit`.
///
/// The caller must close every writer, stop changing this directory, and stop
/// writes through any external hard link before it calls
/// [`GenerationStore::commit`]. The store validates and syncs a quiescent tree.
/// It does not coordinate handles retained by the caller.
#[derive(Debug)]
pub struct StagedGeneration {
    generation: GenerationName,
    path: PathBuf,
    root: PathBuf,
}

impl StagedGeneration {
    pub fn generation(&self) -> &GenerationName {
        &self.generation
    }

    /// Return the directory that the caller may populate before `commit`.
    ///
    /// Stop all changes and close every writer before passing this value to
    /// [`GenerationStore::commit`].
    pub fn path(&self) -> &Path {
        &self.path
    }
}

struct GenerationStoreInner {
    root: PathBuf,
    injector: Arc<dyn FailureInjector>,
    commit: Arc<Mutex<()>>,
}

/// An opt-in durable store for immutable directory generations.
///
/// Stores opened for the same canonical root inside one process share a commit
/// mutex. A single process must own all mutations for a root. Independent
/// writers in other processes are unsupported because this API does not take
/// an operating-system file lock.
#[derive(Clone)]
pub struct GenerationStore {
    inner: Arc<GenerationStoreInner>,
}

impl GenerationStore {
    /// Open an existing real directory with production filesystem behavior.
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Self> {
        Self::open_with_injector(root, Arc::new(NoFailures))
    }

    /// Open an existing real directory with deterministic failure injection.
    pub fn open_with_injector(
        root: impl Into<PathBuf>,
        injector: Arc<dyn FailureInjector>,
    ) -> io::Result<Self> {
        let root = root.into();
        let metadata = std::fs::symlink_metadata(&root)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "generation root must be a real directory: {}",
                    root.display()
                ),
            ));
        }
        let root = std::fs::canonicalize(root)?;
        let commit = shared_root_lock(&root)?;
        Ok(Self {
            inner: Arc::new(GenerationStoreInner {
                root,
                injector,
                commit,
            }),
        })
    }

    /// Create one unique direct-child staging directory.
    pub fn begin(&self, generation: GenerationName) -> io::Result<StagedGeneration> {
        let _guard = self.lock_commit()?;
        let target = self.generation_path(&generation);
        if path_exists(&target)? {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("generation already exists: {}", target.display()),
            ));
        }
        let stage = self.stage_path(&generation);
        std::fs::create_dir(&stage)?;
        Ok(StagedGeneration {
            generation,
            path: stage,
            root: self.inner.root.clone(),
        })
    }

    /// Durably activate a complete staged generation.
    ///
    /// Before this call, the caller must close its writers and stop all changes
    /// under [`StagedGeneration::path`]. The caller must also ensure that no
    /// other process mutates this store root.
    pub fn commit(&self, staged: StagedGeneration) -> Result<GenerationName, CommitError> {
        let _guard = self.lock_commit().map_err(|error| {
            self.commit_error(
                CommitFailureClass::PreCommit,
                CommitStep::ValidateStaging,
                CurrentTarget::Generation(staged.generation.clone()),
                &staged.path,
                error,
            )
        })?;
        let target = CurrentTarget::Generation(staged.generation.clone());
        let mut mutation = Mutation::new(self.inner.injector.as_ref());
        if let Err(error) = read_current_from(&self.inner.root, |relative| {
            mutation.check(CommitStep::ValidateCurrent, relative)
        }) {
            let path = error.path.clone();
            return Err(self.commit_error(
                CommitFailureClass::PreCommit,
                CommitStep::ValidateCurrent,
                target,
                path,
                current_read_as_io(error),
            ));
        }

        self.validate_staged(&staged, &target, &mut mutation)?;
        self.sync_tree(&staged.path, &target, &mut mutation)?;

        let final_path = self.generation_path(&staged.generation);
        self.checked(
            &mut mutation,
            CommitStep::RenameGeneration,
            Path::new(staged.generation.as_str()),
            &target,
            &final_path,
            || std::fs::rename(&staged.path, &final_path),
        )?;
        self.sync_root(
            &mut mutation,
            CommitStep::SyncRootAfterGeneration,
            &target,
            CommitFailureClass::PreCommit,
        )?;

        self.commit_pointer(&target, &mut mutation)?;
        Ok(staged.generation)
    }

    /// Resolve only the exact target named by `CURRENT`.
    pub fn read_current(&self) -> Result<CurrentTarget, CurrentReadError> {
        read_current_from(&self.inner.root, |_| Ok(()))
    }

    /// Initialize an empty store. Missing `CURRENT` never implies this state.
    pub fn initialize_empty(&self) -> Result<(), CommitError> {
        let _guard = self.lock_commit().map_err(|error| {
            self.commit_error(
                CommitFailureClass::PreCommit,
                CommitStep::ValidateCurrent,
                CurrentTarget::Empty,
                &self.inner.root,
                error,
            )
        })?;
        let target = CurrentTarget::Empty;
        let mut mutation = Mutation::new(self.inner.injector.as_ref());
        match read_current_from(&self.inner.root, |relative| {
            mutation.check(CommitStep::ValidateCurrent, relative)
        }) {
            Err(error) if error.kind == CurrentReadErrorKind::Missing => {}
            Ok(_) => {
                return Err(self.commit_error(
                    CommitFailureClass::PreCommit,
                    CommitStep::ValidateCurrent,
                    target,
                    self.inner.root.join(CURRENT_FILE_NAME),
                    io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "CURRENT is already initialized",
                    ),
                ));
            }
            Err(error) => {
                let path = error.path.clone();
                return Err(self.commit_error(
                    CommitFailureClass::PreCommit,
                    CommitStep::ValidateCurrent,
                    target,
                    path,
                    current_read_as_io(error),
                ));
            }
        }

        mutation
            .check(CommitStep::ValidateCurrent, Path::new("."))
            .map_err(|error| {
                self.commit_error(
                    CommitFailureClass::PreCommit,
                    CommitStep::ValidateCurrent,
                    target.clone(),
                    &self.inner.root,
                    error,
                )
            })?;
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&self.inner.root)
            .map_err(|error| {
                self.commit_error(
                    CommitFailureClass::PreCommit,
                    CommitStep::ValidateCurrent,
                    target.clone(),
                    &self.inner.root,
                    error,
                )
            })?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<io::Result<_>>()
            .map_err(|error| {
                self.commit_error(
                    CommitFailureClass::PreCommit,
                    CommitStep::ValidateCurrent,
                    target.clone(),
                    &self.inner.root,
                    error,
                )
            })?;
        entries.sort();
        for entry in entries {
            let relative = entry
                .file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            mutation
                .check(CommitStep::ValidateCurrent, &relative)
                .map_err(|error| {
                    self.commit_error(
                        CommitFailureClass::PreCommit,
                        CommitStep::ValidateCurrent,
                        target.clone(),
                        &entry,
                        error,
                    )
                })?;
            let metadata = std::fs::symlink_metadata(&entry).map_err(|error| {
                self.commit_error(
                    CommitFailureClass::PreCommit,
                    CommitStep::ValidateCurrent,
                    target.clone(),
                    &entry,
                    error,
                )
            })?;
            let valid_generation = entry
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| GenerationName::parse(name.to_owned()).ok())
                .is_some();
            if metadata.is_dir() && !metadata.file_type().is_symlink() && valid_generation {
                return Err(self.commit_error(
                    CommitFailureClass::PreCommit,
                    CommitStep::ValidateCurrent,
                    target,
                    &entry,
                    io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "legacy generation blocks empty initialization",
                    ),
                ));
            }
        }

        self.commit_pointer(&CurrentTarget::Empty, &mut mutation)
    }

    /// Point an uninitialized store at one caller-validated legacy generation.
    pub fn adopt_legacy(&self, generation: GenerationName) -> Result<(), CommitError> {
        let _guard = self.lock_commit().map_err(|error| {
            self.commit_error(
                CommitFailureClass::PreCommit,
                CommitStep::ValidateCurrent,
                CurrentTarget::Generation(generation.clone()),
                &self.inner.root,
                error,
            )
        })?;
        let target = CurrentTarget::Generation(generation.clone());
        let mut mutation = Mutation::new(self.inner.injector.as_ref());
        match read_current_from(&self.inner.root, |relative| {
            mutation.check(CommitStep::ValidateCurrent, relative)
        }) {
            Err(error) if error.kind == CurrentReadErrorKind::Missing => {}
            Ok(_) => {
                return Err(self.commit_error(
                    CommitFailureClass::PreCommit,
                    CommitStep::ValidateCurrent,
                    target,
                    self.inner.root.join(CURRENT_FILE_NAME),
                    io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "CURRENT is already initialized",
                    ),
                ));
            }
            Err(error) => {
                let path = error.path.clone();
                return Err(self.commit_error(
                    CommitFailureClass::PreCommit,
                    CommitStep::ValidateCurrent,
                    target,
                    path,
                    current_read_as_io(error),
                ));
            }
        }

        let generation_path = self.generation_path(&generation);
        self.checked(
            &mut mutation,
            CommitStep::ValidateStaging,
            Path::new(generation.as_str()),
            &target,
            &generation_path,
            || validate_real_directory(&generation_path),
        )?;
        self.sync_tree(&generation_path, &target, &mut mutation)?;
        self.sync_root(
            &mut mutation,
            CommitStep::SyncRootAfterGeneration,
            &target,
            CommitFailureClass::PreCommit,
        )?;
        self.commit_pointer(&target, &mut mutation)
    }

    /// Return the exact direct-child path for one generation.
    pub fn generation_path(&self, generation: &GenerationName) -> PathBuf {
        self.inner.root.join(generation.as_str())
    }

    fn lock_commit(&self) -> io::Result<std::sync::MutexGuard<'_, ()>> {
        self.inner
            .commit
            .lock()
            .map_err(|_| io::Error::other("generation commit mutex poisoned"))
    }

    fn stage_path(&self, generation: &GenerationName) -> PathBuf {
        self.inner
            .root
            .join(format!(".stage-{}", generation.as_str()))
    }

    fn validate_staged(
        &self,
        staged: &StagedGeneration,
        target: &CurrentTarget,
        mutation: &mut Mutation<'_>,
    ) -> Result<(), CommitError> {
        let expected = self.stage_path(&staged.generation);
        if staged.root != self.inner.root || staged.path != expected {
            return Err(self.commit_error(
                CommitFailureClass::PreCommit,
                CommitStep::ValidateStaging,
                target.clone(),
                &staged.path,
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "staged generation belongs to another store",
                ),
            ));
        }
        let stage_relative = PathBuf::from(format!(".stage-{}", staged.generation));
        self.checked(
            mutation,
            CommitStep::ValidateStaging,
            &stage_relative,
            target,
            &staged.path,
            || validate_real_directory(&staged.path),
        )?;
        let final_path = self.generation_path(&staged.generation);
        let exists = self.checked_value(
            mutation,
            CommitStep::ValidateStaging,
            Path::new(staged.generation.as_str()),
            target,
            &final_path,
            || path_exists(&final_path),
        )?;
        if exists {
            return Err(self.commit_error(
                CommitFailureClass::PreCommit,
                CommitStep::ValidateStaging,
                target.clone(),
                final_path,
                io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "immutable generation already exists",
                ),
            ));
        }
        Ok(())
    }

    fn sync_tree(
        &self,
        tree_root: &Path,
        target: &CurrentTarget,
        mutation: &mut Mutation<'_>,
    ) -> Result<(), CommitError> {
        let entries = collect_tree(tree_root, |relative| {
            mutation.check(CommitStep::ValidateStaging, relative)
        })
        .map_err(|(path, error)| {
            self.commit_error(
                CommitFailureClass::PreCommit,
                CommitStep::ValidateStaging,
                target.clone(),
                path,
                error,
            )
        })?;

        for relative in &entries.files {
            let absolute = tree_root.join(relative);
            self.checked(
                mutation,
                CommitStep::SyncFile,
                relative,
                target,
                &absolute,
                || File::open(&absolute).and_then(|file| file.sync_all()),
            )?;
        }
        for relative in &entries.directories {
            let absolute = if relative == Path::new(".") {
                tree_root.to_path_buf()
            } else {
                tree_root.join(relative)
            };
            self.checked(
                mutation,
                CommitStep::SyncDirectory,
                relative,
                target,
                &absolute,
                || strict_sync_directory(&absolute),
            )?;
        }
        Ok(())
    }

    fn commit_pointer(
        &self,
        target: &CurrentTarget,
        mutation: &mut Mutation<'_>,
    ) -> Result<(), CommitError> {
        let current = self.inner.root.join(CURRENT_FILE_NAME);
        let temp = self.inner.root.join(CURRENT_TEMP_FILE_NAME);
        mutation
            .check(
                CommitStep::RemoveStaleCurrentTemp,
                Path::new(CURRENT_TEMP_FILE_NAME),
            )
            .map_err(|error| {
                self.commit_error(
                    CommitFailureClass::PreCommit,
                    CommitStep::RemoveStaleCurrentTemp,
                    target.clone(),
                    &temp,
                    error,
                )
            })?;
        match std::fs::symlink_metadata(&temp) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    return Err(self.commit_error(
                        CommitFailureClass::PreCommit,
                        CommitStep::RemoveStaleCurrentTemp,
                        target.clone(),
                        &temp,
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "CURRENT.tmp must be a regular file",
                        ),
                    ));
                }
                self.checked(
                    mutation,
                    CommitStep::RemoveStaleCurrentTemp,
                    Path::new(CURRENT_TEMP_FILE_NAME),
                    target,
                    &temp,
                    || std::fs::remove_file(&temp),
                )?;
                self.sync_root(
                    mutation,
                    CommitStep::SyncRootAfterTempCleanup,
                    target,
                    CommitFailureClass::PreCommit,
                )?;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(self.commit_error(
                    CommitFailureClass::PreCommit,
                    CommitStep::RemoveStaleCurrentTemp,
                    target.clone(),
                    &temp,
                    error,
                ));
            }
        }

        let mut file = self.checked_value(
            mutation,
            CommitStep::CreateCurrentTemp,
            Path::new(CURRENT_TEMP_FILE_NAME),
            target,
            &temp,
            || OpenOptions::new().write(true).create_new(true).open(&temp),
        )?;
        let bytes = current_bytes(target);
        self.checked(
            mutation,
            CommitStep::WriteCurrentTemp,
            Path::new(CURRENT_TEMP_FILE_NAME),
            target,
            &temp,
            || file.write_all(&bytes),
        )?;
        self.checked(
            mutation,
            CommitStep::SyncCurrentTemp,
            Path::new(CURRENT_TEMP_FILE_NAME),
            target,
            &temp,
            || file.sync_all(),
        )?;
        drop(file);
        self.checked(
            mutation,
            CommitStep::RenameCurrent,
            Path::new(CURRENT_FILE_NAME),
            target,
            &current,
            || std::fs::rename(&temp, &current),
        )?;
        self.sync_root(
            mutation,
            CommitStep::SyncRootAfterCurrent,
            target,
            CommitFailureClass::CommitUncertain,
        )
    }

    fn sync_root(
        &self,
        mutation: &mut Mutation<'_>,
        step: CommitStep,
        target: &CurrentTarget,
        class: CommitFailureClass,
    ) -> Result<(), CommitError> {
        let relative = Path::new(".");
        mutation
            .check(step, relative)
            .and_then(|_| strict_sync_directory(&self.inner.root))
            .map_err(|error| {
                self.commit_error(class, step, target.clone(), &self.inner.root, error)
            })
    }

    fn checked(
        &self,
        mutation: &mut Mutation<'_>,
        step: CommitStep,
        relative: &Path,
        target: &CurrentTarget,
        absolute: &Path,
        operation: impl FnOnce() -> io::Result<()>,
    ) -> Result<(), CommitError> {
        self.checked_value(mutation, step, relative, target, absolute, operation)
    }

    fn checked_value<T>(
        &self,
        mutation: &mut Mutation<'_>,
        step: CommitStep,
        relative: &Path,
        target: &CurrentTarget,
        absolute: &Path,
        operation: impl FnOnce() -> io::Result<T>,
    ) -> Result<T, CommitError> {
        mutation
            .check(step, relative)
            .and_then(|_| operation())
            .map_err(|error| {
                self.commit_error(
                    CommitFailureClass::PreCommit,
                    step,
                    target.clone(),
                    absolute,
                    error,
                )
            })
    }

    fn commit_error(
        &self,
        class: CommitFailureClass,
        step: CommitStep,
        target: CurrentTarget,
        path: impl AsRef<Path>,
        source: io::Error,
    ) -> CommitError {
        CommitError {
            class,
            step,
            target,
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

struct Mutation<'a> {
    injector: &'a dyn FailureInjector,
    occurrences: HashMap<CommitStep, usize>,
}

impl<'a> Mutation<'a> {
    fn new(injector: &'a dyn FailureInjector) -> Self {
        Self {
            injector,
            occurrences: HashMap::new(),
        }
    }

    fn check(&mut self, step: CommitStep, relative_path: &Path) -> io::Result<()> {
        let occurrence = self.occurrences.entry(step).or_default();
        let point = FailurePoint {
            step,
            occurrence: *occurrence,
            relative_path: relative_path.to_path_buf(),
        };
        *occurrence += 1;
        self.injector.check(&point)
    }
}

struct TreeEntries {
    files: Vec<PathBuf>,
    directories: Vec<PathBuf>,
}

fn shared_root_lock(root: &Path) -> io::Result<Arc<Mutex<()>>> {
    static ROOT_LOCKS: OnceLock<Mutex<HashMap<PathBuf, Weak<Mutex<()>>>>> = OnceLock::new();

    let registry = ROOT_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut registry = registry
        .lock()
        .map_err(|_| io::Error::other("generation root-lock registry poisoned"))?;
    registry.retain(|_, lock| lock.strong_count() > 0);
    if let Some(lock) = registry.get(root).and_then(Weak::upgrade) {
        return Ok(lock);
    }
    let lock = Arc::new(Mutex::new(()));
    registry.insert(root.to_path_buf(), Arc::downgrade(&lock));
    Ok(lock)
}

fn collect_tree(
    root: &Path,
    mut before: impl FnMut(&Path) -> io::Result<()>,
) -> Result<TreeEntries, (PathBuf, io::Error)> {
    before(Path::new(".")).map_err(|error| (root.to_path_buf(), error))?;
    validate_real_directory(root).map_err(|error| (root.to_path_buf(), error))?;
    let mut files = Vec::new();
    let mut directories = vec![PathBuf::from(".")];
    collect_directory(
        root,
        Path::new(""),
        &mut files,
        &mut directories,
        &mut before,
    )?;
    files.sort();
    directories.sort_by(|left, right| {
        directory_depth(right)
            .cmp(&directory_depth(left))
            .then_with(|| left.cmp(right))
    });
    Ok(TreeEntries { files, directories })
}

fn collect_directory(
    root: &Path,
    relative: &Path,
    files: &mut Vec<PathBuf>,
    directories: &mut Vec<PathBuf>,
    before: &mut impl FnMut(&Path) -> io::Result<()>,
) -> Result<(), (PathBuf, io::Error)> {
    let directory = if relative.as_os_str().is_empty() {
        root.to_path_buf()
    } else {
        root.join(relative)
    };
    let directory_relative = if relative.as_os_str().is_empty() {
        Path::new(".")
    } else {
        relative
    };
    before(directory_relative).map_err(|error| (directory.clone(), error))?;
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&directory)
        .map_err(|error| (directory.clone(), error))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<_>>()
        .map_err(|error| (directory.clone(), error))?;
    entries.sort();
    for child in entries {
        let child_relative = relative.join(PathBuf::from(child.file_name().ok_or_else(|| {
            (
                child.clone(),
                io::Error::new(io::ErrorKind::InvalidData, "directory entry has no name"),
            )
        })?));
        before(&child_relative).map_err(|error| (child.clone(), error))?;
        let metadata = std::fs::symlink_metadata(&child).map_err(|error| (child.clone(), error))?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            return Err((
                child,
                io::Error::new(io::ErrorKind::InvalidData, "generation contains a symlink"),
            ));
        }
        if metadata.is_file() {
            files.push(child_relative);
        } else if metadata.is_dir() {
            directories.push(child_relative.clone());
            collect_directory(root, &child_relative, files, directories, before)?;
        } else {
            return Err((
                child,
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "generation contains a special filesystem entry",
                ),
            ));
        }
    }
    Ok(())
}

fn directory_depth(path: &Path) -> usize {
    if path == Path::new(".") {
        0
    } else {
        path.components().count()
    }
}

fn validate_real_directory(path: &Path) -> io::Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("expected real directory: {}", path.display()),
        ));
    }
    Ok(())
}

fn strict_sync_directory(path: &Path) -> io::Result<()> {
    OpenOptions::new().read(true).open(path)?.sync_all()
}

fn path_exists(path: &Path) -> io::Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn current_bytes(target: &CurrentTarget) -> Vec<u8> {
    match target {
        CurrentTarget::Empty => EMPTY_CURRENT_BYTES.to_vec(),
        CurrentTarget::Generation(generation) => {
            format!("generation:{}\n", generation.as_str()).into_bytes()
        }
    }
}

fn read_current_from(
    root: &Path,
    mut before: impl FnMut(&Path) -> io::Result<()>,
) -> Result<CurrentTarget, CurrentReadError> {
    let pointer = root.join(CURRENT_FILE_NAME);
    before(Path::new(CURRENT_FILE_NAME)).map_err(|error| CurrentReadError {
        kind: CurrentReadErrorKind::Io,
        path: pointer.clone(),
        source: Some(error),
    })?;
    let metadata = match std::fs::symlink_metadata(&pointer) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(CurrentReadError {
                kind: CurrentReadErrorKind::Missing,
                path: pointer,
                source: Some(error),
            });
        }
        Err(error) => {
            return Err(CurrentReadError {
                kind: CurrentReadErrorKind::Io,
                path: pointer,
                source: Some(error),
            });
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(CurrentReadError {
            kind: CurrentReadErrorKind::PointerNotRegular,
            path: pointer,
            source: None,
        });
    }
    if metadata.len() > MAX_CURRENT_BYTES as u64 {
        return Err(CurrentReadError {
            kind: CurrentReadErrorKind::PointerTooLarge,
            path: pointer,
            source: None,
        });
    }

    before(Path::new(CURRENT_FILE_NAME)).map_err(|error| CurrentReadError {
        kind: CurrentReadErrorKind::Io,
        path: pointer.clone(),
        source: Some(error),
    })?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    let read = File::open(&pointer).and_then(|file| {
        file.take((MAX_CURRENT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
    });
    if let Err(error) = read {
        return Err(CurrentReadError {
            kind: CurrentReadErrorKind::Io,
            path: pointer,
            source: Some(error),
        });
    }
    if bytes.len() > MAX_CURRENT_BYTES {
        return Err(CurrentReadError {
            kind: CurrentReadErrorKind::PointerTooLarge,
            path: pointer,
            source: None,
        });
    }
    if bytes == EMPTY_CURRENT_BYTES {
        return Ok(CurrentTarget::Empty);
    }

    let Some(raw_name) = bytes
        .strip_prefix(CURRENT_PREFIX)
        .and_then(|value| value.strip_suffix(b"\n"))
    else {
        return Err(CurrentReadError {
            kind: CurrentReadErrorKind::Malformed,
            path: pointer,
            source: None,
        });
    };
    let raw_name = std::str::from_utf8(raw_name).map_err(|_| CurrentReadError {
        kind: CurrentReadErrorKind::Malformed,
        path: pointer.clone(),
        source: None,
    })?;
    let generation = GenerationName::parse(raw_name.to_owned()).map_err(|_| CurrentReadError {
        kind: CurrentReadErrorKind::UnsafeTarget,
        path: pointer.clone(),
        source: None,
    })?;
    let target = root.join(generation.as_str());
    before(Path::new(generation.as_str())).map_err(|error| CurrentReadError {
        kind: CurrentReadErrorKind::Io,
        path: target.clone(),
        source: Some(error),
    })?;
    let target_metadata = match std::fs::symlink_metadata(&target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(CurrentReadError {
                kind: CurrentReadErrorKind::TargetMissing,
                path: target,
                source: Some(error),
            });
        }
        Err(error) => {
            return Err(CurrentReadError {
                kind: CurrentReadErrorKind::Io,
                path: target,
                source: Some(error),
            });
        }
    };
    if target_metadata.file_type().is_symlink() {
        return Err(CurrentReadError {
            kind: CurrentReadErrorKind::TargetSymlink,
            path: target,
            source: None,
        });
    }
    if !target_metadata.is_dir() {
        return Err(CurrentReadError {
            kind: CurrentReadErrorKind::TargetNotDirectory,
            path: target,
            source: None,
        });
    }
    Ok(CurrentTarget::Generation(generation))
}

fn current_read_as_io(error: CurrentReadError) -> io::Error {
    let kind = match error.kind {
        CurrentReadErrorKind::Missing | CurrentReadErrorKind::TargetMissing => {
            io::ErrorKind::NotFound
        }
        CurrentReadErrorKind::PointerNotRegular
        | CurrentReadErrorKind::PointerTooLarge
        | CurrentReadErrorKind::Malformed
        | CurrentReadErrorKind::UnsafeTarget
        | CurrentReadErrorKind::TargetNotDirectory
        | CurrentReadErrorKind::TargetSymlink => io::ErrorKind::InvalidData,
        CurrentReadErrorKind::Io => error
            .source
            .as_ref()
            .map(io::Error::kind)
            .unwrap_or(io::ErrorKind::Other),
    };
    io::Error::new(kind, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Clone, Debug)]
    struct InjectedFailure {
        point: FailurePoint,
        kind: io::ErrorKind,
    }

    #[derive(Default)]
    struct RecordingInjector {
        points: Mutex<Vec<FailurePoint>>,
        failure: Mutex<Option<InjectedFailure>>,
    }

    impl RecordingInjector {
        fn points(&self) -> Vec<FailurePoint> {
            self.points.lock().unwrap().clone()
        }

        fn fail_at(&self, point: FailurePoint, kind: io::ErrorKind) {
            *self.failure.lock().unwrap() = Some(InjectedFailure { point, kind });
        }
    }

    impl FailureInjector for RecordingInjector {
        fn check(&self, point: &FailurePoint) -> io::Result<()> {
            self.points.lock().unwrap().push(point.clone());
            let failure = self.failure.lock().unwrap().clone();
            if failure
                .as_ref()
                .is_some_and(|failure| failure.point == *point)
            {
                return Err(io::Error::from(failure.unwrap().kind));
            }
            Ok(())
        }
    }

    fn name(value: &str) -> GenerationName {
        GenerationName::parse(value).unwrap()
    }

    fn seed_current(root: &Path, generation: &str) {
        let generation_path = root.join(generation);
        std::fs::create_dir(&generation_path).unwrap();
        std::fs::write(generation_path.join("payload"), generation.as_bytes()).unwrap();
        std::fs::write(
            root.join(CURRENT_FILE_NAME),
            format!("generation:{generation}\n"),
        )
        .unwrap();
    }

    fn stage_fixture(store: &GenerationStore, generation: &str) -> StagedGeneration {
        let staged = store.begin(name(generation)).unwrap();
        std::fs::write(staged.path().join("z.txt"), b"z").unwrap();
        std::fs::write(staged.path().join("a.txt"), b"a").unwrap();
        std::fs::create_dir(staged.path().join("b")).unwrap();
        std::fs::write(staged.path().join("b/m.txt"), b"m").unwrap();
        std::fs::create_dir(staged.path().join("b/c")).unwrap();
        std::fs::write(staged.path().join("b/c/n.txt"), b"n").unwrap();
        std::fs::create_dir(staged.path().join("a-dir")).unwrap();
        staged
    }

    fn instrumented_fixture(
        stale_temp: bool,
    ) -> (
        tempfile::TempDir,
        Arc<RecordingInjector>,
        GenerationStore,
        StagedGeneration,
    ) {
        let directory = tempfile::tempdir().unwrap();
        seed_current(directory.path(), "old");
        if stale_temp {
            std::fs::write(directory.path().join(CURRENT_TEMP_FILE_NAME), b"stale").unwrap();
        }
        let injector = Arc::new(RecordingInjector::default());
        let store =
            GenerationStore::open_with_injector(directory.path(), injector.clone()).unwrap();
        let staged = stage_fixture(&store, "new");
        (directory, injector, store, staged)
    }

    fn current_generation(store: &GenerationStore) -> String {
        match store.read_current().unwrap() {
            CurrentTarget::Generation(generation) => generation.as_str().to_owned(),
            CurrentTarget::Empty => panic!("expected named generation"),
        }
    }

    fn successful_points(stale_temp: bool) -> Vec<FailurePoint> {
        let (_directory, injector, store, staged) = instrumented_fixture(stale_temp);
        store.commit(staged).unwrap();
        injector.points()
    }

    #[test]
    fn generation_name_accepts_safe_legacy_and_unique_names() {
        for candidate in ["gen-42", "restore_2026.08.27-1", "A0", "0"] {
            assert_eq!(name(candidate).as_str(), candidate);
        }
    }

    #[test]
    fn generation_name_rejects_reserved_traversal_and_non_ascii() {
        let cases = [
            ("", GenerationNameErrorKind::Empty),
            ("CURRENT", GenerationNameErrorKind::Reserved),
            ("CURRENT.tmp", GenerationNameErrorKind::Reserved),
            (".", GenerationNameErrorKind::InvalidCharacter),
            ("..", GenerationNameErrorKind::InvalidCharacter),
            ("../gen", GenerationNameErrorKind::InvalidCharacter),
            ("a/b", GenerationNameErrorKind::InvalidCharacter),
            ("a\\b", GenerationNameErrorKind::InvalidCharacter),
            ("é", GenerationNameErrorKind::InvalidCharacter),
        ];
        for (candidate, expected) in cases {
            assert_eq!(GenerationName::parse(candidate).unwrap_err().kind, expected);
        }
        assert_eq!(
            GenerationName::parse("a".repeat(129)).unwrap_err().kind,
            GenerationNameErrorKind::TooLong
        );
    }

    #[test]
    fn begin_creates_one_unique_direct_child_stage() {
        let directory = tempfile::tempdir().unwrap();
        let store = GenerationStore::open(directory.path()).unwrap();
        let staged = store.begin(name("gen-1")).unwrap();
        assert_eq!(
            staged.path(),
            std::fs::canonicalize(directory.path())
                .unwrap()
                .join(".stage-gen-1")
        );
        assert!(staged.path().is_dir());
        assert_eq!(
            store.begin(name("gen-1")).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        std::fs::create_dir(directory.path().join("gen-2")).unwrap();
        assert_eq!(
            store.begin(name("gen-2")).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
    }

    #[test]
    fn independent_opens_share_one_process_root_lock() {
        let directory = tempfile::tempdir().unwrap();
        let first = GenerationStore::open(directory.path()).unwrap();
        let second = GenerationStore::open(directory.path()).unwrap();
        assert!(Arc::ptr_eq(&first.inner.commit, &second.inner.commit));
    }

    #[test]
    fn commit_syncs_all_files_then_directories_leaf_to_root() {
        let (_directory, injector, store, staged) = instrumented_fixture(false);
        store.commit(staged).unwrap();
        let points = injector.points();
        let sync_points: Vec<_> = points
            .iter()
            .filter(|point| matches!(point.step, CommitStep::SyncFile | CommitStep::SyncDirectory))
            .map(|point| (point.step, point.occurrence, point.relative_path.clone()))
            .collect();
        assert_eq!(
            sync_points,
            vec![
                (CommitStep::SyncFile, 0, PathBuf::from("a.txt")),
                (CommitStep::SyncFile, 1, PathBuf::from("b/c/n.txt")),
                (CommitStep::SyncFile, 2, PathBuf::from("b/m.txt")),
                (CommitStep::SyncFile, 3, PathBuf::from("z.txt")),
                (CommitStep::SyncDirectory, 0, PathBuf::from("b/c")),
                (CommitStep::SyncDirectory, 1, PathBuf::from("a-dir")),
                (CommitStep::SyncDirectory, 2, PathBuf::from("b")),
                (CommitStep::SyncDirectory, 3, PathBuf::from(".")),
            ]
        );
    }

    #[test]
    fn commit_renames_generation_before_writing_current() {
        let points = successful_points(false);
        let rename_generation = points
            .iter()
            .position(|point| point.step == CommitStep::RenameGeneration)
            .unwrap();
        let write_current = points
            .iter()
            .position(|point| point.step == CommitStep::WriteCurrentTemp)
            .unwrap();
        assert!(rename_generation < write_current);
    }

    #[test]
    fn current_rename_is_the_only_commit_point() {
        let rename = successful_points(false)
            .into_iter()
            .find(|point| point.step == CommitStep::RenameCurrent)
            .unwrap();
        let (_directory, injector, store, staged) = instrumented_fixture(false);
        injector.fail_at(rename, io::ErrorKind::Other);
        let error = store.commit(staged).unwrap_err();
        assert_eq!(error.class(), CommitFailureClass::PreCommit);
        assert_eq!(current_generation(&store), "old");

        let final_sync = successful_points(false)
            .into_iter()
            .find(|point| point.step == CommitStep::SyncRootAfterCurrent)
            .unwrap();
        let (_directory, injector, store, staged) = instrumented_fixture(false);
        injector.fail_at(final_sync, io::ErrorKind::Other);
        let error = store.commit(staged).unwrap_err();
        assert_eq!(error.class(), CommitFailureClass::CommitUncertain);
        assert_eq!(current_generation(&store), "new");
    }

    #[test]
    fn every_precommit_injection_preserves_old_current() {
        let points = successful_points(true);
        for point in points
            .into_iter()
            .filter(|point| point.step != CommitStep::SyncRootAfterCurrent)
        {
            let (_directory, injector, store, staged) = instrumented_fixture(true);
            injector.fail_at(point.clone(), io::ErrorKind::Other);
            let error = store.commit(staged).unwrap_err();
            assert_eq!(
                error.class(),
                CommitFailureClass::PreCommit,
                "unexpected class at {point:?}"
            );
            assert_eq!(
                current_generation(&store),
                "old",
                "CURRENT changed at {point:?}"
            );
        }
    }

    #[test]
    fn final_root_sync_failure_is_commit_uncertain() {
        let point = successful_points(false)
            .into_iter()
            .find(|point| point.step == CommitStep::SyncRootAfterCurrent)
            .unwrap();
        let (_directory, injector, store, staged) = instrumented_fixture(false);
        injector.fail_at(point, io::ErrorKind::Other);
        let error = store.commit(staged).unwrap_err();
        assert_eq!(error.class(), CommitFailureClass::CommitUncertain);
        assert_eq!(error.step(), CommitStep::SyncRootAfterCurrent);
        assert_eq!(current_generation(&store), "new");
    }

    #[test]
    fn precommit_storage_full_preserves_io_kind_and_old_current() {
        let point = successful_points(false)
            .into_iter()
            .find(|point| point.step == CommitStep::WriteCurrentTemp)
            .unwrap();
        let (_directory, injector, store, staged) = instrumented_fixture(false);
        injector.fail_at(point, io::ErrorKind::StorageFull);
        let error = store.commit(staged).unwrap_err();
        assert_eq!(error.class(), CommitFailureClass::PreCommit);
        assert_eq!(error.io_error().kind(), io::ErrorKind::StorageFull);
        assert_eq!(current_generation(&store), "old");
    }

    #[test]
    fn commit_uncertain_storage_full_preserves_io_kind() {
        let point = successful_points(false)
            .into_iter()
            .find(|point| point.step == CommitStep::SyncRootAfterCurrent)
            .unwrap();
        let (_directory, injector, store, staged) = instrumented_fixture(false);
        injector.fail_at(point, io::ErrorKind::StorageFull);
        let error = store.commit(staged).unwrap_err();
        assert_eq!(error.class(), CommitFailureClass::CommitUncertain);
        assert_eq!(error.io_error().kind(), io::ErrorKind::StorageFull);
    }

    #[test]
    fn restart_reads_only_the_named_current_generation() {
        let directory = tempfile::tempdir().unwrap();
        seed_current(directory.path(), "old");
        std::fs::create_dir(directory.path().join("newer")).unwrap();
        let reopened = GenerationStore::open(directory.path()).unwrap();
        assert_eq!(current_generation(&reopened), "old");
    }

    #[test]
    fn restart_ignores_unpointed_and_staging_generations() {
        let directory = tempfile::tempdir().unwrap();
        seed_current(directory.path(), "selected");
        std::fs::create_dir(directory.path().join("zzzz-unpointed")).unwrap();
        std::fs::create_dir(directory.path().join(".stage-future")).unwrap();
        let reopened = GenerationStore::open(directory.path()).unwrap();
        assert_eq!(current_generation(&reopened), "selected");
    }

    #[test]
    fn read_current_returns_missing_instead_of_auto_selecting() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir(directory.path().join("gen-999")).unwrap();
        let store = GenerationStore::open(directory.path()).unwrap();
        let error = store.read_current().unwrap_err();
        assert_eq!(error.kind, CurrentReadErrorKind::Missing);
    }

    #[test]
    fn read_current_accepts_explicit_empty() {
        let directory = tempfile::tempdir().unwrap();
        let store = GenerationStore::open(directory.path()).unwrap();
        store.initialize_empty().unwrap();
        assert_eq!(store.read_current().unwrap(), CurrentTarget::Empty);
        let reopened = GenerationStore::open(directory.path()).unwrap();
        assert_eq!(reopened.read_current().unwrap(), CurrentTarget::Empty);
    }

    #[test]
    fn read_current_rejects_malformed_pointer() {
        for bytes in [
            b"".as_slice(),
            b"empty".as_slice(),
            b"empty\nextra".as_slice(),
            b"generation:gen-1".as_slice(),
            b"generation:gen-1\nextra".as_slice(),
            b"other:gen-1\n".as_slice(),
        ] {
            let directory = tempfile::tempdir().unwrap();
            std::fs::write(directory.path().join(CURRENT_FILE_NAME), bytes).unwrap();
            let store = GenerationStore::open(directory.path()).unwrap();
            assert_eq!(
                store.read_current().unwrap_err().kind,
                CurrentReadErrorKind::Malformed
            );
        }
    }

    #[test]
    fn read_current_rejects_traversal_target() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join(CURRENT_FILE_NAME),
            b"generation:../outside\n",
        )
        .unwrap();
        let store = GenerationStore::open(directory.path()).unwrap();
        assert_eq!(
            store.read_current().unwrap_err().kind,
            CurrentReadErrorKind::UnsafeTarget
        );
    }

    #[test]
    fn read_current_rejects_missing_target() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join(CURRENT_FILE_NAME),
            b"generation:missing\n",
        )
        .unwrap();
        let store = GenerationStore::open(directory.path()).unwrap();
        assert_eq!(
            store.read_current().unwrap_err().kind,
            CurrentReadErrorKind::TargetMissing
        );
    }

    #[cfg(unix)]
    #[test]
    fn read_current_rejects_symlink_and_non_directory_target() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("file-target"), b"x").unwrap();
        std::fs::write(
            directory.path().join(CURRENT_FILE_NAME),
            b"generation:file-target\n",
        )
        .unwrap();
        let store = GenerationStore::open(directory.path()).unwrap();
        assert_eq!(
            store.read_current().unwrap_err().kind,
            CurrentReadErrorKind::TargetNotDirectory
        );

        std::fs::create_dir(directory.path().join("real-target")).unwrap();
        symlink(
            directory.path().join("real-target"),
            directory.path().join("link-target"),
        )
        .unwrap();
        std::fs::write(
            directory.path().join(CURRENT_FILE_NAME),
            b"generation:link-target\n",
        )
        .unwrap();
        assert_eq!(
            store.read_current().unwrap_err().kind,
            CurrentReadErrorKind::TargetSymlink
        );
    }

    #[cfg(unix)]
    #[test]
    fn commit_rejects_symlink_and_special_staging_entries() {
        use std::os::unix::fs::symlink;
        use std::os::unix::net::UnixListener;

        let directory = tempfile::tempdir().unwrap();
        seed_current(directory.path(), "old");
        let store = GenerationStore::open(directory.path()).unwrap();
        let staged = store.begin(name("symlinked")).unwrap();
        symlink(
            directory.path().join("old/payload"),
            staged.path().join("payload-link"),
        )
        .unwrap();
        let error = store.commit(staged).unwrap_err();
        assert_eq!(error.class(), CommitFailureClass::PreCommit);
        assert_eq!(current_generation(&store), "old");

        let staged = store.begin(name("socketed")).unwrap();
        let _listener = UnixListener::bind(staged.path().join("socket")).unwrap();
        let error = store.commit(staged).unwrap_err();
        assert_eq!(error.class(), CommitFailureClass::PreCommit);
        assert_eq!(current_generation(&store), "old");
    }

    #[test]
    fn commit_refuses_existing_immutable_generation() {
        let directory = tempfile::tempdir().unwrap();
        seed_current(directory.path(), "old");
        let store = GenerationStore::open(directory.path()).unwrap();
        let staged = store.begin(name("new")).unwrap();
        std::fs::create_dir(directory.path().join("new")).unwrap();
        let error = store.commit(staged).unwrap_err();
        assert_eq!(error.class(), CommitFailureClass::PreCommit);
        assert_eq!(error.step(), CommitStep::ValidateStaging);
        assert_eq!(current_generation(&store), "old");
    }

    #[test]
    fn stale_regular_current_tmp_is_removed_safely() {
        let (directory, _injector, store, staged) = instrumented_fixture(true);
        store.commit(staged).unwrap();
        assert_eq!(current_generation(&store), "new");
        assert!(!directory.path().join(CURRENT_TEMP_FILE_NAME).exists());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_current_tmp_is_rejected_without_touching_current() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        seed_current(directory.path(), "old");
        symlink(
            directory.path().join("old/payload"),
            directory.path().join(CURRENT_TEMP_FILE_NAME),
        )
        .unwrap();
        let store = GenerationStore::open(directory.path()).unwrap();
        let staged = stage_fixture(&store, "new");
        let error = store.commit(staged).unwrap_err();
        assert_eq!(error.step(), CommitStep::RemoveStaleCurrentTemp);
        assert_eq!(error.class(), CommitFailureClass::PreCommit);
        assert_eq!(current_generation(&store), "old");
        assert!(directory
            .path()
            .join(CURRENT_TEMP_FILE_NAME)
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn initialize_empty_refuses_existing_legacy_generation() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir(directory.path().join("gen-1")).unwrap();
        let store = GenerationStore::open(directory.path()).unwrap();
        let error = store.initialize_empty().unwrap_err();
        assert_eq!(error.class(), CommitFailureClass::PreCommit);
        assert_eq!(error.io_error().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            store.read_current().unwrap_err().kind,
            CurrentReadErrorKind::Missing
        );
    }

    #[test]
    fn legacy_adoption_requires_exact_caller_selected_name() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir(directory.path().join("gen-2")).unwrap();
        std::fs::write(directory.path().join("gen-2/data"), b"two").unwrap();
        std::fs::create_dir(directory.path().join("gen-99")).unwrap();
        std::fs::write(directory.path().join("gen-99/data"), b"ninety-nine").unwrap();
        let store = GenerationStore::open(directory.path()).unwrap();
        store.adopt_legacy(name("gen-2")).unwrap();
        assert_eq!(current_generation(&store), "gen-2");
    }

    #[test]
    fn legacy_adoption_never_selects_highest_generation() {
        let directory = tempfile::tempdir().unwrap();
        for generation in ["gen-1", "gen-1000"] {
            std::fs::create_dir(directory.path().join(generation)).unwrap();
            std::fs::write(directory.path().join(generation).join("data"), generation).unwrap();
        }
        let store = GenerationStore::open(directory.path()).unwrap();
        assert_eq!(
            store.read_current().unwrap_err().kind,
            CurrentReadErrorKind::Missing
        );
        store.adopt_legacy(name("gen-1")).unwrap();
        assert_eq!(current_generation(&store), "gen-1");
    }

    #[test]
    fn legacy_adoption_uses_the_same_pointer_commit_semantics() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir(directory.path().join("gen-1")).unwrap();
        std::fs::write(directory.path().join("gen-1/data"), b"one").unwrap();
        let injector = Arc::new(RecordingInjector::default());
        let store =
            GenerationStore::open_with_injector(directory.path(), injector.clone()).unwrap();
        injector.fail_at(
            FailurePoint {
                step: CommitStep::RenameCurrent,
                occurrence: 0,
                relative_path: PathBuf::from(CURRENT_FILE_NAME),
            },
            io::ErrorKind::Other,
        );
        let error = store.adopt_legacy(name("gen-1")).unwrap_err();
        assert_eq!(error.class(), CommitFailureClass::PreCommit);
        assert_eq!(
            store.read_current().unwrap_err().kind,
            CurrentReadErrorKind::Missing
        );

        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir(directory.path().join("gen-1")).unwrap();
        std::fs::write(directory.path().join("gen-1/data"), b"one").unwrap();
        let injector = Arc::new(RecordingInjector::default());
        let store =
            GenerationStore::open_with_injector(directory.path(), injector.clone()).unwrap();
        injector.fail_at(
            FailurePoint {
                step: CommitStep::SyncRootAfterCurrent,
                occurrence: 0,
                relative_path: PathBuf::from("."),
            },
            io::ErrorKind::Other,
        );
        let error = store.adopt_legacy(name("gen-1")).unwrap_err();
        assert_eq!(error.class(), CommitFailureClass::CommitUncertain);
        assert_eq!(current_generation(&store), "gen-1");
    }

    #[test]
    fn failure_points_have_stable_step_occurrence_and_path_order() {
        let first = successful_points(true);
        let second = successful_points(true);
        assert_eq!(first, second);
        for step in [CommitStep::SyncFile, CommitStep::SyncDirectory] {
            let occurrences: Vec<_> = first
                .iter()
                .filter(|point| point.step == step)
                .map(|point| point.occurrence)
                .collect();
            assert_eq!(occurrences, (0..occurrences.len()).collect::<Vec<_>>());
        }
    }
}
// CODEGEN-END
