# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""range: core construction, iteration, and sequence behavior."""


# Construction with 1, 2, and 3 args; negative and reverse steps.
assert list(range(3)) == [0, 1, 2]
assert list(range(1, 5)) == [1, 2, 3, 4]
assert list(range(0)) == []
assert list(range(-3)) == []
assert list(range(1, 10, 3)) == [1, 4, 7]
assert list(range(5, -5, -3)) == [5, 2, -1, -4]

# Empty ranges are falsy; non-empty are truthy.
assert not range(0)
assert not range(5, 5)
assert not range(0, -10)
assert range(1)

# len reflects element count for forward and reverse steps.
assert len(range(0, 10, 2)) == 5
assert len(range(10, 0, -3)) == 4
assert len(range(5, 5)) == 0

# Indexing, including negative indices and steps.
r = range(0, 20, 2)
assert r[0] == 0
assert r[3] == 6
assert r[-1] == 18
assert r[-2] == 16

# Slicing returns a new range.
assert range(10)[2:5] == range(2, 5)
assert range(10)[::2] == range(0, 10, 2)
assert range(10)[::-1] == range(9, -1, -1)
assert list(range(10)[2:8:3]) == [2, 5]

# repr round-trips: 2-arg form when step == 1, else 3-arg form.
assert repr(range(1)) == "range(0, 1)"
assert repr(range(1, 2)) == "range(1, 2)"
assert repr(range(1, 2, 3)) == "range(1, 2, 3)"

# reversed(range) equals the reverse of list(range), for assorted shapes.
for r in [range(10), range(0), range(1, 9, 3), range(8, 0, -3)]:
    assert list(reversed(r)) == list(r)[::-1]

# Equality is value-based over (effective) elements, not over the literal
# start/stop/step triple: differing triples can compare equal.
assert range(0) == range(2, 1, 3)
assert range(0, 3, 2) == range(0, 4, 2)
assert range(5) == range(0, 5, 1)
assert range(5) != range(6)
assert hash(range(0, 3, 2)) == hash(range(0, 4, 2))

# range never compares equal to a tuple or list, even with same elements.
assert (range(2) == (0, 1)) is False
assert (range(2) == [0, 1]) is False

# count() and index() behave as for any sequence; both return plain ints.
assert range(3).count(0) == 1
assert range(3).count(3) == 0
assert range(0, 10, 2).count(4) == 1
assert range(0, 10, 2).count(5) == 0
assert range(3).index(1) == 1
assert range(1, 10, 3).index(4) == 1
assert range(1, -10, -3).index(-5) == 2
assert type(range(3).count(0)) is int
assert type(range(3).index(0)) is int


# The constructor and slicing accept any object implementing __index__,
# using its integer value.
class Idx:
    def __init__(self, n):
        self.n = int(n)

    def __index__(self):
        return self.n


assert list(range(Idx(2), Idx(5))) == [2, 3, 4]
assert range(10)[: Idx(5)] == range(5)
assert range(10)[Idx(2) : Idx(8) : Idx(3)] == range(2, 8, 3)

print("behavior OK")
