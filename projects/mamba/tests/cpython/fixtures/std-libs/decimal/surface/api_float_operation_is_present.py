# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_float_operation_is_present"
# subject = "decimal.FloatOperation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.FloatOperation: api_float_operation_is_present (surface)."""
import decimal

assert hasattr(decimal, "FloatOperation")
print("api_float_operation_is_present OK")
