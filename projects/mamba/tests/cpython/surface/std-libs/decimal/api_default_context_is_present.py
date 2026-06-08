# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_default_context_is_present"
# subject = "decimal.DefaultContext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.DefaultContext: api_default_context_is_present (surface)."""
import decimal

assert hasattr(decimal, "DefaultContext")
print("api_default_context_is_present OK")
