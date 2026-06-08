# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""dict_methods: reversed() over a dict and its views (PEP 372 ordering)."""

d = {"a": 1, "b": 2, "foo": 0, "c": 3, "d": 4}
del d["foo"]

# reversed(dict) walks keys in reverse insertion order.
r = reversed(d)
assert list(r) == ["d", "c", "b", "a"]

# The iterator is exhausted afterwards.
try:
    next(r)
    raise AssertionError("expected StopIteration")
except StopIteration:
    pass

# reversed over each view yields the same reverse insertion order.
assert list(reversed(d.keys())) == ["d", "c", "b", "a"]
assert list(reversed(d.values())) == [4, 3, 2, 1]
assert list(reversed(d.items())) == [("d", 4), ("c", 3), ("b", 2), ("a", 1)]

# An empty dict reverses to nothing.
assert list(reversed({})) == []

# reversed and forward iteration are exact mirrors.
e = {1: "x", 2: "y", 3: "z"}
assert list(reversed(e)) == list(e)[::-1]

print("reversed_iteration OK")
