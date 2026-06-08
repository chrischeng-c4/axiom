# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "immutable_copy_returns_same"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: Decimal is immutable: copy.copy and copy.deepcopy return the same object"""
import copy
from decimal import Decimal

# Decimal is immutable: copy and deepcopy return the same object.
d = Decimal("43.24")
assert copy.copy(d) is d, "copy.copy returns same object"
assert copy.deepcopy(d) is d, "copy.deepcopy returns same object"

print("immutable_copy_returns_same OK")
