//! Py3.12 conformance tests for `str.partition` / `rpartition`
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py — partition
//! sections):
//!   left/right partition, behaviour when delimiter is absent.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_partition_splits_on_first_delim() {
    let out = jit_capture(
        r#"print("a,b,c".partition(","))
"#,
    );
    assert_output(&out, "('a', ',', 'b,c')\n");
}

#[test]
fn test_rpartition_splits_on_last_delim() {
    let out = jit_capture(
        r#"print("a,b,c".rpartition(","))
"#,
    );
    assert_output(&out, "('a,b', ',', 'c')\n");
}

#[test]
fn test_partition_no_delim_returns_original_plus_empties() {
    let out = jit_capture(
        r#"print("nodelim".partition(","))
"#,
    );
    assert_output(&out, "('nodelim', '', '')\n");
}
