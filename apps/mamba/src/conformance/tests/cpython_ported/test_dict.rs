//! Py3.12 conformance tests for dict (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_dict.py — DictTest
//!
//! Coverage: basic dict construction, __len__, __getitem__, __setitem__,
//! __contains__, get(), keys()/values()/items() iteration, pop(), clear(),
//! update(), and equality semantics.
//!
//! Tests that require unimplemented features are marked `#[ignore]` with
//! a comment naming the missing behavior.
//!
//! @issue #759

use super::{assert_output, jit_capture};

// ── construction ──────────────────────────────────────────────────────────────

/// Empty literal dict has len 0.
#[test]
fn test_dict_empty_literal_len() {
    let out = jit_capture(
        r#"d = {}
print(len(d))
"#,
    );
    assert_output(&out, "0\n");
}

/// Non-empty literal dict has matching len.
#[test]
fn test_dict_literal_len() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
print(len(d))
"#,
    );
    assert_output(&out, "3\n");
}

/// dict() constructor with no args returns empty dict.
#[test]
fn test_dict_constructor_empty() {
    let out = jit_capture(
        r#"d = dict()
print(len(d))
"#,
    );
    assert_output(&out, "0\n");
}

// ── __getitem__ / __setitem__ ─────────────────────────────────────────────────

/// __getitem__ returns the value for the key.
#[test]
fn test_dict_getitem_returns_value() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
print(d["a"])
print(d["b"])
"#,
    );
    assert_output(&out, "1\n2\n");
}

/// __setitem__ inserts a new key.
#[test]
fn test_dict_setitem_inserts() {
    let out = jit_capture(
        r#"d = {"a": 1}
d["b"] = 2
print(len(d))
print(d["b"])
"#,
    );
    assert_output(&out, "2\n2\n");
}

/// __setitem__ overwrites an existing key (len unchanged).
#[test]
fn test_dict_setitem_overwrites() {
    let out = jit_capture(
        r#"d = {"a": 1}
d["a"] = 99
print(len(d))
print(d["a"])
"#,
    );
    assert_output(&out, "1\n99\n");
}

// ── __contains__ ──────────────────────────────────────────────────────────────

/// Key present membership returns True.
#[test]
fn test_dict_contains_present() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
print("a" in d)
"#,
    );
    assert_output(&out, "True\n");
}

/// Key absent membership returns False.
#[test]
fn test_dict_contains_absent() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
print("z" in d)
"#,
    );
    assert_output(&out, "False\n");
}

// ── get() ─────────────────────────────────────────────────────────────────────

/// get() of present key returns the value.
#[test]
fn test_dict_get_present_returns_value() {
    let out = jit_capture(
        r#"d = {"a": 1}
print(d.get("a"))
"#,
    );
    assert_output(&out, "1\n");
}

/// get() of missing key without default returns None.
#[test]
fn test_dict_get_missing_returns_none() {
    let out = jit_capture(
        r#"d = {"a": 1}
print(d.get("z"))
"#,
    );
    assert_output(&out, "None\n");
}

/// get() of missing key with default returns the default.
#[test]
fn test_dict_get_missing_with_default() {
    let out = jit_capture(
        r#"d = {"a": 1}
print(d.get("z", -1))
"#,
    );
    assert_output(&out, "-1\n");
}

// ── keys / values / items ────────────────────────────────────────────────────

/// keys() yields the dict's keys (order-independent count check).
#[test]
fn test_dict_keys_count() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
ks = list(d.keys())
print(len(ks))
print("a" in ks)
print("b" in ks)
print("c" in ks)
"#,
    );
    assert_output(&out, "3\nTrue\nTrue\nTrue\n");
}

/// values() yields the dict's values.
#[test]
fn test_dict_values_count() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
vs = list(d.values())
print(len(vs))
print(1 in vs)
print(2 in vs)
print(3 in vs)
"#,
    );
    assert_output(&out, "3\nTrue\nTrue\nTrue\n");
}

/// items() yields (key, value) tuples.
#[test]
fn test_dict_items_count() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
its = list(d.items())
print(len(its))
"#,
    );
    assert_output(&out, "2\n");
}

// ── pop / clear ───────────────────────────────────────────────────────────────

/// pop() removes and returns the value.
#[test]
fn test_dict_pop_present() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
v = d.pop("a")
print(v)
print(len(d))
print("a" in d)
"#,
    );
    assert_output(&out, "1\n1\nFalse\n");
}

/// pop() with default for missing key returns the default.
#[test]
fn test_dict_pop_missing_with_default() {
    let out = jit_capture(
        r#"d = {"a": 1}
v = d.pop("z", -1)
print(v)
print(len(d))
"#,
    );
    assert_output(&out, "-1\n1\n");
}

/// clear() empties the dict.
#[test]
fn test_dict_clear() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
d.clear()
print(len(d))
"#,
    );
    assert_output(&out, "0\n");
}

// ── update ────────────────────────────────────────────────────────────────────

/// update() with another dict merges keys.
#[test]
fn test_dict_update_with_dict() {
    let out = jit_capture(
        r#"d = {"a": 1}
d.update({"b": 2, "c": 3})
print(len(d))
print(d["a"])
print(d["b"])
print(d["c"])
"#,
    );
    assert_output(&out, "3\n1\n2\n3\n");
}

/// update() overrides existing keys.
#[test]
fn test_dict_update_overrides() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
d.update({"a": 99})
print(d["a"])
print(len(d))
"#,
    );
    assert_output(&out, "99\n2\n");
}

// ── equality ─────────────────────────────────────────────────────────────────

/// Two dicts with identical entries are equal.
#[test]
fn test_dict_equal_same_entries() {
    let out = jit_capture(
        r#"a = {"x": 1, "y": 2}
b = {"y": 2, "x": 1}
print(a == b)
"#,
    );
    assert_output(&out, "True\n");
}

/// Dicts with different values are not equal.
#[test]
fn test_dict_not_equal_different_values() {
    let out = jit_capture(
        r#"a = {"x": 1}
b = {"x": 2}
print(a == b)
"#,
    );
    assert_output(&out, "False\n");
}

/// Dicts with different keys are not equal.
#[test]
fn test_dict_not_equal_different_keys() {
    let out = jit_capture(
        r#"a = {"x": 1}
b = {"y": 1}
print(a == b)
"#,
    );
    assert_output(&out, "False\n");
}

// ── bool / truthiness ────────────────────────────────────────────────────────

/// Empty dict is falsy.
#[test]
fn test_dict_bool_empty_is_false() {
    let out = jit_capture(
        r#"d = {}
print(bool(d))
"#,
    );
    assert_output(&out, "False\n");
}

/// Non-empty dict is truthy.
#[test]
fn test_dict_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"d = {"a": 1}
print(bool(d))
"#,
    );
    assert_output(&out, "True\n");
}
