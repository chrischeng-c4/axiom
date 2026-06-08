# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""float.__ceil__ / __floor__ / __trunc__ / is_integer (CPython 3.12 oracle)."""

# __ceil__ and __floor__ return *int*, not float, for finite inputs.
assert isinstance((0.5).__ceil__(), int)
assert isinstance((0.5).__floor__(), int)

# Ceiling rounds toward +inf.
assert (0.5).__ceil__() == 1
assert (1.0).__ceil__() == 1
assert (1.5).__ceil__() == 2
assert (-0.5).__ceil__() == 0
assert (-1.5).__ceil__() == -1

# Floor rounds toward -inf.
assert (0.5).__floor__() == 0
assert (1.5).__floor__() == 1
assert (-0.5).__floor__() == -1
assert (-1.5).__floor__() == -2

# __trunc__ rounds toward zero and also yields int.
assert isinstance((2.9).__trunc__(), int)
assert (2.9).__trunc__() == 2
assert (-2.9).__trunc__() == -2

# is_integer is True exactly for whole-valued finite floats.
assert (1.0).is_integer()
assert not (1.1).is_integer()
assert not float("nan").is_integer()
assert not float("inf").is_integer()

# ceil/floor of non-finite values raise (nan -> ValueError, inf -> Overflow).
for method in ("__ceil__", "__floor__"):
    try:
        getattr(float("nan"), method)()
        raise AssertionError("expected ValueError")
    except ValueError:
        pass
    for inf in (float("inf"), float("-inf")):
        try:
            getattr(inf, method)()
            raise AssertionError("expected OverflowError")
        except OverflowError:
            pass

print("int_dunders OK")
