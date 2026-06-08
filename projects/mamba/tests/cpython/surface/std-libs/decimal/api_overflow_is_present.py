# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_overflow_is_present"
# subject = "decimal.Overflow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.Overflow: api_overflow_is_present (surface)."""
import decimal

assert hasattr(decimal, "Overflow")
print("api_overflow_is_present OK")
