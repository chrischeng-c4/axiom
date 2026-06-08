//! Py3.12 conformance tests for shallow `.copy()` independence
//! across `list`, `dict`, and `set` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_list.py,
//! Lib/test/test_dict.py, and Lib/test/test_set.py — copy
//! sections): mutating a shallow copy must not affect the
//! original container.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_list_copy_independent_of_original() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = a.copy()
b.append(4)
print(a)
print(b)
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[1, 2, 3, 4]\n");
}

#[test]
fn test_dict_copy_independent_of_original() {
    let out = jit_capture(
        r#"d1 = {"x": 1, "y": 2}
d2 = d1.copy()
d2["z"] = 3
print(sorted(d1.items()))
print(sorted(d2.items()))
"#,
    );
    assert_output(
        &out,
        "[('x', 1), ('y', 2)]\n[('x', 1), ('y', 2), ('z', 3)]\n",
    );
}

#[test]
fn test_set_copy_independent_of_original() {
    let out = jit_capture(
        r#"s1 = {1, 2, 3}
s2 = s1.copy()
s2.add(4)
print(sorted(s1))
print(sorted(s2))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[1, 2, 3, 4]\n");
}
