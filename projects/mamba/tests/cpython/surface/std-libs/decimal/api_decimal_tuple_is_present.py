# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_decimal_tuple_is_present"
# subject = "decimal.DecimalTuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.DecimalTuple: api_decimal_tuple_is_present (surface)."""
import decimal

assert hasattr(decimal, "DecimalTuple")
print("api_decimal_tuple_is_present OK")
