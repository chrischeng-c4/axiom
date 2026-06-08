# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/list_methods: core list behavior (CPython 3.12 oracle)."""

# list() over different iterables.
assert list([]) == []
assert list(()) == []
assert list((0, 1, 2, 3)) == [0, 1, 2, 3]
assert list("") == []
assert list("spam") == ["s", "p", "a", "m"]
assert list(x for x in range(10) if x % 2) == [1, 3, 5, 7, 9]

# list() copies its source: equal by value, distinct by identity.
src = [0, 1, 2, 3]
copy = list(src)
assert src == copy
assert src is not copy

# Two distinct empty-list literals are never the same object.
assert [] is not []

# extend() consuming a generator over the very list being extended:
# the generator sees the empty list, so nothing is appended.
x = []
x.extend(-y for y in x)
assert x == []

# Extended-slice assignment with a matching-size source reorders in place.
m = list(range(5))
m[::-1] = [10, 20, 30, 40, 50]
assert m == [50, 40, 30, 20, 10]

m2 = list(range(6))
m2[::2] = [100, 200, 300]
assert m2 == [100, 1, 200, 3, 300, 5]

# A huge step on an extended slice selects only the first matching element.
a = [0, 1, 2, 3, 4]
a[1::1_000_000] = [9]
assert a == [0, 9, 2, 3, 4]
assert a[3::1_000_000] == [3]

# del with an open-ended slice truncates the list.
lst = [0] * 65
del lst[1:]
assert len(lst) == 1

print("behavior OK")
