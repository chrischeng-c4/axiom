# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_decimal_exception_is_present"
# subject = "decimal.DecimalException"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.DecimalException: api_decimal_exception_is_present (surface)."""
import decimal

assert hasattr(decimal, "DecimalException")
print("api_decimal_exception_is_present OK")
