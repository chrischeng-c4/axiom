# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "round_down_exists"
# subject = "decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal: round_down_exists (surface)."""
import decimal

assert hasattr(decimal, "ROUND_DOWN")
print("round_down_exists OK")
