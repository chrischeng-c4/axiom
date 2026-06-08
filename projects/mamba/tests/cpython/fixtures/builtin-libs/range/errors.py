# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""range: documented exception paths (CPython 3.12 oracle)."""


def expect(exc, fn, label):
    try:
        fn()
    except exc as e:
        print(label + ":", type(e).__name__)
        return
    print(label + ": NO_RAISE")
    raise AssertionError(label + " did not raise " + exc.__name__)


# Wrong argument count: zero args and more than three args.
expect(TypeError, lambda: range(), "no_args")
expect(TypeError, lambda: range(1, 2, 3, 4), "too_many_args")

# step == 0 is a ValueError, with both literal and computed zeros.
expect(ValueError, lambda: range(0, 10, 0), "zero_step")
expect(ValueError, lambda: range(1, 2, int(0)), "zero_step_computed")

# Float arguments in any position raise TypeError (range needs ints).
expect(TypeError, lambda: range(1.5), "float_single")
expect(TypeError, lambda: range(0.0, 2, 1), "float_start")
expect(TypeError, lambda: range(1, 2.0, 1), "float_stop")
expect(TypeError, lambda: range(1, 2, 1.0), "float_step")
expect(TypeError, lambda: range(1e100, 1e101, 1e101), "float_huge")

# Non-numeric arguments raise TypeError in any position.
expect(TypeError, lambda: range("abc"), "str_single")
expect(TypeError, lambda: range(0, "spam"), "str_stop")
expect(TypeError, lambda: range(0, 42, "spam"), "str_step")
expect(TypeError, lambda: range([], 1, -1), "list_start")

# Index out of range raises IndexError for both positive and negative.
expect(IndexError, lambda: range(10)[20], "index_oob_pos")
expect(IndexError, lambda: range(10)[-11], "index_oob_neg")

# index() of a missing element raises ValueError.
expect(ValueError, lambda: range(10).index(99), "index_missing")
expect(ValueError, lambda: range(2).index(2), "index_missing_small")

# range is immutable: its attributes cannot be assigned.
def set_start():
    range(5).start = 10  # type: ignore[misc]


expect(AttributeError, set_start, "immutable_start")

# Ordering comparisons between ranges are unsupported.
expect(TypeError, lambda: range(0) < range(0), "order_lt")
expect(TypeError, lambda: range(0) >= range(0), "order_ge")

# The constructor's argument-count error messages are specific.
try:
    range()
except TypeError as e:
    assert "at least 1 argument" in str(e), str(e)
    print("msg_too_few: OK")
try:
    range(1, 2, 3, 4, 5, 6)
except TypeError as e:
    assert "at most 3 arguments" in str(e), str(e)
    print("msg_too_many: OK")

# A user __index__ that raises propagates that exception unchanged, both in
# the constructor and in slicing; one returning a non-int raises TypeError.
class IndexRaises:
    def __index__(self):
        raise RuntimeError("boom")


class IndexBadType:
    def __index__(self):
        return "not a number"


expect(RuntimeError, lambda: range(IndexRaises()), "ctor_index_raises")
expect(TypeError, lambda: range(IndexBadType()), "ctor_index_bad_type")
expect(RuntimeError, lambda: range(0, 10)[: IndexRaises()], "slice_index_raises")
expect(TypeError, lambda: range(0, 10)[: IndexBadType()], "slice_index_bad_type")

print("errors OK")
