# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "nan_equality_is_false"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: comparing Decimal('NaN') == Decimal('NaN') returns False (not a raise, not NaN), and != is always True"""
from decimal import Decimal

# Comparing Decimal NaN with == returns False, not NaN and not a raise; != is
# always True.
assert (Decimal("NaN") == Decimal("NaN")) is False, "NaN == NaN is False"
assert Decimal("NaN") != Decimal("NaN"), "NaN != NaN is True"

print("nan_equality_is_false OK")
