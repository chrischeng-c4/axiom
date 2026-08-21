# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""set_methods: frozensets are hashable and can nest inside sets/frozensets."""

# A frozenset is hashable and equal frozensets share a hash.
fa = frozenset([1, 2, 3])
fb = frozenset([3, 2, 1])
assert fa == fb
assert hash(fa) == hash(fb)

# Frozensets dedup by value when collected into a set: the five spellings
# below collapse to three distinct character sets.
spellings = map(frozenset, ["abcdef", "bcd", "bdcb", "fed", "fedccba"])
distinct = set(spellings)
assert len(distinct) == 3
assert frozenset("abcdef") in distinct  # "fedccba" is the same set
assert frozenset("bcd") in distinct     # "bdcb" too

# A frozenset can be an element of an ordinary set, and round-trips by value.
inner = frozenset([1])
outer = {inner}
popped = outer.pop()
assert type(popped) is frozenset
assert popped == inner
outer.add(inner)
outer.remove(frozenset([1]))   # remove by an equal-but-distinct object
assert outer == set()
outer.discard(inner)           # discarding the absent element is silent

# Frozensets also nest inside other frozensets.
nested = frozenset([frozenset([1, 2]), frozenset([3])])
assert frozenset([1, 2]) in nested
assert len(nested) == 2

# A frozenset works as a dict key.
table = {frozenset([1, 2]): "pair", frozenset(): "empty"}
assert table[frozenset([2, 1])] == "pair"
assert table[frozenset()] == "empty"

print("frozenset_nesting OK")
