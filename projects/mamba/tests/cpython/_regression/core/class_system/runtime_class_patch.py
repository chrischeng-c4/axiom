# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Patching methods on a base class affects live instances (CPython 3.12)."""


class A:
    pass


class B(A):
    pass


class C(A):
    pass


class D(B, C):
    pass


d = D()

# Adding __hash__ to a base in the MRO makes it visible on existing instances,
# and a more-derived base overrides a less-derived one.
A.__hash__ = lambda self: 42
assert hash(d) == 42
C.__hash__ = lambda self: 314
assert hash(d) == 314          # C precedes A in D's MRO
B.__hash__ = lambda self: 144
assert hash(d) == 144          # B precedes C
del B.__hash__
assert hash(d) == 314          # falls back to C
del C.__hash__
assert hash(d) == 42           # falls back to A

# Setting __hash__ = None makes instances unhashable.
A.__hash__ = None
try:
    hash(d)
    print("unhashable: no_raise")
except TypeError:
    print("unhashable: TypeError")


# Patching a regular method on a base is immediately visible.
class Greeter:
    def hello(self):
        return "hi"


g = Greeter()
assert g.hello() == "hi"
Greeter.hello = lambda self: "HELLO"
assert g.hello() == "HELLO"    # same live instance sees the new method

# Adding a brand-new method to the class reaches existing instances too.
Greeter.shout = lambda self: "!!!"
assert g.shout() == "!!!"

print("runtime_class_patch OK")
