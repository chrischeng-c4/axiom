# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_localcontext_is_present"
# subject = "decimal.localcontext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.localcontext: api_localcontext_is_present (surface)."""
import decimal

assert hasattr(decimal, "localcontext")
print("api_localcontext_is_present OK")
