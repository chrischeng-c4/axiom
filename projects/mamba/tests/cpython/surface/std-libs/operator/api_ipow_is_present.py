# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_ipow_is_present"
# subject = "operator.ipow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.ipow: api_ipow_is_present (surface)."""
import operator

assert hasattr(operator, "ipow")
print("api_ipow_is_present OK")
