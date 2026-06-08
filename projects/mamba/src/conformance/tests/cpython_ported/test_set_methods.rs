//! Py3.12 conformance tests for `set` mutating methods (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_set.py — mutation
//! sections):
//!   `add`, `remove`, `discard` (silent on missing), and `clear`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_set_add_and_remove() {
    let out = jit_capture(
        r#"s = {1, 2, 3}
s.add(4)
print(sorted(s))
s.remove(2)
print(sorted(s))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4]\n[1, 3, 4]\n");
}

#[test]
fn test_set_discard_is_silent_on_missing() {
    let out = jit_capture(
        r#"s = {1, 2, 3}
s.discard(99)
print(sorted(s))
s.discard(1)
print(sorted(s))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[2, 3]\n");
}

#[test]
fn test_set_clear_empties() {
    let out = jit_capture(
        r#"s = {1, 2, 3}
s.clear()
print(s)
print(len(s))
print(bool(s))
"#,
    );
    assert_output(&out, "set()\n0\nFalse\n");
}
