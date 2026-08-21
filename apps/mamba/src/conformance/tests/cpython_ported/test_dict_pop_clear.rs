//! Py3.12 conformance tests for `dict.pop` and `dict.clear` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_dict.py — mutation
//! sections):
//!   `pop` returns and removes; `pop` with default returns default on
//!   miss; `clear` empties the dict.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_dict_pop_returns_and_removes() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
v = d.pop("a")
print(v)
print(sorted(d.items()))
"#,
    );
    assert_output(&out, "1\n[('b', 2), ('c', 3)]\n");
}

#[test]
fn test_dict_pop_with_default_on_miss() {
    let out = jit_capture(
        r#"d = {"a": 1}
print(d.pop("z", -1))
print(d.pop("z", "missing"))
print(sorted(d.items()))
"#,
    );
    assert_output(&out, "-1\nmissing\n[('a', 1)]\n");
}

#[test]
fn test_dict_clear_empties_dict() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
d.clear()
print(d)
print(len(d))
print(bool(d))
"#,
    );
    assert_output(&out, "{}\n0\nFalse\n");
}
