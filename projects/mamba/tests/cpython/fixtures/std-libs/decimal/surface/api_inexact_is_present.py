# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_inexact_is_present"
# subject = "decimal.Inexact"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.Inexact: api_inexact_is_present (surface)."""
import decimal

assert hasattr(decimal, "Inexact")
print("api_inexact_is_present OK")
