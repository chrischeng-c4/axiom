// HANDWRITE-BEGIN gap="sift-epoch-bucket-router" tracker="1659" reason="Persist and validate 4096-bucket epoch maps and route future cursors without changing historical ownership."
use std::{
    path::{Path, PathBuf},
    sync::RwLock,
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const VIRTUAL_BUCKETS: usize = 4_096;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EpochMap {
    pub epoch: u64,
    pub activated_at_cursor: u64,
    pub bucket_to_shard: Vec<u16>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Route {
    pub epoch: u64,
    pub shard: u16,
    pub bucket: u16,
}

pub struct ShardRouter {
    path: PathBuf,
    epochs: RwLock<Vec<EpochMap>>,
}

impl ShardRouter {
    pub fn open(root: impl AsRef<Path>, initial_shards: u16) -> Result<Self> {
        if initial_shards == 0 {
            bail!("initial logical shard count must be greater than zero");
        }
        let path = epoch_path(root.as_ref());
        let epochs = if path.exists() {
            serde_json::from_slice(&std::fs::read(&path)?)
                .with_context(|| format!("decode epoch maps {}", path.display()))?
        } else {
            let epochs = vec![EpochMap {
                epoch: 1,
                activated_at_cursor: 0,
                bucket_to_shard: (0..VIRTUAL_BUCKETS)
                    .map(|bucket| (bucket as u16) % initial_shards)
                    .collect(),
            }];
            write_epoch_maps(root.as_ref(), &epochs)?;
            epochs
        };
        validate_epochs(&epochs)?;
        Ok(Self {
            path,
            epochs: RwLock::new(epochs),
        })
    }

    pub fn route(&self, event_id: &str, cursor: u64) -> Route {
        let bucket = bucket_for(event_id);
        let epochs = self.epochs.read().expect("epoch map lock poisoned");
        let epoch = epochs
            .iter()
            .rev()
            .find(|epoch| epoch.activated_at_cursor <= cursor)
            .unwrap_or(&epochs[0]);
        Route {
            epoch: epoch.epoch,
            shard: epoch.bucket_to_shard[usize::from(bucket)],
            bucket,
        }
    }

    pub fn activate(&self, activated_at_cursor: u64, buckets: Vec<u16>) -> Result<EpochMap> {
        if buckets.len() != VIRTUAL_BUCKETS {
            bail!("epoch map must contain exactly {VIRTUAL_BUCKETS} buckets");
        }
        let mut epochs = self.epochs.write().expect("epoch map lock poisoned");
        let last = epochs.last().context("epoch map history is empty")?;
        if activated_at_cursor <= last.activated_at_cursor {
            bail!(
                "new epoch activation cursor {activated_at_cursor} must be after {}",
                last.activated_at_cursor
            );
        }
        let next = EpochMap {
            epoch: last.epoch + 1,
            activated_at_cursor,
            bucket_to_shard: buckets,
        };
        let mut updated = epochs.clone();
        updated.push(next.clone());
        validate_epochs(&updated)?;
        storage_durable::atomic_write(
            &self.path,
            &serde_json::to_vec_pretty(&updated)?,
            storage_durable::FsyncPolicy::Always,
        )?;
        *epochs = updated;
        Ok(next)
    }

    pub fn epochs(&self) -> Vec<EpochMap> {
        self.epochs.read().expect("epoch map lock poisoned").clone()
    }
}

pub(crate) fn bucket_for(event_id: &str) -> u16 {
    let digest = Sha256::digest(event_id.as_bytes());
    u16::from_be_bytes([digest[0], digest[1]]) & 0x0fff
}

pub(crate) fn write_epoch_maps(root: &Path, epochs: &[EpochMap]) -> Result<()> {
    validate_epochs(epochs)?;
    storage_durable::atomic_write(
        epoch_path(root),
        &serde_json::to_vec_pretty(epochs)?,
        storage_durable::FsyncPolicy::Always,
    )
}

fn epoch_path(root: &Path) -> PathBuf {
    root.join("segments").join("epochs.json")
}

fn validate_epochs(epochs: &[EpochMap]) -> Result<()> {
    if epochs.is_empty() {
        bail!("epoch map history must not be empty");
    }
    let mut previous_epoch = 0;
    let mut previous_cursor = None;
    for epoch in epochs {
        if epoch.epoch <= previous_epoch {
            bail!("epoch ids must be strictly increasing");
        }
        if epoch.bucket_to_shard.len() != VIRTUAL_BUCKETS {
            bail!(
                "epoch {} has {} buckets; expected {VIRTUAL_BUCKETS}",
                epoch.epoch,
                epoch.bucket_to_shard.len()
            );
        }
        if let Some(previous) = previous_cursor {
            if epoch.activated_at_cursor <= previous {
                bail!("epoch activation cursors must be strictly increasing");
            }
        }
        previous_epoch = epoch.epoch;
        previous_cursor = Some(epoch.activated_at_cursor);
    }
    Ok(())
}
// HANDWRITE-END
