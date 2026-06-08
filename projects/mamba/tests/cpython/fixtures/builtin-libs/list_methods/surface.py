# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/list_methods: surface probes (CPython 3.12 oracle)."""

# Every documented list mutator / accessor method is present and callable.
for name in (
    "append", "extend", "insert", "remove", "pop", "clear",
    "index", "count", "sort", "reverse", "copy",
):
    assert hasattr(list, name), name
    assert callable(getattr(list, name)), name

# Bound methods exist on instances too.
sample = [1, 2, 3]
for name in ("append", "pop", "index", "count", "sort"):
    assert callable(getattr(sample, name)), name

# list is its own type and constructs from any iterable.
assert type([]) is list
assert list is type(list())

# Core protocol dunders are wired up.
for dunder in (
    "__len__", "__getitem__", "__setitem__", "__delitem__",
    "__contains__", "__iter__", "__add__", "__mul__",
    "__eq__", "__lt__", "__reversed__",
):
    assert hasattr(list, dunder), dunder

print("surface OK")
