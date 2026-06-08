# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_context_is_present"
# subject = "decimal.Context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.Context: api_context_is_present (surface)."""
import decimal

assert hasattr(decimal, "Context")
print("api_context_is_present OK")
