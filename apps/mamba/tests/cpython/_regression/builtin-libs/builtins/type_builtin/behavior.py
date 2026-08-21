"""Behavior contract for builtins.type.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: type(obj) returns the type
assert type(1) is int
assert type(1.0) is float
assert type("a") is str
assert type(b"a") is bytes
assert type([]) is list
assert type(()) is tuple
assert type({}) is dict
assert type(set()) is set
assert type(frozenset()) is frozenset
assert type(True) is bool
assert type(None) is type(None)

# Rule 2: type(type) is type (metaclass loop)
assert type(int) is type, f"type(int) = {type(int).__name__!r}"
assert type(type) is type, f"type(type) = {type(type).__name__!r}"

# Rule 3: three-arg form creates a new class
Point = type("Point", (object,), {"x": 0, "y": 0})
assert Point.__name__ == "Point", f"__name__ = {Point.__name__!r}"
p = Point()
assert isinstance(p, Point), "isinstance(p, Point) failed"
assert isinstance(p, object), "isinstance(p, object) failed"

# Rule 4: inheritance via three-arg form
class Base:
    def greet(self):
        return "hello"
Child = type("Child", (Base,), {})
c = Child()
assert c.greet() == "hello", f"inherited greet = {c.greet()!r}"
assert issubclass(Child, Base), "Child not subclass of Base"

# Rule 5: type.__subclasses__ behavior via issubclass
assert issubclass(bool, int), "bool issubclass of int"
assert issubclass(int, object), "int issubclass of object"
assert not issubclass(str, int), "str should not be subclass of int"

# Rule 6: wrong arg count for one-arg vs three-arg
_raised = False
try:
    type()  # type: ignore[call-overload]
except TypeError:
    _raised = True
assert _raised, "type() with no args did not raise TypeError"

_raised = False
try:
    type("A", (object,))  # type: ignore[call-overload]
except TypeError:
    _raised = True
assert _raised, "type('A', (object,)) with 2 args did not raise TypeError"

# Rule 7: type.__mro__ on user class
class A: pass
class B(A): pass
assert B.__mro__ == (B, A, object), f"__mro__ = {B.__mro__!r}"

print("behavior OK")
