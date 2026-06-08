# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_invalid_context_is_present"
# subject = "decimal.InvalidContext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.InvalidContext: api_invalid_context_is_present (surface)."""
import decimal

assert hasattr(decimal, "InvalidContext")
print("api_invalid_context_is_present OK")
