# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""dict_methods: keys()/items() views behave as sets under set algebra."""

d1 = {"a": 1, "b": 2}
d2 = {"b": 3, "c": 2}
d3 = {"d": 4, "e": 5}

# keys() view supports &, |, ^, - against other views and sets.
assert d1.keys() & d2.keys() == {"b"}
assert d1.keys() | d2.keys() == {"a", "b", "c"}
assert d1.keys() ^ d2.keys() == {"a", "c"}
assert d1.keys() - d2.keys() == {"a"}
assert d1.keys() & d3.keys() == set()

# Operands may be plain sets, tuples, lists, or any iterator.
assert d1.keys() & {"a"} == {"a"}
assert d1.keys() | (1, 2) == {"a", "b", 1, 2}
assert d1.keys() - ["a"] == {"b"}
assert d1.keys() & iter(["a", "z"]) == {"a"}

# Result of a key-view set op is always a plain set, never a view.
assert type(d1.keys() & d2.keys()) is set
assert type([] & d1.keys()) is set

# items() view also supports the full set algebra.
i1 = {"a": 1, "b": 2}.items()
i2 = {"a": 2, "b": 2}.items()
assert i1 & i2 == {("b", 2)}
assert i1 | i2 == {("a", 1), ("a", 2), ("b", 2)}
assert i1 ^ i2 == {("a", 1), ("a", 2)}
assert i1 - i2 == {("a", 1)}
assert i1 & (("a", 1),) == {("a", 1)}

# isdisjoint reports overlap correctly across iterable kinds.
assert not d1.keys().isdisjoint(d2.keys())
assert d1.keys().isdisjoint({"x", "y"})
assert d1.keys().isdisjoint(["x", "y"])
assert {}.keys().isdisjoint([])
assert not {"a": 1, "b": 2}.items().isdisjoint([("b", 2)])

print("views_set_ops OK")
