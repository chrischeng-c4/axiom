# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/set_methods: behavior asserts (CPython 3.12 oracle)."""

# Core algebra, both operator and method forms agree.
a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
assert a | b == {1, 2, 3, 4, 5, 6} == a.union(b)
assert a & b == {3, 4} == a.intersection(b)
assert a - b == {1, 2} == a.difference(b)
assert a ^ b == {1, 2, 5, 6} == a.symmetric_difference(b)

# Equality ignores element order and the set/frozenset distinction.
assert {1, 2, 3} == {3, 2, 1}
assert {1, 2, 3} == frozenset([1, 2, 3])
assert frozenset([1, 2, 3]) == {1, 2, 3}

# A set is never equal to a non-set, even a same-element sequence.
assert ({1, 2} == [1, 2]) is False
assert ({1, 2} == (1, 2)) is False
assert (set("ab") == "ab") is False
assert (set("ab") != "ab") is True

# Membership, length, emptiness.
assert 2 in a and 99 not in a
assert len(a) == 4
assert not set()
assert bool({0}) is True

# copy is a shallow, independent set.
c = a.copy()
c.add(99)
assert 99 not in a and 99 in c

# Dedup at construction; order-independent value.
assert set([1, 1, 2, 2, 3]) == {1, 2, 3}
assert sorted(set("mississippi")) == ["i", "m", "p", "s"]

# Comprehension yields a real set.
assert {x % 3 for x in range(10)} == {0, 1, 2}

# Disjointness.
assert {1, 2}.isdisjoint({3, 4})
assert not {1, 2}.isdisjoint({2, 3})

print("behavior OK")
