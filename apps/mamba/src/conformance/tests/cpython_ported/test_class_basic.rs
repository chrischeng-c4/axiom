//! Py3.12 conformance tests for basic user-defined classes
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_class.py — instance
//! and `__init__`/`__repr__` sections): `__init__` populating
//! attributes, `__repr__` rendering, instance method dispatch, and
//! method-to-method call within an instance.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_init_and_repr() {
    let out = jit_capture(
        r#"class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __repr__(self):
        return "Point(" + str(self.x) + ", " + str(self.y) + ")"

p = Point(3, 4)
q = Point(0, 0)
print(p)
print(q)
print(p.x, p.y)
"#,
    );
    assert_output(&out, "Point(3, 4)\nPoint(0, 0)\n3 4\n");
}

#[test]
fn test_instance_method_dispatch() {
    let out = jit_capture(
        r#"class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def distance_to(self, other):
        dx = self.x - other.x
        dy = self.y - other.y
        return (dx * dx + dy * dy) ** 0.5

p = Point(3, 4)
q = Point(0, 0)
print(p.distance_to(q))
print(q.distance_to(p))
"#,
    );
    assert_output(&out, "5.0\n5.0\n");
}

#[test]
fn test_method_calls_other_method() {
    let out = jit_capture(
        r#"class Box:
    def __init__(self, w, h):
        self.w = w
        self.h = h
    def area(self):
        return self.w * self.h
    def describe(self):
        return "Box(area=" + str(self.area()) + ")"

b = Box(3, 4)
print(b.area())
print(b.describe())
"#,
    );
    assert_output(&out, "12\nBox(area=12)\n");
}
