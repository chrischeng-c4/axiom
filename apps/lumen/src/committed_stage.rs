//! Durable, sequence-ordered staging for committed records.
//!
//! A [`DurableStage`] is proof that the exact record bytes were copied out of
//! RAM, synced, named, and described by a separately synced marker.  The
//! handle has private fields so a caller cannot forge that proof or release a
//! change-budget reservation before the payload transition completed.
//!
//! This module deliberately does not decode records.  Its caller streams an
//! already durable wire representation through [`StageStore::stage`], keeping
//! the staging copy bounded by [`COPY_BUFFER_BYTES`].

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::Arc;

use sha2::{Digest, Sha256};

const FORMAT_MAGIC: &[u8; 8] = b"LCSTAGE\0";
const FORMAT_VERSION: u8 = 1;
const MARKER_SUFFIX: &str = ".commit";
const RECORD_SUFFIX: &str = ".record";
const COPY_BUFFER_BYTES: usize = 64 * 1024;
const MAX_SOURCE_ID_BYTES: usize = 4096;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

/// Identity of the source that committed a record.  This is stored in the
/// marker and checked on recovery; it is never used as a filesystem path.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct SourceIdentity {
    kind: SourceKind,
    origin: String,
}

impl SourceIdentity {
    pub(crate) fn new(kind: SourceKind, origin: impl Into<String>) -> io::Result<Self> {
        let origin = origin.into();
        if origin.is_empty() || origin.len() > MAX_SOURCE_ID_BYTES {
            return Err(invalid("source identity has invalid byte length"));
        }
        Ok(Self { kind, origin })
    }
}

/// Kept deliberately small and versioned.  An unrecognized tag is a recovery
/// error, never a silently reclassified external record.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub(crate) enum SourceKind {
    Local = 1,
    External = 2,
    Replay = 3,
    Raft = 4,
}

impl SourceKind {
    fn decode(tag: u8) -> io::Result<Self> {
        match tag {
            1 => Ok(Self::Local),
            2 => Ok(Self::External),
            3 => Ok(Self::Replay),
            4 => Ok(Self::Raft),
            _ => Err(invalid("committed-stage marker has unknown source kind")),
        }
    }
}

/// Failure points are only for tests and callers that deliberately inject
/// local durable-storage faults.  They run before the corresponding syscall,
/// so an error never yields a durable receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StageFailurePoint {
    SyncPayload,
    RenamePayload,
    SyncPayloadDirectory,
    SyncMarker,
    PublishMarker,
    SyncMarkerDirectory,
}

pub(crate) trait StageFailureInjector: Send + Sync {
    fn check(&self, point: StageFailurePoint) -> io::Result<()>;
}

#[derive(Default)]
struct NoFailures;

impl StageFailureInjector for NoFailures {
    fn check(&self, _: StageFailurePoint) -> io::Result<()> {
        Ok(())
    }
}

/// A private durable staging root.  It has separate `records` and `markers`
/// directories so a marker is the sole recovery-visible commit point.
pub(crate) struct StageStore {
    root: PathBuf,
    injector: Arc<dyn StageFailureInjector>,
}

/// A durable, validated stage receipt.  Construction and fields are private to
/// this module.  Its removal requires a caller-provided verified watermark.
#[derive(Debug)]
pub(crate) struct DurableStage {
    store_root: PathBuf,
    sequence: u64,
    engine_epoch: u64,
    source: SourceIdentity,
    byte_len: u64,
    digest: [u8; 32],
    record_path: PathBuf,
    marker_path: PathBuf,
}

/// Caller-owned proof that a particular Engine epoch and source identity are
/// covered by a durable watermark.  It cannot be constructed outside this
/// module or reused for a different staging root.
pub(crate) struct CleanupProof {
    store_root: PathBuf,
    source: SourceIdentity,
    engine_epoch: u64,
    verified_watermark: u64,
}

impl DurableStage {
    pub(crate) fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(crate) fn engine_epoch(&self) -> u64 {
        self.engine_epoch
    }

