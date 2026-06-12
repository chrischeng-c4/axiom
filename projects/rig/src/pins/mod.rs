// SPEC-MANAGED: projects/rig/tech-design/semantic/source/projects-rig-src-pins-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Declarative pins: floor + ratchet gates over scenario metrics.
//!
//! Pin TOML (mamba pin shape, lumen perf-baseline ratchet spirit):
//!
//! ```toml
//! [[pins]]
//! issue    = "lumen#412"
//! scenario = "load/search_qps"        # scenario_id or its dimension/case suffix
//! metric   = "p99_ms"                 # lower-is-better metrics only (latency, error_rate)
//! floor    = 25.0                     # absolute ceiling: worse than this = RED
//! ratchet  = 0.8                      # vs baseline: new > baseline/ratchet = regression
//! ```
//!
//! `floor` keeps mamba's name: it is the performance floor — sinking below
//! it (numerically: exceeding the value, since these metrics are
//! lower-is-better) is a regression. A pin without a recorded baseline is
//! Info by default; `RIG_STRICT=1` makes it gate.

pub mod baseline;

use serde::{Deserialize, Serialize};
use std::path::Path;

pub use baseline::BaselineStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-pins-mod-rs.md#source
pub struct Pin {
    /// Provenance: tracker reference.
    pub issue: String,
    /// Scenario id (`suite/dimension/case`) or its `dimension/case` suffix.
    pub scenario: String,
    pub metric: String,
    /// Absolute ceiling (lower-is-better metrics). Exceeding it = RED.
    #[serde(default)]
    pub floor: Option<f64>,
    /// Baseline-relative gate: new > baseline / ratchet = regression.
    #[serde(default)]
    pub ratchet: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
struct PinFile {
    #[serde(default)]
    pins: Vec<Pin>,
}

/// Collect every `[[pins]]` entry from `*.toml` under `dir` (sorted walk).
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-pins-mod-rs.md#source
pub fn load_pins(dir: &Path) -> Result<Vec<Pin>, String> {
    let mut paths = Vec::new();
    collect(dir, &mut paths).map_err(|e| format!("could not walk `{}`: {e}", dir.display()))?;
    paths.sort();
    let mut pins = Vec::new();
    for p in paths {
        let text = std::fs::read_to_string(&p)
            .map_err(|e| format!("unreadable `{}`: {e}", p.display()))?;
        let file: PinFile =
            toml::from_str(&text).map_err(|e| format!("bad pin file `{}`: {e}", p.display()))?;
        pins.extend(file.pins);
    }
    Ok(pins)
}

fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "toml") {
            out.push(path);
        }
    }
    Ok(())
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-pins-mod-rs.md#source
impl Pin {
    /// Does this pin gate the given scenario id?
    pub fn matches(&self, scenario_id: &str) -> bool {
        scenario_id == self.scenario || scenario_id.ends_with(&format!("/{}", self.scenario))
    }
}

/// Gate verdict for one pin against one measured value.
#[derive(Debug, Clone, PartialEq)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-pins-mod-rs.md#source
pub enum GateOutcome {
    Pass,
    FloorBreach {
        value: f64,
        floor: f64,
    },
    RatchetBreach {
        value: f64,
        baseline: f64,
        limit: f64,
    },
    NoBaseline {
        value: f64,
    },
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-pins-mod-rs.md#source
pub fn gate(pin: &Pin, scenario_id: &str, value: f64, store: &BaselineStore) -> GateOutcome {
    if let Some(floor) = pin.floor {
        if value > floor {
            return GateOutcome::FloorBreach { value, floor };
        }
    }
    if let Some(ratchet) = pin.ratchet {
        match store.get(scenario_id, &pin.metric) {
            Some(entry) => {
                let limit = entry.value / ratchet.clamp(0.01, 1.0);
                if value > limit {
                    return GateOutcome::RatchetBreach {
                        value,
                        baseline: entry.value,
                        limit,
                    };
                }
            }
            None => return GateOutcome::NoBaseline { value },
        }
    }
    GateOutcome::Pass
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pin(floor: Option<f64>, ratchet: Option<f64>) -> Pin {
        Pin {
            issue: "lumen#412".into(),
            scenario: "load/search_qps".into(),
            metric: "p99_ms".into(),
            floor,
            ratchet,
        }
    }

    #[test]
    fn suffix_matching() {
        let p = pin(Some(25.0), None);
        assert!(p.matches("lumen/load/search_qps"));
        assert!(p.matches("load/search_qps"));
        assert!(!p.matches("lumen/load/other"));
    }

    #[test]
    fn floor_gates_absolutely() {
        let tmp = tempfile::tempdir().unwrap();
        let store = BaselineStore::load(tmp.path());
        let p = pin(Some(25.0), None);
        assert_eq!(
            gate(&p, "lumen/load/search_qps", 10.0, &store),
            GateOutcome::Pass
        );
        assert!(matches!(
            gate(&p, "lumen/load/search_qps", 30.0, &store),
            GateOutcome::FloorBreach { .. }
        ));
    }

    #[test]
    fn ratchet_gates_against_baseline() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = BaselineStore::load(tmp.path());
        store.record("lumen/load/search_qps", "p99_ms", 10.0);
        let p = pin(None, Some(0.8));
        // limit = 10 / 0.8 = 12.5
        assert_eq!(
            gate(&p, "lumen/load/search_qps", 12.0, &store),
            GateOutcome::Pass
        );
        assert!(matches!(
            gate(&p, "lumen/load/search_qps", 13.0, &store),
            GateOutcome::RatchetBreach { .. }
        ));
    }

    #[test]
    fn missing_baseline_is_its_own_outcome() {
        let tmp = tempfile::tempdir().unwrap();
        let store = BaselineStore::load(tmp.path());
        let p = pin(None, Some(0.8));
        assert!(matches!(
            gate(&p, "lumen/load/search_qps", 13.0, &store),
            GateOutcome::NoBaseline { .. }
        ));
    }

    #[test]
    fn pin_files_load_recursively() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("search_p99.toml"),
            r#"
[[pins]]
issue = "lumen#412"
scenario = "load/search_qps"
metric = "p99_ms"
floor = 25.0
ratchet = 0.8
"#,
        )
        .unwrap();
        let pins = load_pins(tmp.path()).unwrap();
        assert_eq!(pins.len(), 1);
        assert_eq!(pins[0].metric, "p99_ms");
    }
}
// CODEGEN-END
