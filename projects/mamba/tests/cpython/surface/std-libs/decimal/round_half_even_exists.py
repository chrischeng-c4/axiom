# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "round_half_even_exists"
# subject = "decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal: round_half_even_exists (surface)."""
import decimal

assert hasattr(decimal, "ROUND_HALF_EVEN")
print("round_half_even_exists OK")