    pub(crate) fn source(&self) -> &SourceIdentity {
        &self.source
    }

    pub(crate) fn byte_len(&self) -> u64 {
        self.byte_len
    }

    /// Reopen through the validated payload path.  This does not deserialize
    /// the record and does not allocate a full-record `Vec`.
    pub(crate) fn open(&self) -> io::Result<File> {
        let mut file = open_regular_readonly(&self.record_path)?;
        let (len, digest) = digest_reader(&mut file)?;
        if len != self.byte_len || digest != self.digest {
            return Err(invalid("committed-stage payload differs from marker"));
        }
        file.seek(SeekFrom::Start(0))?;
        Ok(file)
    }
}

impl StageStore {
    pub(crate) fn new(root: impl Into<PathBuf>) -> io::Result<Self> {
        Self::with_injector(root, Arc::new(NoFailures))
    }

    pub(crate) fn with_injector(
        root: impl Into<PathBuf>,
        injector: Arc<dyn StageFailureInjector>,
    ) -> io::Result<Self> {
        let root = root.into();
        prepare_root(&root)?;
        let root = fs::canonicalize(root)?;
        Ok(Self { root, injector })
    }

    /// Stream a committed record into a new durable stage.
    ///
    /// The source reader is consumed once.  The implementation owns only a
    /// fixed 64 KiB buffer, the SHA-256 state, and small marker metadata.
    pub(crate) fn stage<R: Read>(
        &self,
        sequence: u64,
        engine_epoch: u64,
        source: SourceIdentity,
        mut reader: R,
    ) -> io::Result<DurableStage> {
        self.stage_with_writer(sequence, engine_epoch, source, |writer| {
            copy_reader_fixed(&mut reader, writer)
        })
    }

    /// Stream a serializer directly into staging. This avoids a whole-record
    /// encoded buffer for owned records such as coordinator `Record`.
    pub(crate) fn stage_with_writer<F>(
        &self,
        sequence: u64,
        engine_epoch: u64,
        source: SourceIdentity,
        mut write: F,
    ) -> io::Result<DurableStage>
    where
        F: FnMut(&mut dyn Write) -> io::Result<()>,
    {
        let name = stage_name(sequence, engine_epoch);
        let records = self.records_dir();
        let markers = self.markers_dir();
        ensure_directory(&records)?;
        ensure_directory(&markers)?;

        let record_path = records.join(format!("{name}{RECORD_SUFFIX}"));
        let marker_path = markers.join(format!("{name}{MARKER_SUFFIX}"));
        let payload_temp = records.join(temp_name("payload"));
        let marker_temp = markers.join(temp_name("marker"));
        let result = (|| {
            let (byte_len, digest) = write_and_hash(&payload_temp, &mut write)?;
            let expected = Marker {
                sequence,
                engine_epoch,
                source: source.clone(),
                byte_len,
                digest,
            };
            if path_exists(&marker_path)? {
                let marker = read_marker(&marker_path)?;
                if marker != expected {
                    return Err(invalid("conflicting committed-stage identity or payload"));
                }
                self.finish_existing(&record_path, &marker_path, &expected)?;
                self.remove_redundant_payload_temp(&payload_temp)?;
                return Ok(self.receipt(expected, record_path, marker_path));
            }
            if path_exists(&record_path)? {
                let (old_len, old_digest) = digest_reader(open_regular_readonly(&record_path)?)?;
                if old_len != byte_len || old_digest != digest {
                    return Err(invalid("conflicting committed-stage payload"));
                }
                self.injector.check(StageFailurePoint::SyncPayload)?;
                sync_regular_file(&record_path)?;
                self.injector
                    .check(StageFailurePoint::SyncPayloadDirectory)?;
                sync_directory(&records)?;
                self.finish_marker(&marker_temp, &marker_path, &expected)?;
                self.remove_redundant_payload_temp(&payload_temp)?;
                return Ok(self.receipt(expected, record_path, marker_path));
            }
            self.injector.check(StageFailurePoint::SyncPayload)?;
            sync_regular_file(&payload_temp)?;
            self.injector.check(StageFailurePoint::RenamePayload)?;
            rename_new(&payload_temp, &record_path)?;
            self.injector
                .check(StageFailurePoint::SyncPayloadDirectory)?;
            sync_directory(&records)?;

            self.finish_marker(&marker_temp, &marker_path, &expected)?;
            Ok(self.receipt(expected, record_path, marker_path))
        })();
        if result.is_err() {
            // Do not delete a payload that may have reached a commit-adjacent
            // state.  It is an orphan until an operator/recovery policy proves
            // it safe.  Temp files get the same conservative treatment.
        }
        result
    }

