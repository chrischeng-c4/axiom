---
id: projects-arena-src-spec-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/arena/src/spec.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/arena/src/spec.rs`, captured as a rust-source-unit (td_ast) item-tree
during arena standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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

/// How one target is measured — which rig transport drives it. `http` (alias
/// `service`) and `postgres` are implemented; both run on rig's one open-loop
/// scheduler so their numbers are comparable.
#[derive(Debug, Clone, Deserialize)]
pub struct TargetSpec {
    /// `http` (alias `service`, rig HTTP loadgen) | `postgres` (rig pg transport).
    pub kind: String,
    /// The open-loop load shape (the per-op payload comes from each cell).
    #[serde(default)]
    pub load: Option<LoadShape>,
    /// Required for `kind = "postgres"`: the libpq-style connection DSN.
    #[serde(default)]
    pub dsn: Option<String>,
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

/// The per-target payload for one cell — the glue arena never reads, just
/// forwards to the target's transport. An `http` target uses `request`; a
/// `postgres` target uses `query`.
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
    /// `http` target payload: deserialized straight into rig's `HttpRequest`.
    #[serde(default)]
    pub request: Option<HttpRequest>,
    /// `postgres` target payload: the SQL prepared + executed each tick.
    #[serde(default)]
    pub query: Option<String>,
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
            match t.kind.as_str() {
                "service" | "http" | "postgres" => {}
                other => {
                    return Err(format!(
                        "target `{id}` has unsupported kind `{other}` (use `http` or `postgres`)"
                    ))
                }
            }
            if t.load.is_none() {
                return Err(format!(
                    "target `{id}` is missing its [targets.{id}.load] block"
                ));
            }
            if t.kind == "postgres" && t.dsn.is_none() {
                return Err(format!("postgres target `{id}` is missing `dsn`"));
            }
        }
        for cell in &spec.cells {
            if !cell.targets.contains_key(&spec.base) {
                return Err(format!(
                    "cell `{}` does not include the base target `{}`",
                    cell.name, spec.base
                ));
            }
            for (tid, ct) in &cell.targets {
                let kind = spec.targets[tid].kind.as_str();
                let ok = match kind {
                    "postgres" => ct.query.is_some(),
                    _ => ct.request.is_some(),
                };
                if !ok {
                    let want = if kind == "postgres" {
                        "`query`"
                    } else {
                        "`request`"
                    };
                    return Err(format!(
                        "cell `{}` target `{tid}` (kind `{kind}`) is missing {want}",
                        cell.name
                    ));
                }
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/arena/src/spec.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/arena/src/spec.rs` captured during arena
      standardization onto the codegen ladder.
```
