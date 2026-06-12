// SPEC-MANAGED: projects/rig/tech-design/semantic/source/projects-rig-src-pins-baseline-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! JSON baseline store: `.rig/baselines.json`.
//!
//! Tiny cardinality (scenarios × metrics), so a diffable JSON file beats a
//! SQLite dependency — and matches lumen's `tests/perf-baseline.json`
//! precedent. Keyed `<scenario_id>::<metric>::<host_fingerprint>` so a
//! laptop baseline never gates a CI runner.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-pins-baseline-rs.md#source
pub struct BaselineEntry {
    pub value: f64,
    /// Unix seconds at record time.
    pub recorded_at: u64,
    pub tool_version: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-pins-baseline-rs.md#source
pub struct BaselineStore {
    #[serde(default)]
    entries: BTreeMap<String, BaselineEntry>,
    #[serde(skip)]
    path: Option<PathBuf>,
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-pins-baseline-rs.md#source
pub fn host_fingerprint() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn key(scenario_id: &str, metric: &str) -> String {
    format!("{scenario_id}::{metric}::{}", host_fingerprint())
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-pins-baseline-rs.md#source
impl BaselineStore {
    /// Load from `<dir>/.rig/baselines.json` (absent file = empty store).
    pub fn load(dir: &Path) -> Self {
        Self::load_at(dir.join(".rig").join("baselines.json"))
    }

    /// Load from an explicit JSON path (absent file = empty store). Lets a
    /// sibling tool namespace its own store, e.g. `.arena/baselines.json`,
    /// while reusing the same host-scoped key scheme and ratchet semantics.
    pub fn load_at(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mut store = match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        };
        store.path = Some(path);
        store
    }

    pub fn get(&self, scenario_id: &str, metric: &str) -> Option<&BaselineEntry> {
        self.entries.get(&key(scenario_id, metric))
    }

    pub fn record(&mut self, scenario_id: &str, metric: &str, value: f64) {
        let recorded_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.entries.insert(
            key(scenario_id, metric),
            BaselineEntry {
                value,
                recorded_at,
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        );
    }

    /// Persist back to the path it was loaded from.
    pub fn save(&self) -> std::io::Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = BaselineStore::load(tmp.path());
        assert!(store.get("lumen/load/search_qps", "p99_ms").is_none());
        store.record("lumen/load/search_qps", "p99_ms", 12.5);
        store.save().unwrap();

        let reloaded = BaselineStore::load(tmp.path());
        let entry = reloaded.get("lumen/load/search_qps", "p99_ms").unwrap();
        assert_eq!(entry.value, 12.5);
        assert!(entry.recorded_at > 0);
    }

    #[test]
    fn keys_are_host_scoped() {
        assert!(key("a/b/c", "p99_ms").contains(&host_fingerprint()));
    }
}
// CODEGEN-END