    fn finish_existing(&self, record: &Path, marker: &Path, expected: &Marker) -> io::Result<()> {
        let (len, digest) = digest_reader(open_regular_readonly(record)?)?;
        if len != expected.byte_len || digest != expected.digest {
            return Err(invalid("committed-stage payload differs from marker"));
        }
        self.injector.check(StageFailurePoint::SyncPayload)?;
        sync_regular_file(record)?;
        self.injector
            .check(StageFailurePoint::SyncPayloadDirectory)?;
        sync_directory(&self.records_dir())?;
        self.injector.check(StageFailurePoint::SyncMarker)?;
        sync_regular_file(marker)?;
        self.injector
            .check(StageFailurePoint::SyncMarkerDirectory)?;
        sync_directory(&self.markers_dir())
    }

    /// An existing matching marker proves this invocation's new temporary
    /// payload is not recovery data. Remove only that known file; uncertain
    /// failed-stage payloads and all other orphans remain untouched.
    fn remove_redundant_payload_temp(&self, payload_temp: &Path) -> io::Result<()> {
        ensure_regular_file(payload_temp)?;
        fs::remove_file(payload_temp)?;
        sync_directory(&self.records_dir())
    }

    fn finish_marker(&self, temp: &Path, marker: &Path, expected: &Marker) -> io::Result<()> {
        write_marker(temp, expected)?;
        self.injector.check(StageFailurePoint::SyncMarker)?;
        sync_regular_file(temp)?;
        self.injector.check(StageFailurePoint::PublishMarker)?;
        rename_new(temp, marker)?;
        self.injector
            .check(StageFailurePoint::SyncMarkerDirectory)?;
        sync_directory(&self.markers_dir())
    }

    fn receipt(&self, marker: Marker, record_path: PathBuf, marker_path: PathBuf) -> DurableStage {
        DurableStage {
            store_root: self.root.clone(),
            sequence: marker.sequence,
            engine_epoch: marker.engine_epoch,
            source: marker.source,
            byte_len: marker.byte_len,
            digest: marker.digest,
            record_path,
            marker_path,
        }
    }

    pub(crate) fn cleanup_proof(
        &self,
        source: SourceIdentity,
        engine_epoch: u64,
        verified_watermark: u64,
    ) -> CleanupProof {
        CleanupProof {
            store_root: self.root.clone(),
            source,
            engine_epoch,
            verified_watermark,
        }
    }

