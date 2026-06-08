# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/unpacking: assignment-target unpacking behavior (CPython 3.12 oracle).

Complements sequence_unpack.py (tuple/list/iterable/nested/starred). Here we
cover the source shapes that are easy to forget: implied tuple on the rhs,
unpacking a str, single-target trailing-comma syntax, an arbitrary object
that only implements __getitem__, and the four empty-target forms.
"""

# Implied tuple on the right-hand side (no parentheses).
a, b, c = 7, 8, 9
assert (a, b, c) == (7, 8, 9)
print("implied-tuple:", a, b, c)

# Unpacking a str yields its characters.
p, q, r = "one"
assert (p, q, r) == ("o", "n", "e")
print("str-unpack:", p, q, r)

# Single-element unpacking via a trailing comma on the target.
(only_t,) = (99,)
(only_l,) = [100]
assert only_t == 99 and only_l == 100
print("single-elem:", only_t, only_l)


# An object that is iterable purely through __getitem__ / IndexError.
class Seq:
    def __getitem__(self, i):
        if 0 <= i < 3:
            return i
        raise IndexError


s0, s1, s2 = Seq()
assert (s0, s1, s2) == (0, 1, 2)
print("getitem-seq:", s0, s1, s2)

# Empty-target unpacking: each empty source/target combination is legal.
() = []
[] = ()
[] = []
() = ()
print("empty-targets: ok")

print("behavior OK")
