# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "round_ceiling_exists"
# subject = "decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal: round_ceiling_exists (surface)."""
import decimal

assert hasattr(decimal, "ROUND_CEILING")
print("round_ceiling_exists OK")