    /// Recover only fully published marker/payload pairs.  Orphan data files
    /// and temporary names are intentionally retained and never cleaned here.
    pub(crate) fn recover(&self) -> io::Result<Vec<DurableStage>> {
        let records = self.records_dir();
        let markers = self.markers_dir();
        ensure_directory(&records)?;
        ensure_directory(&markers)?;
        let mut recovered = Vec::new();
        let mut seen = BTreeSet::new();
        for entry in fs::read_dir(&markers)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                return Err(invalid("committed-stage marker has non-UTF8 name"));
            };
            if name.starts_with(".marker-") && name.ends_with(".tmp") {
                // A failed marker publication leaves an uncertain orphan.  It
                // is deliberately retained, but it is not a commit marker.
                continue;
            }
            if !name.ends_with(MARKER_SUFFIX) {
                return Err(invalid("committed-stage marker directory has unknown file"));
            }
            ensure_regular_file(&path)?;
            let marker = read_marker(&path)?;
            let expected = format!(
                "{}{}",
                stage_name(marker.sequence, marker.engine_epoch),
                MARKER_SUFFIX
            );
            if name != expected {
                return Err(invalid(
                    "committed-stage marker name does not match content",
                ));
            }
            if !seen.insert((marker.sequence, marker.engine_epoch, marker.source.clone())) {
                return Err(invalid("duplicate committed-stage marker identity"));
            }
            let record_path = records.join(format!(
                "{}{}",
                stage_name(marker.sequence, marker.engine_epoch),
                RECORD_SUFFIX
            ));
            ensure_regular_file(&record_path)?;
            let (actual_len, actual_digest) = digest_reader(open_regular_readonly(&record_path)?)?;
            if actual_len != marker.byte_len || actual_digest != marker.digest {
                return Err(invalid(
                    "committed-stage payload checksum or length mismatch",
                ));
            }
            recovered.push(DurableStage {
                store_root: self.root.clone(),
                sequence: marker.sequence,
                engine_epoch: marker.engine_epoch,
                source: marker.source,
                byte_len: marker.byte_len,
                digest: marker.digest,
                record_path,
                marker_path: path,
            });
        }
        recovered.sort_by(|left, right| {
            (left.sequence, left.engine_epoch, &left.source).cmp(&(
                right.sequence,
                right.engine_epoch,
                &right.source,
            ))
        });
        Ok(recovered)
    }

    /// Delete a receipt only after the caller has verified a durable Engine
    /// watermark that includes this exact sequence.  There is intentionally no
    /// `Drop` cleanup path.
    pub(crate) fn remove_after_verified_watermark(
        &self,
        stage: DurableStage,
        proof: CleanupProof,
    ) -> io::Result<()> {
        if stage.store_root != self.root || proof.store_root != self.root {
            return Err(invalid("committed-stage receipt belongs to another store"));
        }
        if proof.source != stage.source || proof.engine_epoch != stage.engine_epoch {
            return Err(invalid(
                "cleanup proof does not cover stage source and epoch",
            ));
        }
        if proof.verified_watermark < stage.sequence {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "durable watermark is below committed-stage sequence",
            ));
        }
        // Revalidate before removal so a caller cannot remove a path that a
        // corrupt or substituted marker no longer proves.
        let marker = read_marker(&stage.marker_path)?;
        if marker.sequence != stage.sequence
            || marker.engine_epoch != stage.engine_epoch
            || marker.source != stage.source
            || marker.byte_len != stage.byte_len
            || marker.digest != stage.digest
        {
            return Err(invalid("committed-stage receipt no longer matches marker"));
        }
        let (len, digest) = digest_reader(open_regular_readonly(&stage.record_path)?)?;
        if len != stage.byte_len || digest != stage.digest {
            return Err(invalid(
                "committed-stage receipt payload changed before removal",
            ));
        }
        fs::remove_file(&stage.marker_path)?;
        sync_directory(&self.markers_dir())?;
        fs::remove_file(&stage.record_path)?;
        sync_directory(&self.records_dir())?;
        Ok(())
    }

    fn records_dir(&self) -> PathBuf {
        self.root.join("records")
    }

    fn markers_dir(&self) -> PathBuf {
        self.root.join("markers")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Marker {
    sequence: u64,
    engine_epoch: u64,
    source: SourceIdentity,
    byte_len: u64,
    digest: [u8; 32],
}

fn write_marker(path: &Path, marker: &Marker) -> io::Result<()> {
    let origin = marker.source.origin.as_bytes();
    let origin_len =
        u32::try_from(origin.len()).map_err(|_| invalid("source identity too long"))?;
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(FORMAT_MAGIC)?;
    file.write_all(&[FORMAT_VERSION, marker.source.kind as u8])?;
    file.write_all(&marker.sequence.to_le_bytes())?;
    file.write_all(&marker.engine_epoch.to_le_bytes())?;
    file.write_all(&marker.byte_len.to_le_bytes())?;
    file.write_all(&origin_len.to_le_bytes())?;
    file.write_all(origin)?;
    file.write_all(&marker.digest)?;
    Ok(())
}

fn read_marker(path: &Path) -> io::Result<Marker> {
    let mut file = open_regular_readonly(path)?;
    let mut magic = [0; 8];
    file.read_exact(&mut magic)?;
    if &magic != FORMAT_MAGIC {
        return Err(invalid("committed-stage marker has unknown format"));
    }
    let version = read_u8(&mut file)?;
    if version != FORMAT_VERSION {
        return Err(invalid("committed-stage marker has unknown version"));
    }
    let kind = SourceKind::decode(read_u8(&mut file)?)?;
    let sequence = read_u64(&mut file)?;
    let engine_epoch = read_u64(&mut file)?;
    let byte_len = read_u64(&mut file)?;
    let origin_len =
        usize::try_from(read_u32(&mut file)?).map_err(|_| invalid("invalid source length"))?;
    if origin_len == 0 || origin_len > MAX_SOURCE_ID_BYTES {
        return Err(invalid("committed-stage marker has invalid source length"));
    }
    let mut origin = vec![0; origin_len];
    file.read_exact(&mut origin)?;
    let origin = String::from_utf8(origin)
        .map_err(|_| invalid("committed-stage marker source is not UTF-8"))?;
    let mut digest = [0; 32];
    file.read_exact(&mut digest)?;
    if file.read(&mut [0; 1])? != 0 {
        return Err(invalid("committed-stage marker has trailing bytes"));
    }
    Ok(Marker {
        sequence,
        engine_epoch,
        source: SourceIdentity::new(kind, origin)?,
        byte_len,
        digest,
    })
}

fn copy_reader_fixed<R: Read>(reader: &mut R, output: &mut dyn Write) -> io::Result<()> {
    let mut buffer = [0u8; COPY_BUFFER_BYTES];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        output.write_all(&buffer[..read])?;
    }
    Ok(())
}

