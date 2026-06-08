"""Behavior contract for builtins.object.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: object() creates a new instance each time
a = object()
b = object()
assert a is not b, "object() not creating fresh instances"
assert isinstance(a, object)
assert isinstance(b, object)

# Rule 2: object.__repr__ returns a string containing class name
r = repr(object())
assert "object" in r, f"repr(object()) = {r!r}"
assert r.startswith("<object object"), f"repr format: {r!r}"

# Rule 3: object equality is identity-based by default
a = object()
b = object()
assert a == a, "a == a failed"
assert a != b, "a != b failed"
assert not (a == b), "a == b should be False"

# Rule 4: object is hashable by default
h = hash(object())
assert isinstance(h, int), "hash(object()) not int"

# Rule 5: object doesn't allow setting arbitrary attributes
_raised = False
try:
    o = object()
    o.x = 1  # type: ignore[attr-defined]
except AttributeError:
    _raised = True
assert _raised, "object().x = 1 did not raise AttributeError"

# Rule 6: subclass can set attributes
class _Sub(object):
    pass
s = _Sub()
s.x = 42  # type: ignore[attr-defined]
assert s.x == 42, f"subclass attr = {s.x!r}"

# Rule 7: object.__class__ returns its type
o = object()
assert o.__class__ is object, f"o.__class__ = {o.__class__!r}"
assert type(o) is object, f"type(o) = {type(o).__name__!r}"

# Rule 8: object.__init__ and __new__ are callable
assert callable(object.__init__), "object.__init__ not callable"
assert callable(object.__new__), "object.__new__ not callable"

# Rule 9: object is at top of MRO for all user classes
class Foo: pass
assert object in Foo.__mro__, "object not in Foo.__mro__"
assert Foo.__mro__[-1] is object, "object not at end of MRO"

# Rule 10: object() returns an instance with no attributes beyond defaults
o = object()
_d = dir(o)
assert "__class__" in _d, "__class__ in dir(object())"
assert "__hash__" in _d, "__hash__ in dir(object())"
assert "__repr__" in _d, "__repr__ in dir(object())"

print("behavior OK")
