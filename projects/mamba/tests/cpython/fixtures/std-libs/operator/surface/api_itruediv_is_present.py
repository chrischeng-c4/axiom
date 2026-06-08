# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_itruediv_is_present"
# subject = "operator.itruediv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.itruediv: api_itruediv_is_present (surface)."""
import operator

assert hasattr(operator, "itruediv")
print("api_itruediv_is_present OK")
