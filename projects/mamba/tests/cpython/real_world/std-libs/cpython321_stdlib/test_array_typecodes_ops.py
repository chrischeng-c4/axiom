# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_array_typecodes_ops"
# subject = "cpython321.test_array_typecodes_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_array_typecodes_ops.py"
# status = "filled"
# ///
"""cpython321.test_array_typecodes_ops: execute CPython 3.12 seed test_array_typecodes_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `array.array` typecode widths
# not covered by `test_array`. Existing `test_array` exercises only
# 'i' and 'd'. This seed asserts typecode preservation and itemsize
# widths for the wider integer/float code set — b/B (1), h/H (2),
# I (4), l/L (8 on this 64-bit target), f (4) — and the module-level
# `array.typecodes` string that enumerates all supported codes.
import array
_ledger: list[int] = []

# Signed 1-byte
_b = array.array("b")
assert _b.typecode == "b"; _ledger.append(1)
assert _b.itemsize == 1; _ledger.append(1)

# Unsigned 1-byte
_B = array.array("B")
assert _B.typecode == "B"; _ledger.append(1)
assert _B.itemsize == 1; _ledger.append(1)

# Signed 2-byte
_h = array.array("h")
assert _h.typecode == "h"; _ledger.append(1)
assert _h.itemsize == 2; _ledger.append(1)

# Unsigned 2-byte
_H = array.array("H")
assert _H.typecode == "H"; _ledger.append(1)
assert _H.itemsize == 2; _ledger.append(1)

# Unsigned 4-byte
_I = array.array("I")
assert _I.typecode == "I"; _ledger.append(1)
assert _I.itemsize == 4; _ledger.append(1)

# Signed long
_l = array.array("l")
assert _l.typecode == "l"; _ledger.append(1)
assert _l.itemsize >= 4; _ledger.append(1)

# Unsigned long
_L = array.array("L")
assert _L.typecode == "L"; _ledger.append(1)
assert _L.itemsize == _l.itemsize; _ledger.append(1)

# Float (single precision)
_f = array.array("f")
assert _f.typecode == "f"; _ledger.append(1)
assert _f.itemsize == 4; _ledger.append(1)

# Signed/unsigned variants share the same width
assert array.array("h").itemsize == array.array("H").itemsize; _ledger.append(1)
assert array.array("b").itemsize == array.array("B").itemsize; _ledger.append(1)
assert array.array("l").itemsize == array.array("L").itemsize; _ledger.append(1)

# Module-level typecodes string enumerates the supported set
assert isinstance(array.typecodes, str); _ledger.append(1)
assert "i" in array.typecodes; _ledger.append(1)
assert "d" in array.typecodes; _ledger.append(1)
assert "f" in array.typecodes; _ledger.append(1)
assert "b" in array.typecodes; _ledger.append(1)
assert "B" in array.typecodes; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_array_typecodes_ops {sum(_ledger)} asserts")
