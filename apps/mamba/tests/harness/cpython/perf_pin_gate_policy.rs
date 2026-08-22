//! cpython — perf-pin **gate policy** regression tests (#3070).
//!
//! These tests do not measure anything. They pin the decision that says whether
//! an ungradable perf pin is allowed to report `ok`.
//!
//! ## Why this file exists
//!
//! #2011 clause 41 specifies the production perf gate as
//!
//! ```text
//! MAMBA_REQUIRE_CPYTHON_PERF_BASELINE=1 cargo test -p mamba --release --test perf_pin -- <pin>
//! ```
//!
//! and states that "cross-host skip, missing host, missing baseline, or JSONL
//! to SQLite mismatch is a hard failure, not a passing skip."
//!
//! Before #3070 that was not true. `baseline_required()` was consulted in
//! exactly one function — `load_cpython_baseline` — which covered only the
//! missing-DB and missing-row cases. The host-affinity check lived in a second
//! loader, `load_same_host_baseline`, and `perf_pin.rs`'s cross-host arm
//! returned `Ok(())` without ever asking whether grading had been demanded. On
//! a host whose committed baseline corpus is 102 rows with no host and 20 rows
//! from another machine, that meant all 128 pins reported `ok` while grading
//! zero of them — including pins that fail by 2.90x when actually graded.
//!
//! The gate being green is worthless if the gate cannot go red. That is what is
//! pinned here.
//!
//! ```text
//! cargo test -p mamba --test perf_pin_gate_policy
//! ```
//!
//! Unlike `perf_pin`/`perf_gate_report` this target does NOT require the
//! release profile — it exercises pure policy, not measurement.

use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

#[path = "harness_common.rs"]
mod common;
use common::{baseline_required, classify_baseline_host, require_gradable_baseline,
             NoBaselineReason};

const FLAG: &str = "MAMBA_REQUIRE_CPYTHON_PERF_BASELINE";

