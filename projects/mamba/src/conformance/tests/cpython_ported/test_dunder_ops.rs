//! Py3.12 conformance tests for arithmetic and equality dunders on
//! user-defined classes (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_descr.py — operator
//! overloading sections):
//!   `__add__`, `__eq__`, `__len__` invoked by `+`, `==`, and `len`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_dunder_add_returns_new_instance() {
    let out = jit_capture(
        r#"class Vec:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __add__(self, other):
        return Vec(self.x + other.x, self.y + other.y)
    def __repr__(self):
        return f"Vec({self.x}, {self.y})"

a = Vec(1, 2)
b = Vec(3, 4)
print(a + b)
"#,
    );
    assert_output(&out, "Vec(4, 6)\n");
}

#[test]
fn test_dunder_eq_compares_by_field() {
    let out = jit_capture(
        r#"class Vec:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __eq__(self, other):
        return self.x == other.x and self.y == other.y

print(Vec(1, 2) == Vec(1, 2))
print(Vec(1, 2) == Vec(3, 4))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_dunder_len_drives_builtin_len() {
    let out = jit_capture(
        r#"class Bag:
    def __init__(self):
        self.items = []
    def __len__(self):
        return len(self.items)
    def add(self, x):
        self.items.append(x)

b = Bag()
print(len(b))
b.add(1)
b.add(2)
b.add(3)
print(len(b))
"#,
    );
    assert_output(&out, "0\n3\n");
}
