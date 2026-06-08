# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_ne_is_present"
# subject = "operator.ne"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.ne: api_ne_is_present (surface)."""
import operator

assert hasattr(operator, "ne")
print("api_ne_is_present OK")