fn write_and_hash<F>(target: &Path, write: &mut F) -> io::Result<(u64, [u8; 32])>
where
    F: FnMut(&mut dyn Write) -> io::Result<()>,
{
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)?;
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    {
        let mut writer = HashingWriter {
            output: &mut output,
            hasher: &mut hasher,
            bytes: &mut bytes,
        };
        write(&mut writer)?;
    }
    Ok((bytes, hasher.finalize().into()))
}

struct HashingWriter<'a> {
    output: &'a mut File,
    hasher: &'a mut Sha256,
    bytes: &'a mut u64,
}

impl Write for HashingWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.output.write_all(bytes)?;
        self.hasher.update(bytes);
        *self.bytes = self
            .bytes
            .checked_add(u64::try_from(bytes.len()).expect("slice length fits u64"))
            .ok_or_else(|| invalid("committed-stage payload length overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

fn digest_reader<R: Read>(mut reader: R) -> io::Result<(u64, [u8; 32])> {
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; COPY_BUFFER_BYTES];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes = bytes
            .checked_add(u64::try_from(read).expect("buffer length fits u64"))
            .ok_or_else(|| invalid("committed-stage payload length overflow"))?;
    }
    Ok((bytes, hasher.finalize().into()))
}

fn prepare_root(root: &Path) -> io::Result<()> {
    if !root.exists() {
        fs::create_dir_all(root)?;
    }
    ensure_directory(root)?;
    let records = root.join("records");
    let markers = root.join("markers");
    if !records.exists() {
        fs::create_dir(&records)?;
    }
    if !markers.exists() {
        fs::create_dir(&markers)?;
    }
    ensure_directory(&records)?;
    ensure_directory(&markers)?;
    Ok(())
}

fn ensure_directory(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(invalid("committed-stage path is not a real directory"));
    }
    Ok(())
}

fn ensure_regular_file(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(invalid("committed-stage path is not a real regular file"));
    }
    Ok(())
}

fn sync_regular_file(path: &Path) -> io::Result<()> {
    open_regular_readonly(path)?.sync_all()
}

fn sync_directory(path: &Path) -> io::Result<()> {
    open_directory_readonly(path)?.sync_all()
}

