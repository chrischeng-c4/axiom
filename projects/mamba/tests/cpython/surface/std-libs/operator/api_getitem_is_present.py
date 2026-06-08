# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_getitem_is_present"
# subject = "operator.getitem"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.getitem: api_getitem_is_present (surface)."""
import operator

assert hasattr(operator, "getitem")
print("api_getitem_is_present OK")
