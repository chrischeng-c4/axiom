# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""range: arbitrary-precision (bignum) bounds, evaluated lazily."""

# range stores Python ints, so bounds far beyond the machine word still
# index, count, and compare correctly without materializing the sequence.

# Length and full materialization agree for a small bignum-bounded range.
x = range(10**20, 10**20 + 10, 3)
assert len(x) == 4
assert len(list(x)) == 4

# Direction mismatch yields an empty range.
x = range(10**20 + 10, 10**20, 3)
assert len(x) == 0
assert not x
x = range(10**20, 10**20 + 10, -3)
assert len(x) == 0
assert not x

# Negative bignum bounds and steps.
for x in [range(-(2**100)), range(0, -(2**100)), range(0, 2**100, -1)]:
    assert list(x) == []
    assert not x

# count() and index() work on huge sparse ranges without scanning.
assert range(10**20).count(1) == 1
assert range(10**20).count(10**20) == 0
assert range(10**20).index(1) == 1
assert range(10**20).index(10**20 - 1) == 10**20 - 1
assert range(1, 2**100, 2).count(2**87) == 0
assert range(1, 2**100, 2).count(2**87 + 1) == 1
assert range(1, 2**100, 2).index(2**87 + 1) == 2**86

# membership on a huge strided range respects parity.
assert (2**87 + 1) in range(1, 2**100, 2)
assert (2**87) not in range(1, 2**100, 2)

# Equality and hashing depend on the elements, so different triples that
# enumerate the same values compare and hash equal.
assert range(0, 2**100 - 1, 2) == range(0, 2**100, 2)
assert hash(range(0, 2**100 - 1, 2)) == hash(range(0, 2**100, 2))
assert range(0, 2**100, 2) != range(0, 2**100 + 1, 2)
assert range(2**200, 2**201 - 2**99, 2**100) == range(2**200, 2**201, 2**100)

# Indexing a bignum-bounded range computes start + i*step exactly.
import sys

a = 0
b = sys.maxsize**4
c = 2 * sys.maxsize
x = range(a, b, c)
assert a in x
assert b not in x
assert x[0] == a
idx = sys.maxsize + 1
assert x[idx] == a + idx * c
assert x[idx : idx + 1][0] == a + idx * c

print("big_int OK")
