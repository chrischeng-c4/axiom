# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtins: the 3-argument type(name, bases, namespace) constructor."""

# Distilled from CPython Lib/test/test_builtin.py TestType (re-curated:
# dropped slots/typeparams internals, kept the observable contract).

import collections

# Minimal new type: empty bases default to (object,).
A = type("A", (), {})
assert A.__name__ == "A"
assert A.__qualname__ == "A"
assert A.__bases__ == (object,)
assert A.__base__ is object
inst = A()
assert type(inst) is A
assert inst.__class__ is A

# Multiple bases, with a method namespace.
class Mixin:
    def ham(self):
        return "ham%d" % self


C = type("C", (Mixin, int), {"spam": lambda self: "spam%s" % self})
assert C.__bases__ == (Mixin, int)
assert C.__base__ is int
assert "spam" in C.__dict__       # defined here
assert "ham" not in C.__dict__    # inherited, not in own dict
c = C(42)
assert c == 42
assert c.ham() == "ham42"
assert c.spam() == "spam42"

# __qualname__ in the namespace overrides the qualified name only.
Q = type("Q", (), {"__qualname__": "Outer.Inner"})
assert Q.__name__ == "Q"
assert Q.__qualname__ == "Outer.Inner"

# __doc__ defaults to None when not supplied.
assert type("D", (), {}).__doc__ is None

# Namespace insertion order is preserved into __dict__.
od = collections.OrderedDict([("a", 1), ("b", 2)])
od.move_to_end("a")  # now b, a
Ordered = type("Ordered", (), od)
assert list(Ordered.__dict__.items())[:2] == [("b", 2), ("a", 1)]

# Bad-argument shapes all raise TypeError.
bad_calls = [
    lambda: type(),                       # no args
    lambda: type("A", ()),                # 2 args
    lambda: type("A", (), {}, ()),        # 4 positional args
    lambda: type("A", [], {}),            # bases must be a tuple
    lambda: type("A", (None,), {}),       # non-type base
    lambda: type("A", (bool,), {}),       # cannot subclass bool
    lambda: type("A", (int, str), {}),    # incompatible layout
    lambda: type("a", (), {}, x=5),       # unexpected keyword
]
for call in bad_calls:
    try:
        call()
        raise AssertionError("expected TypeError")
    except TypeError:
        pass

print("type_creation OK")
