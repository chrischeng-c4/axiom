//! Py3.12 conformance tests for `dict` lookup error handling
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_dict.py — access /
//! exception sections):
//!   missing-key raises `KeyError`, `in` checks membership without
//!   raising, `get` returns default, and post-mutation membership.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_missing_key_raises_keyerror() {
    let out = jit_capture(
        r#"d = {"a": 1}
try:
    print(d["z"])
except KeyError:
    print("missing")
"#,
    );
    assert_output(&out, "missing\n");
}

#[test]
fn test_in_check_does_not_raise() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
print("a" in d)
print("z" in d)
print("b" in d)
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\n");
}

#[test]
fn test_get_default_and_mutation_visible_in_membership() {
    let out = jit_capture(
        r#"d = {"a": 1}
print(d.get("a"))
print(d.get("z", "default"))
d["b"] = 2
print(sorted(d.items()))
print("b" in d)
"#,
    );
    assert_output(&out, "1\ndefault\n[('a', 1), ('b', 2)]\nTrue\n");
}
