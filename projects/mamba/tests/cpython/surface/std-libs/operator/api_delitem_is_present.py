# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_delitem_is_present"
# subject = "operator.delitem"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.delitem: api_delitem_is_present (surface)."""
import operator

assert hasattr(operator, "delitem")
print("api_delitem_is_present OK")
