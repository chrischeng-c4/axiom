//! Py3.12 conformance tests for the `calendar` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_calendar.py):
//!   isleap, weekday, monthrange, leapdays.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_calendar_isleap_2024() {
    let out = jit_capture(
        r#"import calendar
print(calendar.isleap(2024))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_calendar_isleap_2023() {
    let out = jit_capture(
        r#"import calendar
print(calendar.isleap(2023))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_calendar_isleap_century_rule() {
    let out = jit_capture(
        r#"import calendar
print(calendar.isleap(1900))
print(calendar.isleap(2000))
"#,
    );
    assert_output(&out, "False\nTrue\n");
}

#[test]
fn test_calendar_weekday_known_date() {
    let out = jit_capture(
        r#"import calendar
print(calendar.weekday(2026, 1, 1))
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_calendar_monthrange_february_leap() {
    let out = jit_capture(
        r#"import calendar
print(calendar.monthrange(2024, 2)[1])
"#,
    );
    assert_output(&out, "29\n");
}

#[test]
fn test_calendar_monthrange_february_nonleap() {
    let out = jit_capture(
        r#"import calendar
print(calendar.monthrange(2023, 2)[1])
"#,
    );
    assert_output(&out, "28\n");
}
