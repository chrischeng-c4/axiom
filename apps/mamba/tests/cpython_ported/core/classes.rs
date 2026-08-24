//! Ported from Lib/test/test_class_ported.py
//! Integration tests: core/classes.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

/// Ported from `Lib/test/test_class_ported.py`.
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

