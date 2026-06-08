# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_sub_is_present"
# subject = "operator.sub"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.sub: api_sub_is_present (surface)."""
import operator

assert hasattr(operator, "sub")
print("api_sub_is_present OK")
