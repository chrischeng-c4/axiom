# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_have_contextvar_is_present"
# subject = "decimal.HAVE_CONTEXTVAR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.HAVE_CONTEXTVAR: api_have_contextvar_is_present (surface)."""
import decimal

assert hasattr(decimal, "HAVE_CONTEXTVAR")
print("api_have_contextvar_is_present OK")
