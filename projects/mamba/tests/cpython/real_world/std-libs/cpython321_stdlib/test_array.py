# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_array"
# subject = "cpython321.test_array"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_array.py"
# status = "filled"
# ///
"""cpython321.test_array: execute CPython 3.12 seed test_array"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: array.array — construction with a list initializer, typecode and
# itemsize attribute readback for 'i' (4-byte signed int) and 'd' (8-byte
# double), append() / extend() growing the buffer, and tolist() round-trip.
# Intentionally NOT exercised on mamba today (tracked separately):
#   * len(array.array(...))    — returns 0
#   * a[idx]                    — returns None
#   * list(array.array(...))    — TypeError: object is not iterable
#   * fromlist / frombytes / tobytes round-trips
#   * array.array typecodes b/u/B/H/I/l (only 'i' and 'd' exercised here)
import array

_ledger: list[int] = []

# 'i' typecode = signed int; itemsize == 4
_i = array.array("i", [10, 20, 30])
assert _i.typecode == "i", f"array('i').typecode == 'i', got {_i.typecode!r}"
_ledger.append(1)

assert _i.itemsize - 4 == 0, f"array('i').itemsize == 4, got {_i.itemsize!r}"
_ledger.append(1)

# tolist round-trips the original list
assert _i.tolist() == [10, 20, 30], (
    f"array('i', [10,20,30]).tolist() round-trips, got {_i.tolist()!r}"
)
_ledger.append(1)

# append() grows the array in order
_i.append(40)
assert _i.tolist() == [10, 20, 30, 40], (
    f"array.append appends in order, got {_i.tolist()!r}"
)
_ledger.append(1)

# extend() with a list appends each element
_i.extend([50, 60])
assert _i.tolist() == [10, 20, 30, 40, 50, 60], (
    f"array.extend appends each element, got {_i.tolist()!r}"
)
_ledger.append(1)

# An array constructed with an empty initializer + append produces the
# expected sequence
_e = array.array("i")
_e.append(1)
_e.append(2)
_e.append(3)
assert _e.tolist() == [1, 2, 3], (
    f"empty array + 3 appends = [1,2,3], got {_e.tolist()!r}"
)
_ledger.append(1)

# 'd' typecode = double-precision float; itemsize == 8
_d = array.array("d", [1.5, 2.5])
assert _d.typecode == "d", f"array('d').typecode == 'd', got {_d.typecode!r}"
_ledger.append(1)

assert _d.itemsize - 8 == 0, f"array('d').itemsize == 8, got {_d.itemsize!r}"
_ledger.append(1)

assert _d.tolist() == [1.5, 2.5], (
    f"array('d', [1.5, 2.5]).tolist() round-trips, got {_d.tolist()!r}"
)
_ledger.append(1)

# 'i' and 'd' have distinct itemsizes
assert _i.itemsize != _d.itemsize, (
    f"itemsize('i') != itemsize('d'), got {_i.itemsize} vs {_d.itemsize}"
)
_ledger.append(1)

# Independent arrays don't share storage (mutating one doesn't move the other)
_i2 = array.array("i", [100, 200])
_i2.append(300)
assert _i.tolist() == [10, 20, 30, 40, 50, 60], (
    f"_i unchanged after _i2.append, got {_i.tolist()!r}"
)
_ledger.append(1)

assert _i2.tolist() == [100, 200, 300], (
    f"_i2 has its own buffer, got {_i2.tolist()!r}"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_array {sum(_ledger)} asserts")
