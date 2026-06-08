# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""list_methods: subclassing list (CPython 3.12 oracle)."""

# A bare subclass keeps its own type while behaving as a list.
class Plain(list):
    pass


u = Plain([1, 2])
assert type(u) is Plain
assert isinstance(u, list)
assert list(u) == [1, 2]

# The base list() constructor rejects keyword arguments even via a subclass
# that does not override __init__/__new__.
try:
    Plain(sequence=())  # type: ignore[call-overload]
    raise AssertionError("expected TypeError")
except TypeError:
    pass

# A subclass may add its own keyword via __init__ and still seed the list.
class WithInit(list):
    def __init__(self, seq, newarg=None):
        super().__init__(seq)
        self.newarg = newarg


u = WithInit([1, 2], newarg=3)
assert type(u) is WithInit
assert list(u) == [1, 2]
assert u.newarg == 3

# A subclass may add its own keyword via __new__ instead.
class WithNew(list):
    def __new__(cls, seq, newarg=None):
        self = super().__new__(cls, seq)
        self.newarg = newarg
        return self


u = WithNew([1, 2], newarg=3)
assert type(u) is WithNew
assert list(u) == [1, 2]
assert u.newarg == 3

# Inherited methods mutate the subclass instance in place.
u.append(9)
assert list(u) == [1, 2, 9]
u.sort()
assert list(u) == [1, 2, 9]

print("list_subclass OK")
