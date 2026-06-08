# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_iadd_is_present"
# subject = "operator.iadd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.iadd: api_iadd_is_present (surface)."""
import operator

assert hasattr(operator, "iadd")
print("api_iadd_is_present OK")
