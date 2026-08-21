# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "unary_operators"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: unary +, -, and abs() behave as expected on Decimal(45)/Decimal(-45)"""
from decimal import Decimal

D = Decimal
assert +D(45) == D(45), "unary plus"
assert -D(45) == D(-45), "unary minus"
assert abs(D(-45)) == D(45), "abs"

print("unary_operators OK")
