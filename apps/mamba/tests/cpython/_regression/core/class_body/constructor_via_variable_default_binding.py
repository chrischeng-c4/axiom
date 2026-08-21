"""Constructing a class through a variable (`Alias = Cls; Alias(...)`), not
the literal class name, fills omitted defaults exactly like a literal
`Cls(...)` call — for both `__init__` and a custom `__new__`, covering
positional, multi-trailing, and keyword-only defaults, with explicit
arguments still overriding them (regression: #1044, the value-call
construction route read uninitialized memory for any omitted default)."""


class Point:
    def __init__(self, a, b=2, c=3):
        self.a = a
        self.b = b
        self.c = c


PointAlias = Point

p1 = PointAlias(1)
assert (p1.a, p1.b, p1.c) == (1, 2, 3)

p2 = PointAlias(1, 9)
assert (p2.a, p2.b, p2.c) == (1, 9, 3)

p3 = PointAlias(1, c=99)
assert (p3.a, p3.b, p3.c) == (1, 2, 99)


class KwOnly:
    def __init__(self, a, *, k=5):
        self.a = a
        self.k = k


KwOnlyAlias = KwOnly

k1 = KwOnlyAlias(1)
assert (k1.a, k1.k) == (1, 5)

k2 = KwOnlyAlias(1, k=9)
assert (k2.a, k2.k) == (1, 9)


class WithNew:
    def __new__(cls, a, b=7):
        obj = object.__new__(cls)
        obj.a = a
        obj.b = b
        return obj


WithNewAlias = WithNew

n1 = WithNewAlias(1)
assert (n1.a, n1.b) == (1, 7)

n2 = WithNewAlias(1, 42)
assert (n2.a, n2.b) == (1, 42)

print("constructor_via_variable_default_binding OK")
