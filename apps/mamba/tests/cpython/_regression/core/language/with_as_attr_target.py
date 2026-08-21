# with...as: attribute / subscript targets in the `as` clause.
#
# CPython grammar permits any assignable target after `as`, not just a
# bare identifier. Mamba previously only accepted a Name token there
# and rejected `as obj.attr` / `as d[key]` with a syntax error. The
# parser now desugars such targets to a fresh temp + a prepended
# assignment (matching the existing `as (a, b)` tuple-unpack desugar).

class CM:
    def __init__(self, v):
        self.v = v
    def __enter__(self):
        return self.v
    def __exit__(self, *a):
        return False

class Holder:
    def __init__(self):
        self.foo: str = ""
        self.x: str = ""
        self.y: str = ""

# Attribute target.
h = Holder()
with CM("attr") as h.foo:
    pass
print(h.foo)

# Subscript target.
d: dict = {}
with CM("sub") as d["k"]:
    pass
print(d["k"])

# Parenthesized form with attribute targets in two items.
o = Holder()
with (CM("p1") as o.x, CM("p2") as o.y):
    pass
print(o.x, o.y)

# Tuple-target inside `as (...)` — each element may be any assignable
# target, including attribute access.
class TupCM:
    def __enter__(self):
        return ("t1", "t2")
    def __exit__(self, *a):
        return False

t = Holder()
with TupCM() as (t.x, t.y):
    pass
print(t.x, t.y)
