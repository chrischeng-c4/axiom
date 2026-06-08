# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_not_is_present"
# subject = "operator.not_"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.not_: api_not_is_present (surface)."""
import operator

assert hasattr(operator, "not_")
print("api_not_is_present OK")
