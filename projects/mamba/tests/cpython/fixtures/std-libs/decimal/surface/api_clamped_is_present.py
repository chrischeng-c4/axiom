# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_clamped_is_present"
# subject = "decimal.Clamped"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.Clamped: api_clamped_is_present (surface)."""
import decimal

assert hasattr(decimal, "Clamped")
print("api_clamped_is_present OK")