/// The stage root is private to this process, but a checked metadata result is
/// still not sufficient: a hostile or accidental concurrent rename can replace
/// the final component before `File::open`.  On supported Unix targets,
/// `O_NOFOLLOW` makes that race fail instead of dereferencing the replacement.
/// The post-open metadata check also rejects non-regular descriptors.
fn open_regular_readonly(path: &Path) -> io::Result<File> {
    ensure_regular_file(path)?;
    let file = open_readonly_nofollow(path)?;
    if !file.metadata()?.is_file() {
        return Err(invalid("committed-stage path opened as non-regular file"));
    }
    Ok(file)
}

fn open_directory_readonly(path: &Path) -> io::Result<File> {
    ensure_directory(path)?;
    let file = open_readonly_nofollow(path)?;
    if !file.metadata()?.is_dir() {
        return Err(invalid("committed-stage path opened as non-directory"));
    }
    Ok(file)
}

fn open_readonly_nofollow(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        use std::os::unix::fs::OpenOptionsExt;
        const O_NOFOLLOW: i32 = 0x0100;
        options.custom_flags(O_NOFOLLOW);
    }
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        use std::os::unix::fs::OpenOptionsExt;
        const O_NOFOLLOW: i32 = 0o400000;
        options.custom_flags(O_NOFOLLOW);
    }
    let file = options.open(path)?;
    Ok(file)
}

fn rename_new(from: &Path, to: &Path) -> io::Result<()> {
    if path_exists(to)? {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            "committed-stage target exists",
        ));
    }
    ensure_regular_file(from)?;
    fs::rename(from, to)
}

