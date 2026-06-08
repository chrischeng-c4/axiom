# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/tuple_methods: core tuple behavior (CPython 3.12 oracle)."""

# tuple() over different iterables.
assert tuple() == ()
assert tuple([]) == ()
assert tuple([0, 1, 2, 3]) == (0, 1, 2, 3)
assert tuple("spam") == ("s", "p", "a", "m")
assert tuple(x for x in range(10) if x % 2) == (1, 3, 5, 7, 9)

# tuple() of a tuple returns an equal value; CPython may share the object,
# but equality is what callers rely on.
src = (0, 1, 2, 3)
assert tuple(src) == src

# Concatenation and repetition build new tuples.
assert (1, 2) + (3, 4) == (1, 2, 3, 4)
assert () + (1, 2) == (1, 2)
assert (1, 2) * 3 == (1, 2, 1, 2, 1, 2)
assert 3 * (1, 2) == (1, 2, 1, 2, 1, 2)
assert (1, 2) * 0 == ()
assert (1, 2) * -1 == ()

# Lexicographic comparison: element-wise, with a prefix being smaller.
assert (1, 2, 3) == (1, 2, 3)
assert (1, 2, 3) < (1, 2, 4)
assert (1, 2) < (1, 2, 0)
assert (2,) > (1, 9, 9)
assert (1, 2, 3) <= (1, 2, 3)

# Hashing: equal tuples hash equally; the empty tuple is hashable; nesting
# is fine as long as every element is hashable.
assert hash(()) == hash(())
assert hash((1, 2, 3)) == hash((1, 2, 3))
assert hash((0.5, (), (-2, 3, (4, 6)))) == hash((0.5, (), (-2, 3, (4, 6))))
assert len({(1, 2), (1, 2), (3, 4)}) == 2

# Building a tuple from a long generator must grow correctly under the hood
# (CPython once had a resize bug here); the result preserves order and length.
def first_n(n):
    for i in range(n):
        yield i


assert list(tuple(first_n(1000))) == list(range(1000))
assert len(tuple(first_n(1000))) == 1000

# Star-unpacking and swap.
a, *rest = (1, 2, 3, 4, 5)
assert a == 1 and rest == [2, 3, 4, 5]
x, y = 10, 20
x, y = y, x
assert (x, y) == (20, 10)

print("behavior OK")
