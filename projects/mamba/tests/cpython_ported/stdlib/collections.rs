//! Ported from Lib/test/test_collections_ported.py
//! Integration tests: stdlib/collections.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_collections_ported.py`.
#[test]
fn test_collections_namedtuple_attribute_access() {
    let out = jit_capture(
        r#"import collections
Point = collections.namedtuple("Point", ["x", "y"])
p = Point(3, 4)
print(p.x)
print(p.y)
"#,
    );
    assert_output(&out, "3\n4\n");
}

/// Ported from `Lib/test/test_collections_ported.py`.
#[test]
fn test_collections_namedtuple_repr() {
    let out = jit_capture(
        r#"import collections
Point = collections.namedtuple("Point", ["x", "y"])
p = Point(3, 4)
print(p)
"#,
    );
    assert_output(&out, "Point(x=3, y=4)\n");
}

/// Ported from `Lib/test/test_collections_ported.py`.
#[test]
fn test_collections_deque_append_and_appendleft() {
    let out = jit_capture(
        r#"import collections
d = collections.deque([1, 2, 3])
d.append(4)
d.appendleft(0)
print(list(d))
"#,
    );
    assert_output(&out, "[0, 1, 2, 3, 4]\n");
}

/// Ported from `Lib/test/test_collections_ported.py`.
#[test]
fn test_collections_deque_pop_and_popleft() {
    let out = jit_capture(
        r#"import collections
d = collections.deque([0, 1, 2, 3, 4])
d.pop()
d.popleft()
print(list(d))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// Ported from `Lib/test/test_collections_ported.py`.
#[test]
fn test_collections_counter_character_counts() {
    let out = jit_capture(
        r#"import collections
c = collections.Counter("aaabbc")
print(c["a"])
print(c["b"])
print(c["c"])
"#,
    );
    assert_output(&out, "3\n2\n1\n");
}

/// Ported from `Lib/test/test_collections_ported.py`.
#[test]
fn test_collections_counter_most_common() {
    let out = jit_capture(
        r#"import collections
c = collections.Counter("aaabbc")
print(c.most_common(2))
"#,
    );
    assert_output(&out, "[('a', 3), ('b', 2)]\n");
}

/// Ported from `Lib/test/test_collections_ported.py`.
#[test]
fn test_collections_defaultdict_list_factory() {
    let out = jit_capture(
        r#"import collections
dd = collections.defaultdict(list)
dd["x"].append(1)
dd["x"].append(2)
dd["y"].append(3)
print(dict(dd))
"#,
    );
    assert_output(&out, "{'x': [1, 2], 'y': [3]}\n");
}

/// Ported from `Lib/test/test_collections_ported.py`.
#[test]
fn test_collections_defaultdict_int_factory() {
    let out = jit_capture(
        r#"import collections
dd = collections.defaultdict(int)
dd["a"] += 1
dd["a"] += 1
dd["b"] += 5
print(dict(dd))
"#,
    );
    assert_output(&out, "{'a': 2, 'b': 5}\n");
}

/// Ported from `Lib/test/test_collections_ported.py`.
#[test]
fn test_collections_ordereddict_preserves_insertion_order() {
    let out = jit_capture(
        r#"import collections
od = collections.OrderedDict()
od["a"] = 1
od["b"] = 2
od["c"] = 3
print(list(od.keys()))
print(list(od.values()))
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n[1, 2, 3]\n");
}

/// Ported from `Lib/test/test_collections_ported.py`.
#[test]
fn test_collections_chainmap_lookup_precedence() {
    let out = jit_capture(
        r#"import collections
cm = collections.ChainMap({"a": 1}, {"b": 2, "a": 99})
print(cm["a"])
print(cm["b"])
"#,
    );
    assert_output(&out, "1\n2\n");
}

/// Ported from `Lib/test/test_collections_ported.py`.
#[test]
fn test_time_perf_counter_non_negative() {
    let out = jit_capture(
        r#"import time
p = time.perf_counter()
print(p >= 0)
"#,
    );
    assert_output(&out, "True\n");
}

