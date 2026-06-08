# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_is_not_is_present"
# subject = "operator.is_not"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.is_not: api_is_not_is_present (surface)."""
import operator

assert hasattr(operator, "is_not")
print("api_is_not_is_present OK")
