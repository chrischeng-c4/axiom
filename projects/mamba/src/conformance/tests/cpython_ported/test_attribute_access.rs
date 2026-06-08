//! Py3.12 conformance tests for `getattr`/`setattr`/`hasattr`
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_getattr.py — built-in
//! attribute access sections):
//!   `getattr` with and without default, `hasattr` present/absent,
//!   `setattr` mutation visible on subsequent access.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_getattr_with_and_without_default() {
    let out = jit_capture(
        r#"class Box:
    pass
b = Box()
b.x = 10
print(getattr(b, "x"))
print(getattr(b, "y", 99))
print(getattr(b, "z", "missing"))
"#,
    );
    assert_output(&out, "10\n99\nmissing\n");
}

#[test]
fn test_hasattr_present_and_absent() {
    let out = jit_capture(
        r#"class Box:
    pass
b = Box()
b.x = 1
print(hasattr(b, "x"))
print(hasattr(b, "z"))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_setattr_mutation_visible_via_attribute_and_getattr() {
    let out = jit_capture(
        r#"class Box:
    pass
b = Box()
setattr(b, "y", 20)
print(b.y)
print(getattr(b, "y"))
setattr(b, "y", 30)
print(b.y)
"#,
    );
    assert_output(&out, "20\n20\n30\n");
}
