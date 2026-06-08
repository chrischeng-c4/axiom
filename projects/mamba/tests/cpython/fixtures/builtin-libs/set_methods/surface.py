# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/set_methods: surface probes (CPython 3.12 oracle)."""

# Every documented set / frozenset method and dunder must resolve.

MUTATING = [
    "add", "remove", "discard", "pop", "clear", "update",
    "intersection_update", "difference_update", "symmetric_difference_update",
]
SHARED = [
    "union", "intersection", "difference", "symmetric_difference",
    "issubset", "issuperset", "isdisjoint", "copy",
]

for name in SHARED:
    assert hasattr(set, name), "set missing " + name
    assert hasattr(frozenset, name), "frozenset missing " + name

# Mutating methods live on set but not on the immutable frozenset.
for name in MUTATING:
    assert hasattr(set, name), "set missing " + name
    assert not hasattr(frozenset, name), "frozenset unexpectedly has " + name

# Operator dunders resolve on both types.
for dunder in ["__or__", "__and__", "__sub__", "__xor__",
               "__contains__", "__iter__", "__len__", "__eq__"]:
    assert hasattr(set, dunder)
    assert hasattr(frozenset, dunder)

# In-place operator dunders are set-only.
for dunder in ["__ior__", "__iand__", "__isub__", "__ixor__"]:
    assert hasattr(set, dunder)
    assert not hasattr(frozenset, dunder)

# Constructors are callable and produce the right type.
assert type(set([1, 2])) is set
assert type(frozenset([1, 2])) is frozenset
assert type({1, 2}) is set

print("surface OK")
