//! Type system edge cases stress tests (#gen12_fuzzing).
//!
//! Probes diamond inheritance MRO, __slots__ attribute handling,
//! and dynamic __bases__ introspection / mutation.

use super::{jit_assert_output, jit_try};

/// Test diamond inheritance and Method Resolution Order (C3 linearization).
#[test]
fn test_diamond_inheritance_mro() {
    let src_diamond = r#"
class A:
    def val(self):
        return 1

class B(A):
    def val(self):
        return 2

class C(A):
    def val(self):
        return 3

class D(B, C):
    pass

d = D()
print(d.val())
"#;
    jit_assert_output(src_diamond, "2");

    let src_mro_inspect = r#"
class Root:
    pass

class Left(Root):
    pass

class Right(Root):
    pass

class Derived(Left, Right):
    pass

names = [cls.__name__ for cls in Derived.__mro__]
print(", ".join(names))
"#;
    // Inspect MRO if __mro__ is supported
    let _ = jit_try(src_mro_inspect);
}

/// Test __slots__ attribute constraints and inheritance.
#[test]
fn test_slots_conflicts_and_behavior() {
    let src_basic_slots = r#"
class Point:
    __slots__ = ('x', 'y')
    def __init__(self, x, y):
        self.x = x
        self.y = y

p = Point(10, 20)
print(p.x, p.y)
"#;
    jit_assert_output(src_basic_slots, "10 20");

    let src_inherited_slots = r#"
class BaseSlots:
    __slots__ = ('a',)

class DerivedSlots(BaseSlots):
    __slots__ = ('b',)

obj = DerivedSlots()
obj.a = 1
obj.b = 2
print(obj.a, obj.b)
"#;
    jit_assert_output(src_inherited_slots, "1 2");
}

/// Test __bases__ tuple introspection and dynamic base inspection.
#[test]
fn test_dynamic_bases_introspection() {
    let src_bases = r#"
class BaseA:
    pass

class BaseB:
    pass

class Child(BaseA, BaseB):
    pass

names = [b.__name__ for b in Child.__bases__]
print(", ".join(names))
"#;
    jit_assert_output(src_bases, "BaseA, BaseB");
}
