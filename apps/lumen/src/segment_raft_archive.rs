//! Immutable `LSEGRAFT` v1 generation archive framing.
//!
//! This module only reads a caller-pinned generation directory.  It never
//! resolves `CURRENT` and never builds an Engine while writing.

use super::*;
use anyhow::ensure;

pub(crate) const MAGIC: &[u8; 8] = b"LSEGRAFT";
const VERSION: u8 = 1;
const COPY_BUF: usize = 64 * 1024;

struct ArchiveCandidate {
    record: GenerationRecord,
    collections: usize,
}

pub(crate) fn write_archive<W: Write + ?Sized>(
    root: &Path,
    sequence: u64,
    out: &mut W,
) -> Result<()> {
    let files = inventory(root)?;
    out.write_all(MAGIC)?;
    out.write_all(&[VERSION])?;
    out.write_all(&sequence.to_le_bytes())?;
    out.write_all(&(files.len() as u64).to_le_bytes())?;
    for (relative, path) in files {
        let encoded = relative.as_bytes();
        out.write_all(&(encoded.len() as u64).to_le_bytes())?;
        out.write_all(encoded)?;
        let len = std::fs::metadata(&path)?.len();
        out.write_all(&len.to_le_bytes())?;
        let digest = digest_file(&path)?;
        out.write_all(&digest)?;
        let mut file = File::open(&path)?;
        copy_exact(&mut file, out, len, None)?;
    }
    Ok(())
}

/// Decode into a caller-created staging directory. The directory must be empty
/// and below the destination root; callers validate/cold-open then publish it.
fn read_header<R: Read + ?Sized>(input: &mut R) -> Result<(u64, u64)> {
    let mut magic = [0; 8];
    input.read_exact(&mut magic)?;
    ensure!(magic == *MAGIC, "unsupported segment Raft archive magic");
    let mut version = [0; 1];
    input.read_exact(&mut version)?;
    ensure!(
        version[0] == VERSION,
        "unsupported segment Raft archive version {}",
        version[0]
    );
    let sequence = read_u64(input)?;
    let count = read_u64(input)?;
    Ok((sequence, count))
}

fn stage_archive<R: Read + ?Sized>(
    input: &mut R,
    staging: &Path,
    sequence: u64,
    count: u64,
) -> Result<ArchiveCandidate> {
    ensure_empty_directory(staging)?;
    let path_limit = filesystem_path_limit(staging)?;
    let mut seen = BTreeSet::new();
    for _ in 0..count {
        let path_len = read_u64(input)?;
        let path_len =
            usize::try_from(path_len).map_err(|_| anyhow!("archive path length overflow"))?;
        ensure!(
            path_len > 0 && path_len < path_limit,
            "archive path cannot fit the destination filesystem"
        );
        let mut raw = vec![0; path_len];
        input.read_exact(&mut raw)?;
        let relative = std::str::from_utf8(&raw).context("archive path is not UTF-8")?;
        validate_relative(relative)?;
        ensure!(
            seen.insert(relative.to_owned()),
            "duplicate archive path {relative}"
        );
        let length = read_u64(input)?;
        let mut expected = [0; 32];
        input.read_exact(&mut expected)?;
        let target = staging.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)?;
        let actual = copy_exact(input, &mut output, length, Some(&expected))?;
        output.flush()?;
        ensure!(actual == expected, "archive digest mismatch for {relative}");
    }
    let mut trailing = [0; 1];
    ensure!(
        input.read(&mut trailing)? == 0,
        "trailing bytes after segment Raft archive"
    );
    let manifest = read_generation_manifest(staging)?;
    ensure!(
        manifest.schema_version == 2,
        "archive requires a v2 generation manifest"
    );
    ensure!(
        manifest.checkpoint_sequence == sequence,
        "archive sequence does not match manifest"
    );
    let mut expected = BTreeSet::from([GENERATION_MANIFEST_FILE.to_owned()]);
    for collection in &manifest.collections {
        expected.insert(format!(
            "{}/{}",
            collection_checkpoint_dir_name(&collection.collection_id),
            CHECKPOINT_SCHEMA_FILE
        ));
        for segment in &collection.segments {
            expected.insert(segment.path.clone());
            if let Some(rows) = &segment.local_rows {
                expected.insert(rows.path.clone());
            }
        }
    }
    ensure!(
        seen == expected,
        "archive file inventory differs from the complete generation manifest"
    );
    let record = GenerationRecord {
        name: GenerationName::parse(format!("gen-{sequence}-rev-{}", manifest.revision))?,
        path: staging.to_path_buf(),
        sequence,
        revision: manifest.revision,
        legacy: false,
        previous: manifest.previous.map(GenerationName::parse).transpose()?,
    };
    let collections =
        validate_generation_layout(&record).context("validate staged archive generation")?;
    Ok(ArchiveCandidate {
        record,
        collections,
    })
}

