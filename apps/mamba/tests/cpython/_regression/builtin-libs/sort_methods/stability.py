# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""sort methods: stability guarantees (CPython 3.12 oracle).

Python's sort is stable: records that compare equal keep their original
relative order. This holds for forward sorts, for reverse sorts, and when a
key collapses many distinct items onto the same value.
"""

# (key, original_index) pairs; the index tags original position.
data = [(5, 0), (3, 1), (5, 2), (3, 3), (5, 4)]

# Forward stable sort by key: equal keys keep ascending original index.
forward = sorted(data, key=lambda t: t[0])
assert forward == [(3, 1), (3, 3), (5, 0), (5, 2), (5, 4)]
print("forward:", forward)

# Sorting by full tuple matches sorting by key for this data (the index
# tie-break already preserves order), confirming the stability invariant.
assert sorted(data) == forward
print("key_eq_natural: ok")

# Reverse stable sort: keys descend, but among equal keys the original
# order is STILL ascending (reverse does not reverse the ties).
backward = sorted(data, key=lambda t: t[0], reverse=True)
assert backward == [(5, 0), (5, 2), (5, 4), (3, 1), (3, 3)]
print("reverse:", backward)

# reverse=True is equivalent to sorting with a negated comparison key,
# preserving tie order either way.
neg = sorted(data, key=lambda t: -t[0])
assert neg == backward
print("reverse_eq_negkey: ok")

# A key collapsing everything to one value leaves the list untouched.
flat = sorted(data, key=lambda t: 0)
assert flat == data
print("constant_key:", flat)

# Stability also applies to list.sort() in place.
inplace = data[:]
inplace.sort(key=lambda t: t[0])
assert inplace == forward
print("inplace_stable: ok")

print("stability OK")
