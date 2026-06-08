# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_min_etiny_is_present"
# subject = "decimal.MIN_ETINY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.MIN_ETINY: api_min_etiny_is_present (surface)."""
import decimal

assert hasattr(decimal, "MIN_ETINY")
print("api_min_etiny_is_present OK")
