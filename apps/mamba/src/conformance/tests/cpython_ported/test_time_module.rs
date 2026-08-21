//! Py3.12 conformance tests for the `time` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_time.py):
//!   time(), monotonic(), perf_counter() — properties only (monotonic
//!   semantics, non-negative, ordering). Exact values are intentionally
//!   not asserted because wall-clock output is non-deterministic.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_time_returns_positive_epoch() {
    let out = jit_capture(
        r#"import time
t = time.time()
print(t > 1700000000)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_time_monotonic_non_decreasing() {
    let out = jit_capture(
        r#"import time
a = time.monotonic()
b = time.monotonic()
print(b >= a)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_time_perf_counter_non_negative() {
    let out = jit_capture(
        r#"import time
p = time.perf_counter()
print(p >= 0)
"#,
    );
    assert_output(&out, "True\n");
}
