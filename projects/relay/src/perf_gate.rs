// SPEC-MANAGED: projects/relay/tech-design/logic/competitor-perf-gate-vs-nats-rabbitmq-redpanda-arena-ratchet.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:d33f5c3d" tracker="pending-tracker" reason="The ratchet gate rule: evaluate per-cell ratios against the recorded baseline (no-regression) plus must-beat, returning a pass/fail verdict."
//! Competitor perf-gate ratchet rule (#125).
//!
//! arena measures each cell across N targets and reduces it to a single
//! `ratio` per cell, normalized so **higher is always better for relay**
//! (latency cells use `peer/relay`, throughput cells use `relay/peer`; either
//! way `> 1.0` means relay wins). This module is the pure decision the gate
//! makes from those ratios: a no-regression ratchet against the recorded
//! baseline plus a must-beat check on the cells where relay claims to win.

/// One cell's measured standing for relay against its competitors.
///
/// @spec projects/relay/tech-design/logic/competitor-perf-gate-vs-nats-rabbitmq-redpanda-arena-ratchet.md#logic
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    /// Cell name (e.g. `broadcast`, `work_queue`, `durable_log`).
    pub name: String,
    /// Current ratio, normalized so higher is better for relay.
    pub ratio: f64,
    /// The recorded baseline ratio from the last passing run.
    pub baseline_ratio: f64,
    /// Whether relay claims to beat the primary bar on this cell.
    pub must_beat: bool,
}

/// The gate decision over all cells.
///
/// @spec projects/relay/tech-design/logic/competitor-perf-gate-vs-nats-rabbitmq-redpanda-arena-ratchet.md#logic
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub passed: bool,
    /// Cells that regressed below `baseline_ratio * ratchet`.
    pub regressions: Vec<String>,
    /// must-beat cells where relay is no longer winning (`ratio < 1.0`).
    pub must_beat_losses: Vec<String>,
}

/// Evaluate the ratchet gate. A cell fails when it regressed below
/// `baseline_ratio * ratchet`, or when it is a must-beat cell and relay is no
/// longer ahead (`ratio < 1.0`). The gate passes only when no cell fails.
///
/// @spec projects/relay/tech-design/logic/competitor-perf-gate-vs-nats-rabbitmq-redpanda-arena-ratchet.md#logic
pub fn evaluate(cells: &[Cell], ratchet: f64) -> Verdict {
    let mut regressions = Vec::new();
    let mut must_beat_losses = Vec::new();
    for c in cells {
        if c.ratio < c.baseline_ratio * ratchet {
            regressions.push(c.name.clone());
        }
        if c.must_beat && c.ratio < 1.0 {
            must_beat_losses.push(c.name.clone());
        }
    }
    Verdict {
        passed: regressions.is_empty() && must_beat_losses.is_empty(),
        regressions,
        must_beat_losses,
    }
}
// HANDWRITE-END
