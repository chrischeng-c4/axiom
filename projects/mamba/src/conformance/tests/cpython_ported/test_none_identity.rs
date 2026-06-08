//! Py3.12 conformance tests for `None` and identity comparison
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_compare.py —
//! identity sections): `is None` / `is not None`, `None` in a list,
//! filtering `None` out of a list, and counting `None` occurrences.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_is_none_and_is_not_none() {
    let out = jit_capture(
        r#"x = None
y = "abc"
print(x is None)
print(y is None)
print(x is not None)
print(y is not None)
print(None is None)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\nTrue\nTrue\n");
}

#[test]
fn test_filter_none_from_list() {
    let out = jit_capture(
        r#"values = [None, 1, None, 2, None]
non_none = [v for v in values if v is not None]
print(non_none)
just_none = [v for v in values if v is None]
print(len(just_none))
"#,
    );
    assert_output(&out, "[1, 2]\n3\n");
}

#[test]
fn test_count_none_via_loop() {
    let out = jit_capture(
        r#"vs = [None, "a", None, "b", None, "c"]
count = 0
present = 0
for v in vs:
    if v is None:
        count = count + 1
    else:
        present = present + 1
print(count)
print(present)
"#,
    );
    assert_output(&out, "3\n3\n");
}
