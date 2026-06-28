# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""slice: documented exception paths (CPython 3.12 oracle)."""


# slice() with zero args raises TypeError (needs at least the stop arg).
try:
    slice()  # type: ignore[call-overload]
    print("zero_args: no_raise")
except TypeError as e:
    print("zero_args:", type(e).__name__, str(e)[:60])


# slice() with more than three args raises TypeError.
try:
    slice(1, 2, 3, 4)  # type: ignore[call-overload]
    print("four_args: no_raise")
except TypeError as e:
    print("four_args:", type(e).__name__, str(e)[:60])


# slice.indices() with a non-int length raises TypeError.
try:
    slice(0, 5).indices("not int")  # type: ignore[arg-type]
    print("bad_length: no_raise")
except TypeError as e:
    print("bad_length:", type(e).__name__, str(e)[:60])


for name, call in (
    ("indices_zero_args", lambda: slice(None).indices()),  # type: ignore[call-arg]
    ("indices_two_args", lambda: slice(None).indices(1, 2)),  # type: ignore[call-arg]
):
    try:
        call()
        print(f"{name}: no_raise")
    except TypeError as e:
        print(f"{name}:", type(e).__name__, str(e)[:60])


# slice is immutable; assigning to a field raises AttributeError.
s = slice(0, 10, 2)
try:
    s.start = 5  # type: ignore[misc]
    print("set_start: no_raise")
except AttributeError as e:
    print("set_start:", type(e).__name__, str(e)[:60])


# A slice with an unhashable member is itself unhashable.
try:
    hash(slice(1, 2, []))  # type: ignore[arg-type]
    print("unhashable_member: no_raise")
except TypeError as e:
    print("unhashable_member:", type(e).__name__, str(e)[:60])


# Slicing a sequence with step 0 raises ValueError on use.
for name, seq in (
    ("list", [1, 2, 3, 4, 5]),
    ("tuple", (1, 2, 3, 4, 5)),
    ("str", "abcde"),
    ("bytes", b"abcde"),
):
    try:
        seq[::0]
        print(f"step_zero_{name}: no_raise")
    except ValueError as e:
        print(f"step_zero_{name}:", type(e).__name__, str(e)[:60])


# indices() accepts SupportsIndex for start/stop/step/length.
class Indexable:
    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value


print("indices_indexable:", slice(Indexable(1), Indexable(4), Indexable(2)).indices(Indexable(6)))


# Bad slice components and bad length preserve CPython exception classes.
for name, bad in (
    ("bad_start", slice(0.0, 2)),
    ("bad_stop", slice(0, 2.0)),
    ("bad_step", slice(0, 2, 1.0)),
):
    try:
        bad.indices(5)
        print(f"{name}: no_raise")
    except TypeError as e:
        print(f"{name}:", type(e).__name__, str(e)[:60])

try:
    slice(0, 10, 1).indices(-1)
    print("negative_length: no_raise")
except ValueError as e:
    print("negative_length:", type(e).__name__, str(e)[:60])

try:
    slice(0, 10, 0).indices(5)
    print("indices_step_zero: no_raise")
except ValueError as e:
    print("indices_step_zero:", type(e).__name__, str(e)[:60])


# Happy path: indices() with a valid length, and field readback.
print("indices:", slice(1, 10, 2).indices(5))
print("start:", s.start, "stop:", s.stop, "step:", s.step)
