//! Py3.12 conformance tests for `@property` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_descr.py — property
//! section):
//!   read-only property derived from attribute, property exposing
//!   computed value across instances.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_property_doubles_underscore_attr() {
    let out = jit_capture(
        r#"class C:
    def __init__(self, x):
        self._x = x
    @property
    def x(self):
        return self._x * 2

c = C(5)
print(c.x)
"#,
    );
    assert_output(&out, "10\n");
}

#[test]
fn test_property_distinct_per_instance() {
    let out = jit_capture(
        r#"class Square:
    def __init__(self, side):
        self.side = side
    @property
    def area(self):
        return self.side * self.side

print(Square(3).area)
print(Square(5).area)
print(Square(10).area)
"#,
    );
    assert_output(&out, "9\n25\n100\n");
}
