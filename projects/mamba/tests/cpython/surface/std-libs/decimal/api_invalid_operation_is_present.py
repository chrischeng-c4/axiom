# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_invalid_operation_is_present"
# subject = "decimal.InvalidOperation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.InvalidOperation: api_invalid_operation_is_present (surface)."""
import decimal

assert hasattr(decimal, "InvalidOperation")
print("api_invalid_operation_is_present OK")
