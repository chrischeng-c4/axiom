# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Subclassing float: __new__/__init__ keywords and fromhex (CPython 3.12 oracle)."""


# Plain subclass keeps its type but compares as the underlying float value.
class Sub(float):
    pass


u = Sub(2.5)
assert type(u) is Sub
assert float(u) == 2.5
assert u + 0.5 == 3.0

# float() itself rejects keyword arguments.
try:
    float(x="3.14")
    raise AssertionError("expected TypeError")
except TypeError as e:
    assert "keyword argument" in str(e)


# A subclass may add its own keyword via __init__ (value still set by float).
class WithInit(float):
    def __init__(self, arg, newarg=None):
        self.newarg = newarg


w = WithInit(2.5, newarg=3)
assert type(w) is WithInit
assert float(w) == 2.5
assert w.newarg == 3


# Or override __new__ to transform the stored value.
class WithNew(float):
    def __new__(cls, arg, newarg=None):
        self = super().__new__(cls, arg)
        self.newarg = newarg
        return self


n = WithNew(2.5, newarg=7)
assert type(n) is WithNew
assert float(n) == 2.5
assert n.newarg == 7


# fromhex on a subclass routes through that subclass's __new__.
class Plus1(float):
    def __new__(cls, value):
        return float.__new__(cls, value + 1)


p = Plus1.fromhex((1.5).hex())
assert type(p) is Plus1
assert p == 2.5

print("subclass OK")
