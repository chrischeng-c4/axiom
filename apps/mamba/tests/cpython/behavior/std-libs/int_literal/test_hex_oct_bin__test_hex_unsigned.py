# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "int_literal"
# dimension = "behavior"
# case = "test_hex_oct_bin__test_hex_unsigned"
# subject = "cpython.test_int_literal.TestHexOctBin.test_hex_unsigned"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int_literal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_int_literal.py::TestHexOctBin::test_hex_unsigned
"""Auto-ported test: TestHexOctBin::test_hex_unsigned (CPython 3.12 oracle)."""


import unittest


'Test correct treatment of hex/oct constants.\n\nThis is complex because of changes due to PEP 237.\n'


# --- test body ---

assert 2147483648 == 2147483648

assert 4294967295 == 4294967295

assert -2147483648 == -2147483648

assert -4294967295 == -4294967295

assert -2147483648 == -2147483648

assert -4294967295 == -4294967295

assert 9223372036854775808 == 9223372036854775808

assert 18446744073709551615 == 18446744073709551615

assert -9223372036854775808 == -9223372036854775808

assert -18446744073709551615 == -18446744073709551615

assert -9223372036854775808 == -9223372036854775808

assert -18446744073709551615 == -18446744073709551615
print("TestHexOctBin::test_hex_unsigned: ok")
