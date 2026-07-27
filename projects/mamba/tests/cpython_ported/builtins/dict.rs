//! Ported from Lib/test/test_dict_ported.py
//! Integration tests: builtins/dict.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_keys_values_items_views() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
print(sorted(d.keys()))
print(sorted(d.values()))
print(sorted(d.items()))
"#,
    );
    assert_output(
        &out,
        "['a', 'b', 'c']\n[1, 2, 3]\n[('a', 1), ('b', 2), ('c', 3)]\n",
    );
}

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_iter_dict_keys_for_loop() {
    let out = jit_capture(
        r#"d = {1: "a", 2: "b", 3: "c"}
total = 0
for k in d:
    total = total + k
print(total)
"#,
    );
    assert_output(&out, "6\n");
}

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_iter_dict_values_for_loop() {
    let out = jit_capture(
        r#"d = {"a": 10, "b": 20, "c": 30}
total = 0
for v in d.values():
    total = total + v
print(total)
"#,
    );
    assert_output(&out, "60\n");
}

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_iteration_size_mutation_raises_runtime_error() {
    let out = jit_capture(
        r#"d = {1: 1, 2: 2, 3: 3}
try:
    for k in d:
        d[k + 100] = 0
    print("no_raise")
except RuntimeError:
    print("RuntimeError")
"#,
    );
    assert_output(&out, "RuntimeError\n");
}

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_update_overwrites_and_adds() {
    let out = jit_capture(
        r#"a = {"x": 1, "y": 2}
b = {"y": 20, "z": 30}
a.update(b)
print(sorted(a.items()))
"#,
    );
    assert_output(&out, "[('x', 1), ('y', 20), ('z', 30)]\n");
}

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_update_with_literal() {
    let out = jit_capture(
        r#"c = {"a": 1}
c.update({"b": 2, "c": 3})
print(sorted(c.items()))
"#,
    );
    assert_output(&out, "[('a', 1), ('b', 2), ('c', 3)]\n");
}

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_union_pipe_operator() {
    let out = jit_capture(
        r#"m1 = {1: "a", 2: "b"}
m2 = {2: "B", 3: "C"}
merged = m1 | m2
print(sorted(merged.items()))
print(sorted(m1.items()))
"#,
    );
    assert_output(
        &out,
        "[(1, 'a'), (2, 'B'), (3, 'C')]\n[(1, 'a'), (2, 'b')]\n",
    );
}

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_char_histogram_via_in_guard() {
    let out = jit_capture(
        r#"d = {}
for c in "hello world":
    if c in d:
        d[c] = d[c] + 1
    else:
        d[c] = 1
print(sorted(d.items()))
"#,
    );
    assert_output(
        &out,
        "[(' ', 1), ('d', 1), ('e', 1), ('h', 1), ('l', 3), ('o', 2), ('r', 1), ('w', 1)]\n",
    );
}

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_word_histogram_via_get_default() {
    let out = jit_capture(
        r#"words = ["apple", "bee", "cat", "apple", "bee", "apple"]
counts = {}
for w in words:
    counts[w] = counts.get(w, 0) + 1
print(sorted(counts.items()))
"#,
    );
    assert_output(&out, "[('apple', 3), ('bee', 2), ('cat', 1)]\n");
}

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_counter_largest_via_max_over_keys() {
    let out = jit_capture(
        r#"counts = {"a": 3, "b": 1, "c": 5, "d": 2}
items = sorted(counts.items(), key=lambda kv: -kv[1])
print(items[0])
print(items[-1])
"#,
    );
    assert_output(&out, "('c', 5)\n('b', 1)\n");
}

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_comprehension_from_pair_list() {
    let out = jit_capture(
        r#"print({k: v for k, v in [("a", 1), ("b", 2)]})
"#,
    );
    assert_output(&out, "{'a': 1, 'b': 2}\n");
}

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_comprehension_transforming_range() {
    let out = jit_capture(
        r#"d = {x: x*x for x in range(4)}
print(sorted(d.items()))
"#,
    );
    assert_output(&out, "[(0, 0), (1, 1), (2, 4), (3, 9)]\n");
}

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_comprehension_with_filter() {
    let out = jit_capture(
        r#"d = {x: x for x in range(5) if x > 1}
print(sorted(d.items()))
"#,
    );
    assert_output(&out, "[(2, 2), (3, 3), (4, 4)]\n");
}

/// Empty literal dict has len 0.
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_constructor_empty() {
    let out = jit_capture(
        r#"d = dict()
print(len(d))
"#,
    );
    assert_output(&out, "0\n");
}

/// __getitem__ returns the value for the key.
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Key present membership returns True.
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_contains_absent() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
print("z" in d)
"#,
    );
    assert_output(&out, "False\n");
}

/// get() of present key returns the value.
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_get_missing_with_default() {
    let out = jit_capture(
        r#"d = {"a": 1}
print(d.get("z", -1))
"#,
    );
    assert_output(&out, "-1\n");
}

/// keys() yields the dict's keys (order-independent count check).
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
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

/// pop() removes and returns the value.
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
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

/// update() with another dict merges keys.
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Two dicts with identical entries are equal.
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Empty dict is falsy.
/// Ported from `Lib/test/test_dict_ported.py`.
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
/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_dict_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"d = {"a": 1}
print(bool(d))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
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

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_group_words_by_first_letter() {
    let out = jit_capture(
        r#"words = ["apple", "ant", "banana", "berry", "cherry", "almond"]
groups = {}
for w in words:
    k = w[0]
    if k not in groups:
        groups[k] = []
    groups[k].append(w)
for k in sorted(groups.keys()):
    print(k, groups[k])
"#,
    );
    assert_output(
        &out,
        "a ['apple', 'ant', 'almond']\nb ['banana', 'berry']\nc ['cherry']\n",
    );
}

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_char_count_via_dict_get_default() {
    let out = jit_capture(
        r#"counts = {}
for ch in "mississippi":
    counts[ch] = counts.get(ch, 0) + 1
for k in sorted(counts.keys()):
    print(k, counts[k])
"#,
    );
    assert_output(&out, "i 4\nm 1\np 2\ns 4\n");
}

/// Ported from `Lib/test/test_dict_ported.py`.
#[test]
fn test_sorted_keys_after_grouping() {
    let out = jit_capture(
        r#"items = [("z", 1), ("a", 2), ("m", 3), ("a", 4)]
agg = {}
for k, v in items:
    if k not in agg:
        agg[k] = 0
    agg[k] = agg[k] + v
for k in sorted(agg.keys()):
    print(k, agg[k])
"#,
    );
    assert_output(&out, "a 6\nm 3\nz 1\n");
}

