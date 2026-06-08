# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_abs_is_present"
# subject = "operator.abs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.abs: api_abs_is_present (surface)."""
import operator

assert hasattr(operator, "abs")
print("api_abs_is_present OK")
