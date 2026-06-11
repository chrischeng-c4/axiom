//! The comparison spec — one `arena.toml` describes the whole comparison.
//!
//! The cell name is the join key; each `[cells.targets.<id>]` sub-table is the
//! OPAQUE glue payload arena passes straight into a load profile and never
//! inspects. `gate`/`reason`/`floor` live on the PEER side (mirrors lumen's
//! `perf-baseline.json` per-cell-per-peer `{gate, baseline}`), so per-peer
//! classification (base WINS vs peer A but is EXEMPT vs peer B on one cell) is
//! representable.

use std::collections::BTreeMap;

use serde::Deserialize;

use rig::scenario::step::HttpRequest;

/// Default comparable scalar: tail latency (lower-is-better).
fn default_metric() -> String {
    "p99_ms".to_string()
}
/// Default baseline ratchet (lumen perf-baseline parity).
fn default_ratchet() -> f64 {
    0.8
}
/// Default per-cell gate when a peer omits it.
fn default_gate() -> String {
    "exempt".to_string()
}

/// The whole comparison.
#[derive(Debug, Clone, Deserialize)]
pub struct Spec {
    pub spec_version: u32,
    pub name: String,
    /// The target every ratio divides BY (`ratio = peer / base`).
    pub base: String,
    /// The comparable scalar; cells may override.
    #[serde(default = "default_metric")]
    pub metric: String,
    #[serde(default = "default_ratchet")]
    pub ratchet: f64,
    /// Path to the ratcheted baseline store; defaults to `.arena/baselines.json`.
    #[serde(default)]
    pub baseline: Option<String>,
    pub targets: BTreeMap<String, TargetSpec>,
    pub cells: Vec<Cell>,
}

/// How one target is measured. v1 implements `service` only.
#[derive(Debug, Clone, Deserialize)]
pub struct TargetSpec {
    /// `service` (rig loadgen) | `runtime` (meter, deferred) | `command` (deferred).
    pub kind: String,
    /// Required for `kind = "service"`: the open-loop load shape (request comes
    /// from each cell).
    #[serde(default)]
    pub load: Option<LoadShape>,
    /// `kind = "runtime"` (deferred): the binary meter spawns.
    #[serde(default)]
    pub exec: Option<String>,
    /// `kind = "runtime"` (deferred): the meter level.
    #[serde(default)]
    pub level: Option<String>,
}

/// A [`rig::scenario::load::LoadProfile`] minus its `request` (the request is
/// supplied per-cell).
#[derive(Debug, Clone, Deserialize)]
pub struct LoadShape {
    pub target_qps: u32,
    pub workers: u32,
    pub duration_secs: u64,
    #[serde(default)]
    pub warmup_secs: u64,
}

/// One logical workload, expressed per-target.
#[derive(Debug, Clone, Deserialize)]
pub struct Cell {
    pub name: String,
    /// Optional per-cell metric override (e.g. a runtime cell comparing cpu).
    #[serde(default)]
    pub metric: Option<String>,
    /// One entry per target. The `base` target needs no gate.
    pub targets: BTreeMap<String, CellTarget>,
}

/// The per-target payload for one cell. `request` is the glue arena never reads.
#[derive(Debug, Clone, Deserialize)]
pub struct CellTarget {
    /// `win` | `exempt` | `target` (ignored on the base target).
    #[serde(default = "default_gate")]
    pub gate: String,
    /// Human reason an EXEMPT cell is not gated (surfaced in the report).
    #[serde(default)]
    pub reason: Option<String>,
    /// Floor for a `target` gate.
    #[serde(default)]
    pub floor: Option<f64>,
    /// The opaque per-target request, deserialized straight into rig's
    /// `HttpRequest`. arena moves it into the load profile untouched.
    pub request: HttpRequest,
}

impl Spec {
    /// Parse a spec from TOML text, validating the structural invariants arena
    /// relies on (base target present, every cell names the base, service
    /// targets carry a load shape).
    pub fn parse(text: &str) -> Result<Spec, String> {
        let spec: Spec = toml::from_str(text).map_err(|e| format!("invalid arena spec: {e}"))?;
        if !spec.targets.contains_key(&spec.base) {
            return Err(format!(
                "base target `{}` is not declared in [targets]",
                spec.base
            ));
        }
        for (id, t) in &spec.targets {
            if t.kind == "service" && t.load.is_none() {
                return Err(format!(
                    "service target `{id}` is missing its [targets.{id}.load] block"
                ));
            }
        }
        for cell in &spec.cells {
            if !cell.targets.contains_key(&spec.base) {
                return Err(format!(
                    "cell `{}` does not include the base target `{}`",
                    cell.name, spec.base
                ));
            }
        }
        Ok(spec)
    }

    /// The effective metric for a cell (cell override > spec default).
    pub fn cell_metric<'a>(&'a self, cell: &'a Cell) -> &'a str {
        cell.metric.as_deref().unwrap_or(&self.metric)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = r#"
spec_version = 1
name = "demo"
base = "a"
metric = "p99_ms"

[targets.a]
kind = "service"
[targets.a.load]
target_qps = 100
workers = 8
duration_secs = 10

[targets.b]
kind = "service"
[targets.b.load]
target_qps = 100
workers = 8
duration_secs = 10

[[cells]]
name = "c1"
[cells.targets.a]
request = { method = "POST", url = "http://127.0.0.1:1/s", body = "{}" }
[cells.targets.b]
gate = "win"
request = { method = "POST", url = "http://127.0.0.1:2/s", body = "{}" }
"#;

    #[test]
    fn parses_and_validates() {
        let spec = Spec::parse(SPEC).unwrap();
        assert_eq!(spec.base, "a");
        assert_eq!(spec.ratchet, 0.8);
        assert_eq!(spec.cells.len(), 1);
        assert_eq!(spec.cells[0].targets["b"].gate, "win");
        assert_eq!(spec.cell_metric(&spec.cells[0]), "p99_ms");
    }

    #[test]
    fn rejects_missing_base_target() {
        let bad = SPEC.replace("base = \"a\"", "base = \"z\"");
        assert!(Spec::parse(&bad).is_err());
    }

    #[test]
    fn rejects_service_target_without_load() {
        let bad = "spec_version = 1\nname=\"d\"\nbase=\"a\"\n[targets.a]\nkind=\"service\"\n[[cells]]\nname=\"c\"\n[cells.targets.a]\nrequest = { method=\"GET\", url=\"http://x/\" }\n";
        assert!(Spec::parse(bad).is_err());
    }
}
