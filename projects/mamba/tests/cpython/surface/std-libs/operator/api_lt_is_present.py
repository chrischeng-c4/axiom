# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_lt_is_present"
# subject = "operator.lt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.lt: api_lt_is_present (surface)."""
import operator

assert hasattr(operator, "lt")
print("api_lt_is_present OK")
