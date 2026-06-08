# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "round_half_up_exists"
# subject = "decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal: round_half_up_exists (surface)."""
import decimal

assert hasattr(decimal, "ROUND_HALF_UP")
print("round_half_up_exists OK")
