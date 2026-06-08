# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/unpacking: error paths for assignment unpacking (CPython 3.12 oracle).

The arity ValueErrors and the non-iterable TypeError carry specific, stable
messages; we assert on those exact strings. We also check that an exception
raised inside a custom __getitem__ during unpacking propagates verbatim
instead of being swallowed by the sequence-length probe.
"""


# Unpacking a non-iterable scalar raises TypeError with a precise message.
try:
    a, b, c = 7
    print("non-iterable: no_raise")
except TypeError as e:
    assert str(e) == "cannot unpack non-iterable int object", str(e)
    print("non-iterable:", type(e).__name__, str(e))


# Too many values for the fixed target arity.
try:
    a, b = (1, 2, 3)
    print("too-many: no_raise")
except ValueError as e:
    assert str(e) == "too many values to unpack (expected 2)", str(e)
    print("too-many:", type(e).__name__, str(e))


# Empty target with a non-empty source: expected 0.
try:
    () = [42]
    print("expected-0: no_raise")
except ValueError as e:
    assert str(e) == "too many values to unpack (expected 0)", str(e)
    print("expected-0:", type(e).__name__, str(e))


# A short __getitem__ sequence reports how many it actually yielded.
class Seq:
    def __getitem__(self, i):
        if 0 <= i < 3:
            return i
        raise IndexError


try:
    a, b, c, d = Seq()
    print("too-few: no_raise")
except ValueError as e:
    assert str(e) == "not enough values to unpack (expected 4, got 3)", str(e)
    print("too-few:", type(e).__name__, str(e))


# An unexpected exception from __getitem__ propagates unchanged; it is not
# converted into a ValueError by the length check.
class BozoError(Exception):
    pass


class BadSeq:
    def __getitem__(self, i):
        if 0 <= i < 3:
            return i
        if i == 3:
            raise BozoError
        raise IndexError


try:
    a, b, c, d, e = BadSeq()
    print("propagate: no_raise")
except BozoError:
    print("propagate: BozoError")
