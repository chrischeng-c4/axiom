# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/tuple_methods: subclassing tuple (CPython 3.12 oracle)."""

# A bare subclass keeps its own type while behaving as a tuple.
class Plain(tuple):
    pass


u = Plain([1, 2])
assert type(u) is Plain
assert isinstance(u, tuple)
assert list(u) == [1, 2]
assert u[0] == 1 and u[-1] == 2
assert u.count(1) == 1
assert u.index(2) == 1

# The base tuple() constructor rejects keyword arguments even through a
# subclass that does not override __init__/__new__.
try:
    Plain(sequence=())  # type: ignore[call-overload]
    raise AssertionError("expected TypeError")
except TypeError:
    pass

# A subclass may add its own keyword via __init__; the tuple contents still
# come from the positional argument.
class WithInit(tuple):
    def __init__(self, seq, newarg=None):
        self.newarg = newarg


u = WithInit([1, 2], newarg=3)
assert type(u) is WithInit
assert list(u) == [1, 2]
assert u.newarg == 3

# A subclass may instead seed the tuple and stash extra state in __new__.
class WithNew(tuple):
    def __new__(cls, seq, newarg=None):
        self = super().__new__(cls, seq)
        self.newarg = newarg
        return self


u = WithNew([1, 2], newarg=3)
assert type(u) is WithNew
assert list(u) == [1, 2]
assert u.newarg == 3

# A subclass instance is still immutable and equal to the plain tuple value.
assert u == (1, 2)
assert hash(u) == hash((1, 2))

print("tuple_subclass OK")
