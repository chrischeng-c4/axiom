//! Py3.12 conformance tests for `dict.update` and the PEP 584 `|`
//! merge operator (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_dict.py — update
//! and union sections): in-place merge via `update`, multi-key
//! update, and non-mutating `|` union with later-wins precedence.
//!
//! @issue #759

use super::{assert_output, jit_capture};

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