/// `baseline_required()` reads a process-global env var, so every test that
/// sets it must hold this lock. Tests in one binary share a process and run in
/// parallel by default.
fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    // A poisoned lock only means some earlier test panicked (several here panic
    // by design, via #[should_panic]); the guarded state is just an env var.
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Run `body` with the require flag set to `value` (or unset for `None`),
/// restoring whatever was there before — **including when `body` panics**,
/// which several tests below do by design. Without the unwind-safe restore, a
/// `#[should_panic]` test would leave `FLAG=1` set for every later test in the
/// same process.
fn with_flag<T>(value: Option<&str>, body: impl FnOnce() -> T) -> T {
    fn apply(value: Option<&str>) {
        match value {
            Some(v) => std::env::set_var(FLAG, v),
            None => std::env::remove_var(FLAG),
        }
    }

    let _guard = env_lock();
    let previous = std::env::var(FLAG).ok();
    apply(value);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
    apply(previous.as_deref());
    match result {
        Ok(out) => out,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

fn pin_path() -> &'static Path {
    Path::new("tests/harness/cpython/config/perf/pins/example_1234.toml")
}

// ---------------------------------------------------------------------------
// Host classification: no-host and different-named-host are distinct reasons.
// ---------------------------------------------------------------------------

#[test]
fn same_host_is_gradable() {
    assert_eq!(classify_baseline_host(Some("hostA"), Some("hostA")), None);
}

#[test]
fn different_named_host_is_cross_host() {
    assert_eq!(
        classify_baseline_host(Some("hostA"), Some("hostB")),
        Some(NoBaselineReason::CrossHost)
    );
}

#[test]
fn row_with_no_recorded_host_is_no_host_not_cross_host() {
    // 102 of the 122 committed rows are in this state (legacy pre-#966). It
    // must not be reported as "recorded on a different host" — the remedy is
    // to re-record, not to change machines.
    assert_eq!(
        classify_baseline_host(None, Some("hostA")),
        Some(NoBaselineReason::NoHost)
    );
}

#[test]
fn undetectable_local_hostname_is_no_host() {
    assert_eq!(
        classify_baseline_host(Some("hostA"), None),
        Some(NoBaselineReason::NoHost)
    );
    assert_eq!(classify_baseline_host(None, None), Some(NoBaselineReason::NoHost));
}

#[test]
fn reasons_have_distinct_stable_slugs() {
    assert_eq!(NoBaselineReason::Missing.as_str(), "missing");
    assert_eq!(NoBaselineReason::CrossHost.as_str(), "cross-host");
    assert_eq!(NoBaselineReason::NoHost.as_str(), "no-host");
}

// ---------------------------------------------------------------------------
// The flag itself.
// ---------------------------------------------------------------------------

#[test]
fn require_flag_parses_the_documented_spellings() {
    for truthy in ["1", "true", "TRUE", "yes", "required"] {
        assert!(
            with_flag(Some(truthy), baseline_required),
            "{truthy:?} should enable the require flag"
        );
    }
    for falsy in ["0", "no", "false", ""] {
        assert!(
            !with_flag(Some(falsy), baseline_required),
            "{falsy:?} should not enable the require flag"
        );
    }
    assert!(!with_flag(None, baseline_required), "unset means not required");
}

// ---------------------------------------------------------------------------
// The regression proper: ungradable + required == hard failure.
// ---------------------------------------------------------------------------

#[test]
fn cross_host_skip_is_allowed_when_grading_was_not_demanded() {
    with_flag(None, || {
        require_gradable_baseline(NoBaselineReason::CrossHost, 1234, "example", pin_path());
        require_gradable_baseline(NoBaselineReason::NoHost, 1234, "example", pin_path());
        require_gradable_baseline(NoBaselineReason::Missing, 1234, "example", pin_path());
    });
}

#[test]
fn explicitly_disabled_flag_still_allows_the_skip() {
    with_flag(Some("0"), || {
        require_gradable_baseline(NoBaselineReason::CrossHost, 1234, "example", pin_path());
    });
}

#[test]
#[should_panic(expected = "MAMBA_REQUIRE_CPYTHON_PERF_BASELINE is set")]
fn cross_host_is_a_hard_failure_when_grading_was_demanded() {
    with_flag(Some("1"), || {
        require_gradable_baseline(NoBaselineReason::CrossHost, 1234, "example", pin_path());
    });
}

#[test]
#[should_panic(expected = "MAMBA_REQUIRE_CPYTHON_PERF_BASELINE is set")]
fn no_host_is_a_hard_failure_when_grading_was_demanded() {
    // The case that actually applies to this repo's committed corpus.
    with_flag(Some("1"), || {
        require_gradable_baseline(NoBaselineReason::NoHost, 1234, "example", pin_path());
    });
}

#[test]
#[should_panic(expected = "MAMBA_REQUIRE_CPYTHON_PERF_BASELINE is set")]
fn missing_baseline_is_a_hard_failure_when_grading_was_demanded() {
    with_flag(Some("1"), || {
        require_gradable_baseline(NoBaselineReason::Missing, 1234, "example", pin_path());
    });
}

#[test]
fn failure_message_names_the_reason_and_the_remedy() {
    let panic = std::panic::catch_unwind(|| {
        with_flag(Some("1"), || {
            require_gradable_baseline(NoBaselineReason::NoHost, 1234, "example", pin_path());
        })
    })
    .expect_err("should have panicked");
    let msg = panic
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
        .expect("panic payload should be a string");

    // An operator reading CI output must learn: which pin, why, and what to do.
    assert!(msg.contains("#1234 example"), "should name the pin: {msg}");
    assert!(msg.contains("no-host"), "should name the reason slug: {msg}");
    assert!(
        msg.contains("no recorded host"),
        "should explain the reason: {msg}"
    );
    assert!(
        msg.contains("perf_baseline.py record"),
        "should name the remedy: {msg}"
    );
}
