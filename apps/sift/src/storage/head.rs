//! Monotonic journal identity that survives hot-segment eviction.
//!
//! A cold event can live only in the committed archive. The local segment set
//! can therefore be empty while the next accepted event must still receive a
//! cursor larger than every earlier event. This small control record owns that
//! monotonic high-water mark and the retained event count.

use std::{fs, os::unix::fs::PermissionsExt, path::Path};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

const JOURNAL_HEAD_FORMAT_VERSION: u16 = 1;
const JOURNAL_HEAD_PATH: &str = "control/journal-head.json";

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JournalHead {
    format_version: u16,
    pub last_cursor: u64,
    pub retained_events: u64,
}

impl JournalHead {
    pub fn new(last_cursor: u64, retained_events: u64) -> Self {
        Self {
            format_version: JOURNAL_HEAD_FORMAT_VERSION,
            last_cursor,
            retained_events,
        }
    }

    pub fn load(root: &Path) -> Result<Option<Self>> {
        let path = root.join(JOURNAL_HEAD_PATH);
        if !path.exists() {
            return Ok(None);
        }
        let head: Self = serde_json::from_slice(
            &fs::read(&path).with_context(|| format!("read journal head {}", path.display()))?,
        )
        .with_context(|| format!("decode journal head {}", path.display()))?;
        if head.format_version != JOURNAL_HEAD_FORMAT_VERSION {
            bail!(
                "unsupported journal head format {}; expected {}",
                head.format_version,
                JOURNAL_HEAD_FORMAT_VERSION
            );
        }
        if head.retained_events > head.last_cursor {
            bail!("journal head retained event count exceeds its last cursor");
        }
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        Ok(Some(head))
    }

    pub fn persist(self, root: &Path) -> Result<()> {
        if self.retained_events > self.last_cursor {
            bail!("journal head retained event count exceeds its last cursor");
        }
        let path = root.join(JOURNAL_HEAD_PATH);
        storage_durable::atomic_write(
            &path,
            &serde_json::to_vec_pretty(&self)?,
            storage_durable::FsyncPolicy::Always,
        )
        .with_context(|| format!("persist journal head {}", path.display()))?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        Ok(())
    }
}
