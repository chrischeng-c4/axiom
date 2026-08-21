//! Py3.12 conformance tests for the `abc` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_abc.py):
//!   ABC subclassing, abstractmethod implementation, isinstance against
//!   abstract base.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_abc_concrete_subclass_implements_abstract() {
    let out = jit_capture(
        r#"import abc
class Shape(abc.ABC):
    @abc.abstractmethod
    def area(self):
        pass
class Square(Shape):
    def __init__(self, s):
        self.s = s
    def area(self):
        return self.s * self.s
print(Square(3).area())
print(Square(4).area())
"#,
    );
    assert_output(&out, "9\n16\n");
}

#[test]
fn test_abc_isinstance_against_base() {
    let out = jit_capture(
        r#"import abc
class Animal(abc.ABC):
    @abc.abstractmethod
    def sound(self):
        pass
class Dog(Animal):
    def sound(self):
        return "woof"
d = Dog()
print(isinstance(d, Animal))
print(isinstance(d, Dog))
print(d.sound())
"#,
    );
    assert_output(&out, "True\nTrue\nwoof\n");
}

#[test]
fn test_abc_method_dispatch_through_base() {
    let out = jit_capture(
        r#"import abc
class Base(abc.ABC):
    @abc.abstractmethod
    def value(self):
        pass
    def doubled(self):
        return self.value() * 2
class Concrete(Base):
    def value(self):
        return 21
print(Concrete().doubled())
"#,
    );
    assert_output(&out, "42\n");
}
