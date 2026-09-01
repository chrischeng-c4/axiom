//! Rebuildable, disk-backed event-id index.
//!
//! The canonical WAL owns durability. This index keeps only a fixed-size Bloom
//! filter in memory. Positive Bloom matches are verified against one of 4096
//! fixed-record shard files, so resident memory does not grow with ingest.

use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::RwLock,
};

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use crate::StoredEvent;

const SHARD_COUNT: usize = 4096;
const ENTRY_BYTES: usize = 40;
const DEFAULT_BLOOM_BYTES: usize = 32 * 1024 * 1024;

pub struct DedupeIndex {
    root: PathBuf,
    bloom: RwLock<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DedupeStats {
    pub count: u64,
    pub last_cursor: u64,
}

impl DedupeIndex {
    pub fn open(root: impl AsRef<Path>) -> Result<(Self, DedupeStats)> {
        let root = root.as_ref().join("indexes").join("dedupe");
        fs::create_dir_all(&root)
            .with_context(|| format!("create dedupe index root {}", root.display()))?;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
        let index = Self {
            root,
            bloom: RwLock::new(vec![0; DEFAULT_BLOOM_BYTES]),
        };
        let stats = match index.load_existing() {
            Ok((count, last_cursor)) => DedupeStats { count, last_cursor },
            Err(error) => {
                tracing::warn!(%error, "rebuilding damaged Sift dedupe index");
                index.reset()?;
                DedupeStats::default()
            }
        };
        Ok((index, stats))
    }

    pub fn lookup(&self, event_id: &str) -> Result<Option<u64>> {
        let digest = event_digest(event_id);
        if !self.bloom_contains(&digest) {
            return Ok(None);
        }
        let path = self.shard_path(&digest);
        if !path.exists() {
            return Ok(None);
        }
        let mut file = File::open(&path)
            .with_context(|| format!("open dedupe index shard {}", path.display()))?;
        let mut entry = [0u8; ENTRY_BYTES];
        loop {
            match file.read_exact(&mut entry) {
                Ok(()) => {
                    if entry[..32] == digest {
                        return Ok(Some(u64::from_le_bytes(
                            entry[32..40].try_into().expect("fixed cursor bytes"),
                        )));
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
                Err(error) => return Err(error.into()),
            }
        }
    }

    pub fn append_batch(&self, events: &[StoredEvent]) -> Result<()> {
        let entries = events
            .iter()
            .map(|event| (event_digest(&event.event.event_id), event.cursor))
            .collect::<Vec<_>>();
        self.append_entries(&entries)
    }

    pub fn reset(&self) -> Result<()> {
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().and_then(|value| value.to_str()) == Some("idx") {
                fs::remove_file(&path)
                    .with_context(|| format!("remove rebuildable index {}", path.display()))?;
            }
        }
        self.bloom
            .write()
            .expect("dedupe Bloom lock poisoned")
            .fill(0);
        Ok(())
    }

    fn append_entries(&self, entries: &[([u8; 32], u64)]) -> Result<()> {
        let mut shards = BTreeMap::<usize, Vec<([u8; 32], u64)>>::new();
        for (digest, cursor) in entries {
            shards
                .entry(shard_for(digest))
                .or_default()
                .push((*digest, *cursor));
        }
        for (shard, entries) in shards {
            let path = self.root.join(format!("{shard:03x}.idx"));
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .with_context(|| format!("append dedupe index shard {}", path.display()))?;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            for (digest, cursor) in &entries {
                file.write_all(digest)?;
                file.write_all(&cursor.to_le_bytes())?;
            }
        }
        let mut bloom = self.bloom.write().expect("dedupe Bloom lock poisoned");
        for (digest, _) in entries {
            bloom_insert(&mut bloom, &digest);
        }
        Ok(())
    }

    fn load_existing(&self) -> Result<(u64, u64)> {
        let mut count = 0u64;
        let mut last = 0u64;
        let mut bloom = self.bloom.write().expect("dedupe Bloom lock poisoned");
        bloom.fill(0);
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().and_then(|value| value.to_str()) != Some("idx") {
                continue;
            }
            let bytes = fs::read(&path)
                .with_context(|| format!("read dedupe index shard {}", path.display()))?;
            if bytes.len() % ENTRY_BYTES != 0 {
                bail!("dedupe index shard {} has a torn record", path.display());
            }
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            for entry in bytes.chunks_exact(ENTRY_BYTES) {
                let digest: [u8; 32] = entry[..32].try_into().expect("fixed digest bytes");
                let cursor =
                    u64::from_le_bytes(entry[32..40].try_into().expect("fixed cursor bytes"));
                bloom_insert(&mut bloom, &digest);
                count = count.saturating_add(1);
                last = last.max(cursor);
            }
        }
        Ok((count, last))
    }

    fn shard_path(&self, digest: &[u8; 32]) -> PathBuf {
        self.root.join(format!("{:03x}.idx", shard_for(digest)))
    }

    fn bloom_contains(&self, digest: &[u8; 32]) -> bool {
        let bloom = self.bloom.read().expect("dedupe Bloom lock poisoned");
        bloom_positions(digest, bloom.len() * 8)
            .into_iter()
            .all(|position| bloom[position / 8] & (1 << (position % 8)) != 0)
    }
}

fn event_digest(event_id: &str) -> [u8; 32] {
    Sha256::digest(event_id.as_bytes()).into()
}

fn shard_for(digest: &[u8; 32]) -> usize {
    (((digest[0] as usize) << 4) | ((digest[1] as usize) >> 4)) % SHARD_COUNT
}

fn bloom_insert(bloom: &mut [u8], digest: &[u8; 32]) {
    for position in bloom_positions(digest, bloom.len() * 8) {
        bloom[position / 8] |= 1 << (position % 8);
    }
}

fn bloom_positions(digest: &[u8; 32], bits: usize) -> [usize; 4] {
    let word = |offset| {
        u64::from_le_bytes(
            digest[offset..offset + 8]
                .try_into()
                .expect("fixed digest word"),
        ) as usize
            % bits
    };
    [word(0), word(8), word(16), word(24)]
}
