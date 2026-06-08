//! Py3.12 conformance tests for `@classmethod` and `@staticmethod`
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_descr.py — classmethod
//! and staticmethod sections):
//!   classmethod mutates class-level state via `cls`; staticmethod
//!   ignores its receiver; both dispatch correctly from instances.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_classmethod_mutates_class_state() {
    let out = jit_capture(
        r#"class Counter:
    count = 0
    @classmethod
    def inc(cls):
        cls.count += 1
        return cls.count

print(Counter.inc())
print(Counter.inc())
print(Counter.count)
"#,
    );
    assert_output(&out, "1\n2\n2\n");
}

#[test]
fn test_staticmethod_called_on_class() {
    let out = jit_capture(
        r#"class M:
    @staticmethod
    def add(a, b):
        return a + b

print(M.add(2, 3))
"#,
    );
    assert_output(&out, "5\n");
}

#[test]
fn test_staticmethod_called_on_instance() {
    let out = jit_capture(
        r#"class M:
    @staticmethod
    def mul(a, b):
        return a * b

m = M()
print(m.mul(4, 5))
"#,
    );
    assert_output(&out, "20\n");
}
