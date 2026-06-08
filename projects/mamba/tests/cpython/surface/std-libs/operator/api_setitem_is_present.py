# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_setitem_is_present"
# subject = "operator.setitem"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.setitem: api_setitem_is_present (surface)."""
import operator

assert hasattr(operator, "setitem")
print("api_setitem_is_present OK")
