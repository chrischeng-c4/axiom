# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_max_prec_is_present"
# subject = "decimal.MAX_PREC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.MAX_PREC: api_max_prec_is_present (surface)."""
import decimal

assert hasattr(decimal, "MAX_PREC")
print("api_max_prec_is_present OK")
