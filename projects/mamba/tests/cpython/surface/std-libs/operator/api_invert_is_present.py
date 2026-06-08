# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_invert_is_present"
# subject = "operator.invert"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.invert: api_invert_is_present (surface)."""
import operator

assert hasattr(operator, "invert")
print("api_invert_is_present OK")
