//! Py3.12 conformance tests for the `copy` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_copy.py):
//!   copy.copy (shallow), copy.deepcopy (recursive) on lists and dicts.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_copy_shallow_list_independence() {
    let out = jit_capture(
        r#"import copy
a = [1, 2, 3]
b = copy.copy(a)
b.append(4)
print(a)
print(b)
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[1, 2, 3, 4]\n");
}

#[test]
fn test_copy_shallow_nested_shares_inner() {
    let out = jit_capture(
        r#"import copy
a = [1, [2, 3]]
b = copy.copy(a)
b[1].append(99)
print(a)
print(b)
"#,
    );
    assert_output(&out, "[1, [2, 3, 99]]\n[1, [2, 3, 99]]\n");
}

#[test]
fn test_copy_deepcopy_isolates_nested_list() {
    let out = jit_capture(
        r#"import copy
a = [1, [2, 3]]
b = copy.deepcopy(a)
b[1].append(99)
print(a)
print(b)
"#,
    );
    assert_output(&out, "[1, [2, 3]]\n[1, [2, 3, 99]]\n");
}

#[test]
fn test_copy_deepcopy_isolates_nested_dict() {
    let out = jit_capture(
        r#"import copy
d = {"a": [1, 2], "b": {"c": 3}}
e = copy.deepcopy(d)
e["a"].append(99)
e["b"]["c"] = 999
print(d["a"])
print(d["b"]["c"])
print(e["a"])
print(e["b"]["c"])
"#,
    );
    assert_output(&out, "[1, 2]\n3\n[1, 2, 99]\n999\n");
}

#[test]
fn test_copy_copy_dict_top_level_independence() {
    let out = jit_capture(
        r#"import copy
d = {"a": 1, "b": 2}
e = copy.copy(d)
e["c"] = 3
print("c" in d)
print("c" in e)
"#,
    );
    assert_output(&out, "False\nTrue\n");
}
