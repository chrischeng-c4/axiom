//! Ported from Lib/test/test_datetime_ported.py
//! Integration tests: stdlib/datetime.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_datetime_ported.py`.
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

/// Ported from `Lib/test/test_datetime_ported.py`.
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

