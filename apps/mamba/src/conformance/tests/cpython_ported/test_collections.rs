//! Py3.12 conformance tests for the `collections` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_collections.py):
//!   namedtuple (attribute access, repr), deque (append/appendleft/pop/popleft),
//!   Counter (counting, most_common), defaultdict (factory), OrderedDict
//!   (insertion order), ChainMap (lookup precedence).
//!
//! namedtuple positional indexing (`p[0]`) is intentionally excluded —
//! currently returns None under mamba; deferred as a separate runtime gap.
//!
//! @issue #759

use super::{assert_output, jit_capture};

// ---------------------------------------------------------------- namedtuple

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

// ---------------------------------------------------------------- deque

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

// ---------------------------------------------------------------- Counter

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

// ---------------------------------------------------------------- defaultdict

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

// ---------------------------------------------------------------- OrderedDict

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

// ---------------------------------------------------------------- ChainMap

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
