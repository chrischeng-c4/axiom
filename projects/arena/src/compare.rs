//! Ratio classification — a direct port of lumen's `judge()`
//! (projects/lumen/tests/perf_gate_vs_db.rs). `ratio = peer / base` over a
//! lower-is-better metric, so `ratio > 1` means the base target wins.

/// The outcome of classifying one (cell, peer) ratio against its gate.
#[derive(Debug, Clone, PartialEq)]
pub enum Classification {
    /// `gate = "win"`: the base must beat the peer by `req = max(1, ratchet*baseline)`.
    Win { req: f64, ok: bool },
    /// `gate = "target"`: aspirational floor; red below it but never gates.
    Target { floor: f64, ok: bool },
    /// `gate = "exempt"`: reported, never compared for pass/fail.
    Exempt,
}

impl Classification {
    /// A short human verdict label for the comparison table.
    pub fn label(&self) -> String {
        match self {
            Classification::Win { req, ok: true } => format!("WIN ok>={req:.1}"),
            Classification::Win { req, ok: false } => format!("WIN<{req:.1}"),
            Classification::Target { ok: true, .. } => "TARGET ok".to_string(),
            Classification::Target { ok: false, .. } => "TARGET red".to_string(),
            Classification::Exempt => "exempt".to_string(),
        }
    }

    /// A WIN cell that breached its ratcheted requirement — the only
    /// build-failing outcome (maps to a `PinRegression` finding → exit 2).
    pub fn is_win_breach(&self) -> bool {
        matches!(self, Classification::Win { ok: false, .. })
    }

    /// A TARGET cell below its floor — reported red but never gates.
    pub fn is_target_red(&self) -> bool {
        matches!(self, Classification::Target { ok: false, .. })
    }
}

/// Classify one peer's `ratio` for a cell. `baseline` is the recorded ratcheted
/// ratio (None ⇒ first run, treated as floor 1.0); `floor` applies to `target`.
pub fn classify(
    gate: &str,
    ratio: f64,
    ratchet: f64,
    baseline: Option<f64>,
    floor: Option<f64>,
) -> Classification {
    match gate {
        "win" => {
            let base = baseline.unwrap_or(1.0);
            let req = (ratchet * base).max(1.0);
            Classification::Win {
                req,
                ok: ratio + 1e-9 >= req,
            }
        }
        "target" => {
            let floor = floor.unwrap_or(1.0);
            Classification::Target {
                floor,
                ok: ratio + 1e-9 >= floor,
            }
        }
        _ => Classification::Exempt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win_passes_at_or_above_ratcheted_requirement() {
        // baseline 10, ratchet 0.8 -> req 8.0; ratio 9 passes, 7 breaches.
        assert!(matches!(
            classify("win", 9.0, 0.8, Some(10.0), None),
            Classification::Win { ok: true, .. }
        ));
        let breach = classify("win", 7.0, 0.8, Some(10.0), None);
        assert!(breach.is_win_breach());
        assert_eq!(breach.label(), "WIN<8.0");
    }

    #[test]
    fn win_floor_is_one_when_no_baseline() {
        // req = max(1, 0.8*1) = 1.0; any ratio >= 1 passes the first run.
        assert!(matches!(
            classify("win", 1.0, 0.8, None, None),
            Classification::Win { ok: true, req, .. } if (req - 1.0).abs() < 1e-9
        ));
    }

    #[test]
    fn target_is_red_below_floor_but_not_a_breach() {
        let red = classify("target", 0.4, 0.8, None, Some(0.5));
        assert!(red.is_target_red());
        assert!(!red.is_win_breach());
    }

    #[test]
    fn exempt_never_judges() {
        assert_eq!(
            classify("exempt", 0.01, 0.8, None, None),
            Classification::Exempt
        );
    }
}
