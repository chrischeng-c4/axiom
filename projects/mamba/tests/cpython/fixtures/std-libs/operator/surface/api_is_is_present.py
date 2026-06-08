# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_is_is_present"
# subject = "operator.is_"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.is_: api_is_is_present (surface)."""
import operator

assert hasattr(operator, "is_")
print("api_is_is_present OK")
