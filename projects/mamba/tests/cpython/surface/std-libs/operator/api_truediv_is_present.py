# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_truediv_is_present"
# subject = "operator.truediv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.truediv: api_truediv_is_present (surface)."""
import operator

assert hasattr(operator, "truediv")
print("api_truediv_is_present OK")
