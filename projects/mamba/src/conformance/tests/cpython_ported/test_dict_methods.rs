//! Py3.12 conformance tests for `dict` accessor and mutating methods
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_dict.py — method
//! sections):
//!   `get` with and without default, `setdefault`, `update`, and
//!   `keys`/`values`/`items` views (sorted for deterministic output).
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_dict_get_with_and_without_default() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
print(d.get("a"))
print(d.get("z"))
print(d.get("z", -1))
"#,
    );
    assert_output(&out, "1\nNone\n-1\n");
}

#[test]
fn test_dict_setdefault_only_inserts_if_missing() {
    let out = jit_capture(
        r#"d = {}
d.setdefault("k", 10)
d.setdefault("k", 20)
print(d)
"#,
    );
    assert_output(&out, "{'k': 10}\n");
}

#[test]
fn test_dict_update_merges_keys() {
    let out = jit_capture(
        r#"d = {"a": 1}
d.update({"b": 2, "c": 3})
print(sorted(d.items()))
"#,
    );
    assert_output(&out, "[('a', 1), ('b', 2), ('c', 3)]\n");
}

#[test]
fn test_dict_keys_values_items_views() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
print(sorted(d.keys()))
print(sorted(d.values()))
print(sorted(d.items()))
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n[1, 2, 3]\n[('a', 1), ('b', 2), ('c', 3)]\n");
}
