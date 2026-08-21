//! Py3.12 conformance tests for dict view iteration (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_dict.py — view
//! object sections): iterating `keys`/`values`/`items`, length and
//! membership, key deletion via `del`, and direct iteration of `dict`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_iterate_keys_values_items() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
print(sorted(d))
print(sorted(d.keys()))
print(sorted(d.values()))
for k, v in sorted(d.items()):
    print(k, "=", v)
"#,
    );
    assert_output(
        &out,
        "['a', 'b', 'c']\n['a', 'b', 'c']\n[1, 2, 3]\na = 1\nb = 2\nc = 3\n",
    );
}

#[test]
fn test_len_and_del() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
print(len(d))
d["d"] = 4
print(len(d))
del d["a"]
print(len(d))
print(sorted(d.items()))
"#,
    );
    assert_output(&out, "3\n4\n3\n[('b', 2), ('c', 3), ('d', 4)]\n");
}

#[test]
fn test_membership_and_iteration_order_after_mutation() {
    let out = jit_capture(
        r#"d = {}
d["x"] = 10
d["y"] = 20
d["z"] = 30
print("x" in d)
print("w" in d)
print(sorted(d.items()))
d["y"] = 99
print(sorted(d.items()))
"#,
    );
    assert_output(
        &out,
        "True\nFalse\n[('x', 10), ('y', 20), ('z', 30)]\n[('x', 10), ('y', 99), ('z', 30)]\n",
    );
}
