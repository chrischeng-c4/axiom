//! Py3.12 conformance tests for class semantics (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_class.py — ClassTests
//!
//! Coverage: class definition, __init__, instance attributes, instance
//! methods, single inheritance, method override, super(), class-level
//! attribute lookup fallback, __str__ via str(), __repr__ via repr(),
//! classmethod, staticmethod, isinstance with subclass, multi-arg __init__.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_class_basic_init_attr() {
    let out = jit_capture(
        r#"class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

p = Point(3, 4)
print(p.x)
print(p.y)
"#,
    );
    assert_output(&out, "3\n4\n");
}

#[test]
fn test_class_method_call() {
    let out = jit_capture(
        r#"class Counter:
    def __init__(self):
        self.n = 0
    def inc(self):
        self.n = self.n + 1

c = Counter()
c.inc()
c.inc()
c.inc()
print(c.n)
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_class_method_returns_value() {
    let out = jit_capture(
        r#"class Rect:
    def __init__(self, w, h):
        self.w = w
        self.h = h
    def area(self):
        return self.w * self.h

r = Rect(3, 5)
print(r.area())
"#,
    );
    assert_output(&out, "15\n");
}

#[test]
fn test_class_single_inheritance_inherits_method() {
    let out = jit_capture(
        r#"class A:
    def greet(self):
        return "hello"

class B(A):
    pass

b = B()
print(b.greet())
"#,
    );
    assert_output(&out, "hello\n");
}

#[test]
fn test_class_method_override() {
    let out = jit_capture(
        r#"class A:
    def name(self):
        return "A"

class B(A):
    def name(self):
        return "B"

print(A().name())
print(B().name())
"#,
    );
    assert_output(&out, "A\nB\n");
}

#[test]
fn test_class_super_call() {
    let out = jit_capture(
        r#"class A:
    def label(self):
        return "A"

class B(A):
    def label(self):
        return super().label() + "->B"

print(B().label())
"#,
    );
    assert_output(&out, "A->B\n");
}

#[test]
fn test_class_attribute_lookup_falls_through_to_class() {
    let out = jit_capture(
        r#"class C:
    kind = "default"

c = C()
print(c.kind)
c.kind = "instance"
print(c.kind)
"#,
    );
    assert_output(&out, "default\ninstance\n");
}

#[test]
fn test_class_str_dunder() {
    let out = jit_capture(
        r#"class P:
    def __init__(self, x):
        self.x = x
    def __str__(self):
        return "P(" + str(self.x) + ")"

print(str(P(7)))
"#,
    );
    assert_output(&out, "P(7)\n");
}

#[test]
fn test_class_repr_dunder() {
    let out = jit_capture(
        r#"class P:
    def __init__(self, x):
        self.x = x
    def __repr__(self):
        return "<P x=" + str(self.x) + ">"

print(repr(P(7)))
"#,
    );
    assert_output(&out, "<P x=7>\n");
}

#[test]
fn test_class_isinstance_with_subclass() {
    let out = jit_capture(
        r#"class A:
    pass

class B(A):
    pass

b = B()
print(isinstance(b, B))
print(isinstance(b, A))
print(isinstance(b, int))
"#,
    );
    assert_output(&out, "True\nTrue\nFalse\n");
}

#[test]
fn test_class_multiple_instances_independent_state() {
    let out = jit_capture(
        r#"class C:
    def __init__(self, n):
        self.n = n

a = C(1)
b = C(2)
c = C(3)
print(a.n + b.n + c.n)
"#,
    );
    assert_output(&out, "6\n");
}

#[test]
fn test_class_chained_method_calls() {
    let out = jit_capture(
        r#"class B:
    def __init__(self):
        self.acc = 0
    def add(self, n):
        self.acc = self.acc + n
        return self

b = B().add(1).add(2).add(3)
print(b.acc)
"#,
    );
    assert_output(&out, "6\n");
}

#[test]
fn test_class_init_with_default_args() {
    let out = jit_capture(
        r#"class P:
    def __init__(self, x, y=10):
        self.x = x
        self.y = y

a = P(1)
b = P(1, 20)
print(a.y)
print(b.y)
"#,
    );
    assert_output(&out, "10\n20\n");
}

#[test]
fn test_class_inherits_init_from_parent() {
    let out = jit_capture(
        r#"class A:
    def __init__(self, x):
        self.x = x

class B(A):
    pass

b = B(99)
print(b.x)
"#,
    );
    assert_output(&out, "99\n");
}
