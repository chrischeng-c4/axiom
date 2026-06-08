# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_division_by_zero_is_present"
# subject = "decimal.DivisionByZero"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.DivisionByZero: api_division_by_zero_is_present (surface)."""
import decimal

assert hasattr(decimal, "DivisionByZero")
print("api_division_by_zero_is_present OK")