/// A path is metadata, never an allocation sized by an unchecked peer field.
/// The limit comes from the receiving filesystem, not the document API.
#[cfg(unix)]
fn filesystem_path_limit(staging: &Path) -> Result<usize> {
    use std::os::fd::AsRawFd;
    let directory = File::open(staging)?;
    // SAFETY: directory owns an open descriptor for the duration of fpathconf.
    let limit = unsafe { libc::fpathconf(directory.as_raw_fd(), libc::_PC_PATH_MAX) };
    ensure!(limit > 0, "destination filesystem has no finite path limit");
    Ok(usize::try_from(limit)?)
}

#[cfg(not(unix))]
fn filesystem_path_limit(_staging: &Path) -> Result<usize> {
    bail!("segment Raft archive requires the supported Unix filesystem backend")
}

fn inventory(root: &Path) -> Result<Vec<(String, PathBuf)>> {
    fn visit(root: &Path, here: &Path, out: &mut Vec<(String, PathBuf)>) -> Result<()> {
        for entry in std::fs::read_dir(here)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let path = entry.path();
            ensure!(
                !ty.is_symlink(),
                "generation archive refuses symlink {}",
                path.display()
            );
            if ty.is_dir() {
                visit(root, &path, out)?;
            } else {
                ensure!(
                    ty.is_file(),
                    "generation archive refuses non-regular file {}",
                    path.display()
                );
                let rel = path
                    .strip_prefix(root)?
                    .to_str()
                    .ok_or_else(|| anyhow!("non-UTF8 generation path"))?
                    .to_owned();
                validate_relative(&rel)?;
                out.push((rel, path));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    visit(root, root, &mut out)?;
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}
fn validate_relative(path: &str) -> Result<()> {
    ensure!(
        !path.is_empty() && !path.starts_with('/') && !path.contains(['\\', '\0']),
        "invalid archive path"
    );
    ensure!(
        path.split('/')
            .all(|p| !p.is_empty() && p != "." && p != ".."),
        "invalid archive path {path}"
    );
    Ok(())
}
fn digest_file(path: &Path) -> Result<[u8; 32]> {
    let mut f = File::open(path)?;
    let mut h = Sha256::new();
    let mut b = [0; COPY_BUF];
    loop {
        let n = f.read(&mut b)?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(h.finalize().into())
}
fn copy_exact<R: Read + ?Sized, W: Write + ?Sized>(
    input: &mut R,
    out: &mut W,
    len: u64,
    expected: Option<&[u8; 32]>,
) -> Result<[u8; 32]> {
    let mut left = len;
    let mut h = Sha256::new();
    let mut b = [0; COPY_BUF];
    while left > 0 {
        let want = usize::try_from(left.min(COPY_BUF as u64)).unwrap();
        input.read_exact(&mut b[..want])?;
        out.write_all(&b[..want])?;
        h.update(&b[..want]);
        left -= want as u64;
    }
    let got: [u8; 32] = h.finalize().into();
    if let Some(expected) = expected {
        ensure!(&got == expected, "archive payload digest mismatch");
    }
    Ok(got)
}
fn read_u64<R: Read + ?Sized>(r: &mut R) -> Result<u64> {
    let mut b = [0; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}
fn ensure_empty_directory(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    ensure!(
        std::fs::read_dir(path)?.next().is_none(),
        "archive staging directory is not empty"
    );
    Ok(())
}
/// Cleanup only the private path this invocation created. A committed directory
/// has moved away from this name and is never removed by this guard.
struct StageCleanup(PathBuf);
impl Drop for StageCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl SegmentRdbStore {
    pub(crate) fn validate_raft_archive(&self, input: &mut dyn Read) -> Result<()> {
        let _permit = self.save_gate.lock_owned();
        let (sequence, count) = read_header(input)?;
        let (_, staged) = self.begin_next_generation(sequence)?;
        let _cleanup = StageCleanup(staged.path().to_owned());
        let candidate = stage_archive(input, staged.path(), sequence, count)?;
        // Receiver validation may build the backend; snapshot encoding never does.
        let engine = Engine::new();
        self.reopen_once(&engine, &candidate.record, candidate.collections)?;
        Ok(())
    }

    pub(crate) fn restore_raft_archive(
        &self,
        live: &Arc<Engine>,
        input: &mut dyn Read,
        activated: impl FnOnce(u64),
    ) -> Result<u64> {
        let _permit = self.save_gate.lock_owned();
        let (sequence, count) = read_header(input)?;
        let (revision, staged) = self.begin_next_generation(sequence)?;
        let _cleanup = StageCleanup(staged.path().to_owned());
        let mut candidate = stage_archive(input, staged.path(), sequence, count)?;
        let mut manifest = read_generation_manifest(staged.path())?;
        let previous = self
            .current_record()?
            .filter(|old| old.sequence <= sequence)
            .map(|old| old.name);
        manifest.revision = revision;
        manifest.previous = previous.as_ref().map(|name| name.as_str().to_owned());
        write_generation_manifest(staged.path(), &manifest)?;
        candidate.record.name = staged.generation().clone();
        candidate.record.revision = revision;
        candidate.record.previous = previous;
        validate_generation_layout(&candidate.record)?;
        let fresh = Engine::new();
        self.reopen_once(&fresh, &candidate.record, candidate.collections)?;
        let final_path = self.generations.generation_path(staged.generation());
        fresh.bind_legacy_checkpoint_origin(&final_path)?;
        self.retain_root_for(&fresh);
        self.background
            .pin_loaded_engine(staged.generation().as_str().to_owned(), &fresh)?;
        let encoded_manifest = serde_json::to_vec(&manifest)?;
        let inhibition = live
            .capture_barrier
            .apply()
            .inhibit_checkpoints_for_restore();
        let name = match self.generations.commit(staged) {
            Ok(name) => name,
            Err(error) => {
                if error.class() == storage_durable::CommitFailureClass::CommitUncertain {
                    inhibition.mark_uncertain();
                }
                return Err(error.into());
            }
        };
        // There are no more file reads or backend reconstruction after CURRENT.
        let activation = inhibition.activation_apply();
        if let Err(error) = live.activate_replacement(fresh) {
            inhibition.mark_uncertain();
            return Err(error).context("Raft snapshot published; live activation requires restart");
        }
        activation.initialize_sequence(sequence);
        activated(sequence);
        *self
            .verified_catalog
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some((name, encoded_manifest));
        Ok(sequence)
    }
}

impl SegmentRdbStore {
    pub(crate) fn restore_legacy_raft_snapshot(
        &self,
        live: &Arc<Engine>,
        rdb: crate::rdb::RdbSnapshot,
        activated: impl FnOnce(u64),
    ) -> Result<u64> {
        let sequence = rdb.up_to_seq;
        let fresh = Arc::new(Engine::new());
        fresh.restore(rdb.snapshot)?;
        fresh.capture_barrier.apply().initialize_sequence(sequence);
        let inhibition = live
            .capture_barrier
            .apply()
            .inhibit_checkpoints_for_restore();
        let permit = self.save_gate.lock_owned();
        if let Err(error) = self.save_inner_permitted(
            &fresh,
            sequence,
            true,
            permit,
            SaveIntent::RaftRestore,
            None,
        ) {
            if error.chain().any(|cause| {
                cause
                    .downcast_ref::<storage_durable::CommitError>()
                    .is_some_and(|commit| {
                        commit.class() == storage_durable::CommitFailureClass::CommitUncertain
                    })
            }) {
                inhibition.mark_uncertain();
            }
            return Err(error);
        }
        let fresh = match Arc::try_unwrap(fresh) {
            Ok(fresh) => fresh,
            Err(_) => {
                inhibition.mark_uncertain();
                bail!("legacy Raft restore published; unexpected candidate owner requires restart");
            }
        };
        let activation = inhibition.activation_apply();
        if let Err(error) = live.activate_replacement(fresh) {
            inhibition.mark_uncertain();
            return Err(error)
                .context("legacy Raft restore published; activation requires restart");
        }
        activation.initialize_sequence(sequence);
        activated(sequence);
        Ok(sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (tempfile::TempDir, Vec<u8>) {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        let name = store.save_required(&engine, 2).unwrap();
        let mut bytes = Vec::new();
        write_archive(&store.generations.generation_path(&name), 2, &mut bytes).unwrap();
        (dir, bytes)
    }

    #[test]
    fn archive_roundtrip_publishes_exact_sequence_and_survives_cold_open() {
        let (_source, bytes) = fixture();
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let live = Arc::new(Engine::new());
        store.validate_raft_archive(&mut bytes.as_slice()).unwrap();
        let mut observed = None;
        assert_eq!(
            store
                .restore_raft_archive(&live, &mut bytes.as_slice(), |seq| observed = Some(seq))
                .unwrap(),
            2
        );
        assert_eq!(observed, Some(2));
        assert_eq!(
            store.load_current_generation().unwrap().unwrap().sequence,
            2
        );
        // A subsequent checkpoint uses the imported immutable generation path.
        store.save_required(&live, 2).unwrap();
    }

    #[test]
    fn malformed_archive_metadata_and_payload_preserve_current() {
        let (_source, bytes) = fixture();
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let live = Arc::new(Engine::new());
        store.save_required(&live, 1).unwrap();
        let before = std::fs::read(dir.path().join("CURRENT")).unwrap();
        let mut mutations = Vec::new();
        for offset in [0, 8, 9, bytes.len() - 1] {
            let mut bad = bytes.clone();
            bad[offset] ^= 0xff;
            mutations.push(bad);
        }
        mutations.push(bytes[..bytes.len() - 1].to_vec());
        let mut trailing = bytes.clone();
        trailing.push(0);
        mutations.push(trailing);
        let mut huge_path = bytes.clone();
        huge_path[25..33].copy_from_slice(&u64::MAX.to_le_bytes());
        mutations.push(huge_path);
        let mut traversal = bytes.clone();
        traversal[33..36].copy_from_slice(b"../");
        mutations.push(traversal);
        let mut duplicate = bytes.clone();
        duplicate[17..25].copy_from_slice(&2u64.to_le_bytes());
        duplicate.extend_from_slice(&bytes[25..]);
        mutations.push(duplicate);
        for (case, bad) in mutations.iter().enumerate() {
            assert!(
                store.validate_raft_archive(&mut bad.as_slice()).is_err(),
                "validation case {case}"
            );
            assert!(
                store
                    .restore_raft_archive(&live, &mut bad.as_slice(), |_| panic!(
                        "invalid archive activated"
                    ))
                    .is_err(),
                "restore case {case}"
            );
            assert_eq!(
                std::fs::read(dir.path().join("CURRENT")).unwrap(),
                before,
                "case {case}"
            );
        }
    }

    #[test]
    fn archive_manifest_byte_change_needs_digest_verification() {
        let (_source, mut bytes) = fixture();
        assert_eq!(bytes.last(), Some(&b'\n'));
        // Both encodings are valid identical JSON values. Only the byte digest
        // can reject this corrupted payload before a fresh backend is opened.
        *bytes.last_mut().unwrap() = b' ';
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(
            store.validate_raft_archive(&mut bytes.as_slice()).is_err(),
            "archive must verify the digest even when changed manifest bytes still decode"
        );
    }

    #[test]
    fn archive_refuses_payload_files_not_named_in_the_complete_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine
            .create_collection(
                "docs",
                serde_json::from_value(serde_json::json!({
                    "fields": {"kind": {"type": "keyword"}}
                }))
                .unwrap(),
            )
            .unwrap();
        let name = store.save_required(&engine, 2).unwrap();
        let root = store.generations.generation_path(&name);
        std::fs::write(
            root.join(collection_checkpoint_dir_name("docs"))
                .join("unlisted.payload"),
            b"unreferenced bytes",
        )
        .unwrap();
        let mut bytes = Vec::new();
        write_archive(&root, 2, &mut bytes).unwrap();
        let receiver = tempfile::tempdir().unwrap();
        let receiver = SegmentRdbStore::new(receiver.path()).unwrap();
        assert!(
            receiver
                .validate_raft_archive(&mut bytes.as_slice())
                .is_err(),
            "archive must reject a payload file absent from the complete manifest"
        );
    }

    #[cfg(unix)]
    #[test]
    fn archive_inventory_refuses_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink("/private/tmp", dir.path().join("escape")).unwrap();
        assert!(write_archive(dir.path(), 1, &mut Vec::new()).is_err());
    }
}
