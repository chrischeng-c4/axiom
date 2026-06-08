# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_and_is_present"
# subject = "operator.and_"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.and_: api_and_is_present (surface)."""
import operator

assert hasattr(operator, "and_")
print("api_and_is_present OK")
