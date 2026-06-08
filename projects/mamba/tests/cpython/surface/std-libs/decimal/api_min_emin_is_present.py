# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_min_emin_is_present"
# subject = "decimal.MIN_EMIN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.MIN_EMIN: api_min_emin_is_present (surface)."""
import decimal

assert hasattr(decimal, "MIN_EMIN")
print("api_min_emin_is_present OK")
