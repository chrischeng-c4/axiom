# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "binary_methods_accept_int"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: binary Decimal methods accept int operands as if they were Decimals: compare/max/quantize/scaleb/fma agree on int vs Decimal arguments"""
from decimal import Decimal

D = Decimal
# Binary Decimal methods accept int operands as if they were Decimals.
assert D(4).compare(3) == D(4).compare(D(3)), "compare int operand"
assert D(567).max(123) == D(567).max(D(123)), "max int operand"
assert D(1234).quantize(100) == D(1234).quantize(D(100)), "quantize int operand"
assert D("9.123").scaleb(-100) == D("9.123").scaleb(D(-100)), "scaleb int operand"
assert D(-12).fma(45, 67) == D(-12).fma(D(45), D(67)), "fma int operands"

print("binary_methods_accept_int OK")
