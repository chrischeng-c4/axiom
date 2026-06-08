# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/tuple_methods: surface probes (CPython 3.12 oracle)."""

# The two public tuple methods are present and callable, both on the type
# and on instances.
for name in ("count", "index"):
    assert hasattr(tuple, name), name
    assert callable(getattr(tuple, name)), name

sample = (1, 2, 3)
for name in ("count", "index"):
    assert callable(getattr(sample, name)), name

# tuple is its own type and round-trips through its constructor.
assert type(()) is tuple
assert tuple is type(tuple())

# Core sequence-protocol dunders are wired up (tuple is immutable, so it has
# no __setitem__ / __delitem__).
for dunder in (
    "__len__", "__getitem__", "__contains__", "__iter__",
    "__add__", "__mul__", "__rmul__", "__hash__",
    "__eq__", "__ne__", "__lt__", "__le__", "__gt__", "__ge__",
    "__getnewargs__",
):
    assert hasattr(tuple, dunder), dunder

# Tuples are immutable: the mutating sequence dunders are absent.
for absent in ("__setitem__", "__delitem__"):
    assert not hasattr(tuple, absent), absent

# Unlike list, tuple is hashable, so __hash__ is not None.
assert tuple.__hash__ is not None

print("surface OK")
