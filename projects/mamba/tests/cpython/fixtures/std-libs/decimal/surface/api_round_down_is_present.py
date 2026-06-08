# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_round_down_is_present"
# subject = "decimal.ROUND_DOWN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.ROUND_DOWN: api_round_down_is_present (surface)."""
import decimal

assert hasattr(decimal, "ROUND_DOWN")
print("api_round_down_is_present OK")
