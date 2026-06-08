# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "copy_atomic_returns_identity"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: copy.copy of immutable atoms (None, ints, floats, str, bytes, range, slice, frozenset, type, function code, empty/immutable tuples) returns the very same object"""
import copy


def f():
    return None


class NewStyle:
    pass


# For these immutable, non-recursive values copy.copy returns the original
# object by identity — no new object is built.
atoms = [
    None, ..., NotImplemented, 42, 2 ** 100, 3.14, True, False, 1j,
    "hello", "héllo", b"world", bytes(range(8)),
    f.__code__, range(10), slice(1, 10, 2),
    NewStyle, max, property(), frozenset({1, 2, 3}), frozenset(),
    (), (1, 2, 3),
]
for x in atoms:
    assert copy.copy(x) is x, f"copy.copy should return identity for {x!r}"

print("copy_atomic_returns_identity OK")
