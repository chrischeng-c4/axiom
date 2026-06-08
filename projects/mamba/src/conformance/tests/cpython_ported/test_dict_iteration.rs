//! Py3.12 conformance tests for iterating over a `dict` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_dict.py — iteration
//! sections):
//!   `for k in d` yields keys, `.values()` yields values, `.items()`
//!   yields tuple pairs unpackable in `for k, v in ...`. Membership
//!   `in` and `len` are also exercised.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_dict_iteration_yields_keys() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
keys = []
for k in d:
    keys.append(k)
print(sorted(keys))
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n");
}

#[test]
fn test_dict_values_iteration_yields_values() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
vals = []
for v in d.values():
    vals.append(v)
print(sorted(vals))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

#[test]
fn test_dict_items_unpacking_in_for() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
pairs = []
for k, v in d.items():
    pairs.append((k, v))
print(sorted(pairs))
"#,
    );
    assert_output(&out, "[('a', 1), ('b', 2), ('c', 3)]\n");
}

#[test]
fn test_dict_membership_and_len() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
print("a" in d)
print("z" in d)
print(len(d))
"#,
    );
    assert_output(&out, "True\nFalse\n3\n");
}
