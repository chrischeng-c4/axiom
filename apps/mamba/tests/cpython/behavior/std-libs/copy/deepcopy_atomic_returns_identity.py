# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_atomic_returns_identity"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of immutable atoms and tuples-of-immutables returns the same object, but a tuple holding a mutable member is rebuilt with an independent inner copy"""
import copy


def f():
    return None


class NewStyle:
    pass


# deepcopy returns identity for immutable atoms, plus tuples whose members are
# themselves all immutable (no deep work needed).
atoms = [
    None, ..., NotImplemented, 42, 2 ** 100, 3.14, True, False, 1j,
    "hello", b"world", f.__code__, NewStyle, range(10), max, property(),
    (), ((1, 2), 3),
]
for x in atoms:
    assert copy.deepcopy(x) is x, f"deepcopy should return identity for {x!r}"

# A tuple holding a mutable member is NOT atomic: deepcopy rebuilds it so the
# inner mutable is independent.
nested = ([1, 2], 3)
deep = copy.deepcopy(nested)
assert deep == nested and deep is not nested, "deepcopy of nested tuple is new but equal"
assert deep[0] is not nested[0], "deepcopy copies the inner mutable list"

print("deepcopy_atomic_returns_identity OK")
