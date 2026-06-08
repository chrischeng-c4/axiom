# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "cross_type_vs_complex"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal vs complex: equality holds only when the imaginary part is zero, and ordering against complex returns NotImplemented"""
from decimal import Decimal

db = Decimal("3.0")
# Equality works only when the imaginary part is zero; ordering against a
# complex is undefined and returns NotImplemented.
assert db == 3.0 + 0j, "Decimal == complex (real)"
assert db != 3.0 + 1j, "Decimal != complex (imag)"
assert db.__lt__(3.0 + 0j) is NotImplemented, "ordering vs complex is NotImplemented"

print("cross_type_vs_complex OK")
