//! Py3.12 conformance tests for the `gc` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_gc.py):
//!   gc.isenabled, gc.get_count shape, gc.collect smoke test.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_gc_isenabled_default() {
    let out = jit_capture(
        r#"import gc
print(gc.isenabled())
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_gc_get_count_three_generations() {
    let out = jit_capture(
        r#"import gc
counts = gc.get_count()
print(len(counts))
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_gc_collect_returns_non_negative() {
    let out = jit_capture(
        r#"import gc
n = gc.collect()
print(n >= 0)
"#,
    );
    assert_output(&out, "True\n");
}
