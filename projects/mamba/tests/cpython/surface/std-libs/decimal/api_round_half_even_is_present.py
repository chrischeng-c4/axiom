# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_round_half_even_is_present"
# subject = "decimal.ROUND_HALF_EVEN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.ROUND_HALF_EVEN: api_round_half_even_is_present (surface)."""
import decimal

assert hasattr(decimal, "ROUND_HALF_EVEN")
print("api_round_half_even_is_present OK")
