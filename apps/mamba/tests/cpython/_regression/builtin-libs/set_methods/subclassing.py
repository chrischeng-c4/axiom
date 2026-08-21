# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""set_methods: subclassing set and frozenset with custom __init__ / __new__."""

# A bare set subclass behaves like a set but keeps its own type.
class MySet(set):
    pass

m = MySet([1, 2, 2, 3])
assert type(m) is MySet
assert isinstance(m, set)
assert set(m) == {1, 2, 3}
m.add(4)
assert set(m) == {1, 2, 3, 4}

# The constructor still rejects unexpected keyword arguments.
try:
    MySet(sequence=())  # type: ignore[call-arg]
    raise AssertionError("expected TypeError")
except TypeError:
    pass

# A subclass may add state via __init__ alongside the inherited contents.
class SetWithInit(set):
    def __init__(self, arg, newarg=None):
        super().__init__(arg)
        self.newarg = newarg

s = SetWithInit([1, 2], newarg=3)
assert type(s) is SetWithInit
assert set(s) == {1, 2}
assert s.newarg == 3

# A frozenset subclass must thread extra args through __new__ (frozensets are
# built immutably, so __init__ cannot set their contents).
class FrozenWithNew(frozenset):
    def __new__(cls, arg, newarg=None):
        self = super().__new__(cls, arg)
        self.newarg = newarg
        return self

f = FrozenWithNew([1, 2], newarg=7)
assert type(f) is FrozenWithNew
assert isinstance(f, frozenset)
assert set(f) == {1, 2}
assert f.newarg == 7

# Subclass instances still compare equal to plain sets by value.
assert m.__class__ is MySet
assert MySet([1, 2, 3]) == {1, 2, 3}
assert FrozenWithNew([1, 2]) == frozenset([1, 2])

print("subclassing OK")
