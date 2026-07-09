// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Clock/time control primitive for E2E cases (#2879).
//!
//! Some product flows are time-sensitive (countdown banners, session
//! expiries, debounced "saved" toasts). Reproducing those flows
//! reliably needs a controlled clock that the case can freeze, set,
//! and advance deterministically. This module owns the policy and
//! the evidence record; the actual page-level injection is performed
//! by the browser driver — see [`ClockControl::to_init_script`] for
//! the JS shim payload the runner injects via CDP.
//!
//! Scheduling virtualisation beyond `Date.now()` + `performance.now()`
//! is out of scope (split into a later issue).

use serde::{Deserialize, Serialize};

/// Stable schema tag for [`ClockEvidence`].
pub const CLOCK_SCHEMA_VERSION: &str = "jet.e2e.clock.v1";

/// Initial clock mode for a case. `Live` leaves the page clock alone;
/// `Frozen` pins both `Date.now()` and `performance.now()` to the
/// configured value until `advance` is called.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClockMode {
    Live,
    Frozen,
}

/// Per-case controlled clock. `now_ms` is the wall epoch the page
/// observes; `monotonic_ms` is the offset `performance.now()`
/// observes. Both advance with [`ClockControl::advance`] so the page
/// sees a consistent virtual time.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockControl {
    pub mode: ClockMode,
    pub now_ms: u64,
    pub monotonic_ms: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<ClockEvent>,
}

/// One transition applied to the clock. Recorded into evidence so an
/// inspector can replay the case's time without inferring it from
/// behaviour.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClockEvent {
    Freeze { now_ms: u64 },
    SetTime { now_ms: u64 },
    Advance { delta_ms: u64, now_ms: u64 },
    Release,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl ClockControl {
    /// Build a frozen clock at the supplied wall-clock epoch (ms).
    /// `performance.now()` starts at 0 so monotonic-now diffs remain
    /// stable.
    pub fn frozen_at(now_ms: u64) -> Self {
        Self {
            mode: ClockMode::Frozen,
            now_ms,
            monotonic_ms: 0,
            history: vec![ClockEvent::Freeze { now_ms }],
        }
    }

    /// Build a live (uncontrolled) clock — useful when only a subset
    /// of cases need a frozen clock.
    pub fn live() -> Self {
        Self {
            mode: ClockMode::Live,
            now_ms: 0,
            monotonic_ms: 0,
            history: Vec::new(),
        }
    }

    /// Jump the virtual clock by `delta_ms`. Updates both wall and
    /// monotonic clocks so the page never observes them desync.
    pub fn advance(&mut self, delta_ms: u64) {
        self.now_ms = self.now_ms.saturating_add(delta_ms);
        self.monotonic_ms = self.monotonic_ms.saturating_add(delta_ms);
        self.history.push(ClockEvent::Advance {
            delta_ms,
            now_ms: self.now_ms,
        });
    }

    /// Re-anchor the wall clock to a specific epoch without advancing
    /// the monotonic clock — equivalent to "the user changed system
    /// time" inside the test environment.
    pub fn set_time(&mut self, now_ms: u64) {
        self.now_ms = now_ms;
        self.history.push(ClockEvent::SetTime { now_ms });
    }

    /// Release the clock control back to the real page clock.
    pub fn release(&mut self) {
        self.mode = ClockMode::Live;
        self.history.push(ClockEvent::Release);
    }

    /// JS init-script payload the CDP driver runs at document start.
    /// Returns `None` when the clock is live (nothing to inject).
    pub fn to_init_script(&self) -> Option<String> {
        if self.mode == ClockMode::Live {
            return None;
        }
        Some(format!(
            "(()=>{{const NOW={};const PERF={};const D=Date;\
             function Frozen(){{return new D(NOW);}}\
             Frozen.now=()=>NOW;Frozen.UTC=D.UTC;Frozen.parse=D.parse;\
             globalThis.Date=Frozen;\
             const p=globalThis.performance;\
             if(p){{p.now=()=>PERF;}}}})();",
            self.now_ms, self.monotonic_ms,
        ))
    }
}

/// Evidence wrapper that pairs the clock state with the schema tag.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockEvidence {
    pub schema_version: String,
    pub control: ClockControl,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl ClockEvidence {
    pub fn from_control(control: ClockControl) -> Self {
        Self {
            schema_version: CLOCK_SCHEMA_VERSION.to_string(),
            control,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Render a fake "today's date" string using whatever epoch the
    /// supplied clock currently reports. Stands in for a real product
    /// component that reads `Date.now()` during a step.
    fn render_today(c: &ClockControl) -> String {
        format!("epoch:{}", c.now_ms)
    }

    #[test]
    fn frozen_clock_produces_deterministic_output() {
        // Stop condition (#2879): a case can freeze time
        // deterministically.
        let clock = ClockControl::frozen_at(1_700_000_000_000);
        let first = render_today(&clock);
        let second = render_today(&clock);
        assert_eq!(first, "epoch:1700000000000");
        assert_eq!(first, second);
    }

    #[test]
    fn advance_moves_both_wall_and_monotonic_clocks() {
        let mut clock = ClockControl::frozen_at(1_000);
        clock.advance(500);
        assert_eq!(clock.now_ms, 1_500);
        assert_eq!(clock.monotonic_ms, 500);
        clock.advance(250);
        assert_eq!(clock.now_ms, 1_750);
        assert_eq!(clock.monotonic_ms, 750);
    }

    #[test]
    fn set_time_does_not_advance_monotonic_clock() {
        let mut clock = ClockControl::frozen_at(1_000);
        clock.set_time(50_000);
        assert_eq!(clock.now_ms, 50_000);
        assert_eq!(clock.monotonic_ms, 0);
    }

    #[test]
    fn release_returns_to_live_mode() {
        let mut clock = ClockControl::frozen_at(1_000);
        clock.release();
        assert_eq!(clock.mode, ClockMode::Live);
        assert!(clock.to_init_script().is_none());
    }

    #[test]
    fn evidence_records_history_so_inspector_can_replay() {
        // Stop condition (#2879): evidence records the configured time
        // state.
        let mut clock = ClockControl::frozen_at(1_000);
        clock.advance(500);
        clock.set_time(99_999);
        clock.release();
        let ev = ClockEvidence::from_control(clock);
        let kinds: Vec<&str> = ev
            .control
            .history
            .iter()
            .map(|e| match e {
                ClockEvent::Freeze { .. } => "freeze",
                ClockEvent::SetTime { .. } => "set_time",
                ClockEvent::Advance { .. } => "advance",
                ClockEvent::Release => "release",
            })
            .collect();
        assert_eq!(kinds, vec!["freeze", "advance", "set_time", "release"]);
    }

    #[test]
    fn frozen_clock_emits_init_script_with_now_and_perf_values() {
        let clock = ClockControl::frozen_at(42_000);
        let script = clock.to_init_script().unwrap();
        assert!(script.contains("NOW=42000"), "{script}");
        assert!(script.contains("PERF=0"), "{script}");
        assert!(script.contains("globalThis.Date=Frozen"), "{script}");
    }

    #[test]
    fn evidence_round_trips_through_json() {
        let mut clock = ClockControl::frozen_at(1_000);
        clock.advance(10);
        let ev = ClockEvidence::from_control(clock);
        let json = serde_json::to_string(&ev).unwrap();
        let back: ClockEvidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ev);
        assert!(json.contains("\"mode\":\"frozen\""), "{json}");
    }
}
// CODEGEN-END
