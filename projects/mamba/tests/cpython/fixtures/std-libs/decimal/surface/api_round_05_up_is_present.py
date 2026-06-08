# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_round_05_up_is_present"
# subject = "decimal.ROUND_05UP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.ROUND_05UP: api_round_05_up_is_present (surface)."""
import decimal

assert hasattr(decimal, "ROUND_05UP")
print("api_round_05_up_is_present OK")
