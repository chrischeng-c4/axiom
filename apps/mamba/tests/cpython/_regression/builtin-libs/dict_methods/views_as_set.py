# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""dict_methods: dict_keys/items/values views as set-like, comparable objects."""

d = {1: 10, "a": "ABC"}

# keys() is set-like: len, membership, set() materialization, equality.
keys = d.keys()
assert len(keys) == 2
assert set(keys) == {1, "a"}
assert keys == {1, "a"}
assert keys != {1, "a", "b"}
assert keys != {1}
assert keys != 42
assert 1 in keys
assert "a" in keys
assert 10 not in keys

# items() is set-like over (key, value) pairs.
items = d.items()
assert len(items) == 2
assert set(items) == {(1, 10), ("a", "ABC")}
assert items == {(1, 10), ("a", "ABC")}
assert items != {(1, 10), ("a", "def")}
assert items != 42
assert (1, 10) in items
assert ("a", "ABC") in items
assert (1, 11) not in items
assert 1 not in items          # bare key is not an item pair
assert (1,) not in items       # wrong arity
assert (1, 2, 3) not in items

# values() supports len + membership materialization (not set equality).
values = d.values()
assert len(values) == 2
assert set(values) == {10, "ABC"}
assert 10 in values
assert 99 not in values

# Views compare to one another and reflect the live dict.
e = d.copy()
assert d.keys() == e.keys()
assert d.items() == e.items()
e["a"] = "def"
assert d.items() != e.items()
assert d.keys() == e.keys()    # same keys, different values

# A keys() view of (k1, k2) pairs can equal an items() view of {k1: k2}.
dk = {(1, 1): 11, (2, 2): 22}
ei = {1: 1, 2: 2}
assert dk.keys() == ei.items()
assert dk.items() != ei.keys()

# Each view exposes a read-only .mapping proxy that equals the source dict.
mp = type(type.__dict__)
src = {"foo": "bar"}
for m in (src.keys().mapping, src.values().mapping, src.items().mapping):
    assert isinstance(m, mp)
    assert m == src

print("views_as_set OK")
