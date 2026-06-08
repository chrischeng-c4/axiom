// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
// CODEGEN-BEGIN
//! Gate logic — fold a manifest + results + waivers into a `GateReport`.

use chrono::{DateTime, Utc};

use crate::manifest::GatingManifest;
use crate::result::{ChannelResult, Status};
use crate::waivers::Waivers;

/// Exit-code contract — keep aligned with `docs/gating-manifest.md`.
pub const EXIT_PASS: i32 = 0;
pub const EXIT_BLOCKING_FAIL: i32 = 1;
pub const EXIT_SOFT_FAIL: i32 = 2;
pub const EXIT_SKIPPED: i32 = 77;

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Clone)]
pub struct GateReport {
    pub total: usize,
    pub pass: usize,
    pub fail: usize,
    pub waived: usize,
    pub skipped: usize,
    pub exit_code: i32,
    pub blocking: bool,
    pub notes: Vec<String>,
}

/// Run the gate. Pure function — no I/O, no clock; the caller passes
/// `now` so tests are deterministic.
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
pub fn run_gate(
    manifest: &GatingManifest,
    results: &[ChannelResult],
    waivers: &Waivers,
    now: DateTime<Utc>,
) -> GateReport {
    if results.is_empty() {
        return GateReport {
            total: 0,
            pass: 0,
            fail: 0,
            waived: 0,
            skipped: 0,
            exit_code: EXIT_SKIPPED,
            blocking: manifest.blocking,
            notes: vec!["no channel-result.json files found".to_string()],
        };
    }

    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut waived = 0usize;
    let mut skipped = 0usize;
    let mut notes: Vec<String> = Vec::new();

    for r in results {
        match r.status {
            Status::Pass => pass += 1,
            Status::Skipped => skipped += 1,
            Status::Waived => waived += 1,
            Status::Fail => {
                let covered = manifest.allow_waivers
                    && waivers.applies_to(&r.fixture_id, &r.channel, now).is_some();
                if covered {
                    waived += 1;
                    notes.push(format!(
                        "waived: {fixture}/{channel}",
                        fixture = r.fixture_id,
                        channel = r.channel
                    ));
                } else {
                    fail += 1;
                    notes.push(format!(
                        "fail: {fixture}/{channel} (diff={value})",
                        fixture = r.fixture_id,
                        channel = r.channel,
                        value = r.diff_value
                    ));
                }
            }
        }
    }

    let exit_code = if fail == 0 {
        EXIT_PASS
    } else if manifest.blocking {
        EXIT_BLOCKING_FAIL
    } else {
        EXIT_SOFT_FAIL
    };

    GateReport {
        total: results.len(),
        pass,
        fail,
        waived,
        skipped,
        exit_code,
        blocking: manifest.blocking,
        notes,
    }
}
// CODEGEN-END
