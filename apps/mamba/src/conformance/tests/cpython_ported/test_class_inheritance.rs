//! Py3.12 conformance tests for single inheritance and method override
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_class.py — single
//! inheritance and polymorphic dispatch sections): subclass with
//! overridden method, polymorphic iteration over a heterogeneous list,
//! and access to inherited fields.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_subclass_override() {
    let out = jit_capture(
        r#"class Animal:
    def __init__(self, name):
        self.name = name
    def speak(self):
        return self.name + " makes a sound"

class Dog(Animal):
    def speak(self):
        return self.name + " barks"

class Cat(Animal):
    def speak(self):
        return self.name + " meows"

print(Animal("Critter").speak())
print(Dog("Rex").speak())
print(Cat("Whiskers").speak())
"#,
    );
    assert_output(&out, "Critter makes a sound\nRex barks\nWhiskers meows\n");
}

#[test]
fn test_polymorphic_dispatch_in_list() {
    let out = jit_capture(
        r#"class Animal:
    def __init__(self, name):
        self.name = name
    def speak(self):
        return self.name + " makes a sound"

class Dog(Animal):
    def speak(self):
        return self.name + " barks"

class Cat(Animal):
    def speak(self):
        return self.name + " meows"

zoo = [Animal("X"), Dog("Y"), Cat("Z")]
for x in zoo:
    print(x.speak())
"#,
    );
    assert_output(&out, "X makes a sound\nY barks\nZ meows\n");
}

#[test]
fn test_inherited_method_and_attribute_access() {
    let out = jit_capture(
        r#"class Shape:
    def __init__(self, name):
        self.name = name
    def label(self):
        return "shape:" + self.name

class Square(Shape):
    def area(self, side):
        return side * side

s = Square("sq")
print(s.name)
print(s.label())
print(s.area(4))
print(s.area(7))
"#,
    );
    assert_output(&out, "sq\nshape:sq\n16\n49\n");
}