fn path_exists(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn stage_name(sequence: u64, engine_epoch: u64) -> String {
    format!("{sequence:020}-{engine_epoch:016x}")
}

fn temp_name(prefix: &str) -> String {
    let id = NEXT_TEMP.fetch_add(1, AtomicOrdering::Relaxed);
    format!(".{prefix}-{}-{id:016x}.tmp", std::process::id())
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}

fn read_u8(file: &mut File) -> io::Result<u8> {
    let mut value = [0; 1];
    file.read_exact(&mut value)?;
    Ok(value[0])
}

fn read_u32(file: &mut File) -> io::Result<u32> {
    let mut value = [0; 4];
    file.read_exact(&mut value)?;
    Ok(u32::from_le_bytes(value))
}

fn read_u64(file: &mut File) -> io::Result<u64> {
    let mut value = [0; 8];
    file.read_exact(&mut value)?;
    Ok(u64::from_le_bytes(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::Mutex;

    struct FailAt(Mutex<Option<StageFailurePoint>>);

    impl StageFailureInjector for FailAt {
        fn check(&self, point: StageFailurePoint) -> io::Result<()> {
            let mut wanted = self.0.lock().unwrap();
            if *wanted == Some(point) {
                *wanted = None;
                return Err(io::Error::other("injected stage fault"));
            }
            Ok(())
        }
    }

    fn source() -> SourceIdentity {
        SourceIdentity::new(SourceKind::External, "nats:orders:1").unwrap()
    }

    fn store(root: &Path, point: StageFailurePoint) -> StageStore {
        StageStore::with_injector(root, Arc::new(FailAt(Mutex::new(Some(point))))).unwrap()
    }

    struct BoundedReader {
        remaining: usize,
        largest_buffer: usize,
    }

    impl Read for BoundedReader {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            self.largest_buffer = self.largest_buffer.max(output.len());
            let count = self.remaining.min(output.len());
            output[..count].fill(b'x');
            self.remaining -= count;
            Ok(count)
        }
    }

    #[test]
    fn receipt_requires_every_payload_and_marker_durability_step() {
        for point in [
            StageFailurePoint::SyncPayload,
            StageFailurePoint::RenamePayload,
            StageFailurePoint::SyncPayloadDirectory,
            StageFailurePoint::SyncMarker,
            StageFailurePoint::PublishMarker,
            StageFailurePoint::SyncMarkerDirectory,
        ] {
            let root = tempfile::tempdir().unwrap();
            let error = store(root.path(), point)
                .stage(7, 3, source(), Cursor::new(b"payload"))
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Other, "{point:?}");
            if point != StageFailurePoint::SyncMarkerDirectory {
                assert!(
                    store(root.path(), StageFailurePoint::SyncPayload)
                        .recover()
                        .unwrap()
                        .is_empty(),
                    "{point:?} must not publish a recovery-visible marker"
                );
            }
        }
    }

    #[test]
    fn staging_reads_only_the_fixed_copy_buffer() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        let mut reader = BoundedReader {
            remaining: COPY_BUFFER_BYTES * 3 + 7,
            largest_buffer: 0,
        };
        let stage = store.stage(5, 1, source(), &mut reader).unwrap();
        assert_eq!(stage.byte_len(), (COPY_BUFFER_BYTES * 3 + 7) as u64);
        assert_eq!(reader.largest_buffer, COPY_BUFFER_BYTES);
    }

    #[test]
    fn recovery_is_sorted_and_validates_exact_payload() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        store.stage(9, 2, source(), Cursor::new(b"nine")).unwrap();
        store.stage(4, 2, source(), Cursor::new(b"four")).unwrap();
        let stages = store.recover().unwrap();
        assert_eq!(
            stages
                .iter()
                .map(DurableStage::sequence)
                .collect::<Vec<_>>(),
            [4, 9]
        );
        let mut bytes = Vec::new();
        stages[0].open().unwrap().read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"four");

        fs::write(&stages[0].record_path, b"changed").unwrap();
        assert!(store.recover().is_err());
    }

    #[test]
    fn orphan_payload_is_not_auto_removed_and_marker_corruption_fails_closed() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        let orphan = store.records_dir().join(".payload-orphan.tmp");
        fs::write(&orphan, b"uncommitted").unwrap();
        assert!(store.recover().unwrap().is_empty());
        assert!(orphan.exists());

        let stage = store.stage(8, 1, source(), Cursor::new(b"eight")).unwrap();
        fs::write(&stage.marker_path, b"unknown").unwrap();
        assert!(store.recover().is_err());
    }

    #[test]
    fn recovery_rejects_missing_and_truncated_committed_pairs() {
        let missing_root = tempfile::tempdir().unwrap();
        let missing_store = StageStore::new(missing_root.path()).unwrap();
        let missing = missing_store
            .stage(20, 1, source(), Cursor::new(b"twenty"))
            .unwrap();
        fs::remove_file(&missing.record_path).unwrap();
        assert!(missing_store.recover().is_err());

        let truncated_root = tempfile::tempdir().unwrap();
        let truncated_store = StageStore::new(truncated_root.path()).unwrap();
        let truncated = truncated_store
            .stage(21, 1, source(), Cursor::new(b"twenty-one"))
            .unwrap();
        fs::write(&truncated.marker_path, FORMAT_MAGIC).unwrap();
        assert!(truncated_store.recover().is_err());
    }

    #[test]
    fn removal_requires_verified_watermark_and_never_happens_on_drop() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        let stage = store
            .stage(12, 7, source(), Cursor::new(b"twelve"))
            .unwrap();
        let marker = stage.marker_path.clone();
        drop(stage);
        assert!(marker.exists());
        let stage = store.recover().unwrap().pop().unwrap();
        assert!(store
            .remove_after_verified_watermark(stage, store.cleanup_proof(source(), 7, 11))
            .is_err());
        let stage = store.recover().unwrap().pop().unwrap();
        store
            .remove_after_verified_watermark(stage, store.cleanup_proof(source(), 7, 12))
            .unwrap();
        assert!(store.recover().unwrap().is_empty());
    }

    #[test]
    fn recovery_rejects_symlink_marker_when_supported() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let root = tempfile::tempdir().unwrap();
            let store = StageStore::new(root.path()).unwrap();
            symlink(
                "/dev/null",
                store
                    .markers_dir()
                    .join("00000000000000000001-0000000000000001.commit"),
            )
            .unwrap();
            assert!(store.recover().is_err());
        }
    }

    #[test]
    fn open_rejects_a_payload_path_replaced_by_symlink_when_supported() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let root = tempfile::tempdir().unwrap();
            let store = StageStore::new(root.path()).unwrap();
            let stage = store
                .stage(30, 1, source(), Cursor::new(b"thirty"))
                .unwrap();
            fs::remove_file(&stage.record_path).unwrap();
            symlink("/dev/null", &stage.record_path).unwrap();
            assert!(stage.open().is_err());
        }
    }

    #[test]
    fn every_failed_durability_step_retries_the_same_payload_to_a_receipt() {
        for point in [
            StageFailurePoint::SyncPayload,
            StageFailurePoint::RenamePayload,
            StageFailurePoint::SyncPayloadDirectory,
            StageFailurePoint::SyncMarker,
            StageFailurePoint::PublishMarker,
            StageFailurePoint::SyncMarkerDirectory,
        ] {
            let root = tempfile::tempdir().unwrap();
            let store = store(root.path(), point);
            assert!(store.stage(41, 9, source(), Cursor::new(b"retry")).is_err());
            let receipt = store.stage(41, 9, source(), Cursor::new(b"retry")).unwrap();
            assert_eq!(receipt.sequence(), 41, "{point:?}");
            assert_eq!(store.recover().unwrap().len(), 1, "{point:?}");
        }
    }

    #[test]
    fn retry_rejects_conflicting_payload_and_foreign_cleanup_proof() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        let stage = store.stage(50, 4, source(), Cursor::new(b"first")).unwrap();
        assert!(store.stage(50, 4, source(), Cursor::new(b"other")).is_err());
        let other_root = tempfile::tempdir().unwrap();
        let other = StageStore::new(other_root.path()).unwrap();
        assert!(other
            .remove_after_verified_watermark(stage, other.cleanup_proof(source(), 4, 50))
            .is_err());
    }

    #[test]
    fn identical_retry_removes_only_its_redundant_payload_temp() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        let unrelated = store.records_dir().join(".payload-unrelated.tmp");
        fs::write(&unrelated, b"unrelated orphan").unwrap();

        let first = store.stage(51, 4, source(), Cursor::new(b"same")).unwrap();
        let original_record = first.record_path.clone();
        let original_marker = first.marker_path.clone();
        store.stage(51, 4, source(), Cursor::new(b"same")).unwrap();

        let mut record_names = fs::read_dir(store.records_dir())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        record_names.sort();
        assert_eq!(
            record_names,
            vec![
                unrelated.file_name().unwrap().to_owned(),
                original_record.file_name().unwrap().to_owned(),
            ]
        );
        assert!(unrelated.exists(), "unrelated orphan must stay retained");
        assert!(original_record.exists());
        assert!(original_marker.exists());
        assert_eq!(store.recover().unwrap().len(), 1);
        assert!(store
            .stage(51, 4, source(), Cursor::new(b"conflict"))
            .is_err());
        assert!(original_record.exists());
        assert!(original_marker.exists());
    }

    #[test]
    fn retry_after_marker_publish_fault_keeps_prior_marker_orphan_only() {
        let root = tempfile::tempdir().unwrap();
        let store = store(root.path(), StageFailurePoint::PublishMarker);
        assert!(store.stage(52, 4, source(), Cursor::new(b"same")).is_err());
        let mut prior_marker_orphans = fs::read_dir(store.markers_dir())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with(".marker-"))
            })
            .collect::<Vec<_>>();
        assert_eq!(prior_marker_orphans.len(), 1);
        let prior_marker_orphan = prior_marker_orphans.pop().unwrap();

        let receipt = store.stage(52, 4, source(), Cursor::new(b"same")).unwrap();
        let record_names = fs::read_dir(store.records_dir())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(
            record_names,
            vec![receipt.record_path.file_name().unwrap().to_owned()]
        );
        assert!(
            prior_marker_orphan.exists(),
            "first uncertain marker orphan must stay"
        );
        assert!(receipt.marker_path.exists());
        assert_eq!(store.recover().unwrap().len(), 1);
    }
}
