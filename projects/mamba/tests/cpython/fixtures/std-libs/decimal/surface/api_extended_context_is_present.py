# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_extended_context_is_present"
# subject = "decimal.ExtendedContext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.ExtendedContext: api_extended_context_is_present (surface)."""
import decimal

assert hasattr(decimal, "ExtendedContext")
print("api_extended_context_is_present OK")
