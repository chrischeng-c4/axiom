# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "int_and_trunc_toward_zero"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: int() and math.trunc() truncate toward zero and agree with float's int(): int(Decimal('3.99'))==3, int(Decimal('-3.99'))==-3"""
import math
from decimal import Decimal

D = Decimal
# int() and math.trunc() truncate toward zero and agree with float's int().
for x in (-250, -1, 0, 1, 137, 249):
    s = "%0.2f" % (x / 100.0)
    assert int(D(s)) == int(float(s)), f"int(Decimal({s!r}))"
    assert math.trunc(D(s)) == int(D(s)), f"trunc(Decimal({s!r}))"
assert int(D("3.99")) == 3, "int truncates fraction"
assert int(D("-3.99")) == -3, "int truncates toward zero"

print("int_and_trunc_toward_zero OK")
