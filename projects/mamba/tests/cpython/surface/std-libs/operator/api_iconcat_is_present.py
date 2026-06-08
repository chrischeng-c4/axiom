# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_iconcat_is_present"
# subject = "operator.iconcat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.iconcat: api_iconcat_is_present (surface)."""
import operator

assert hasattr(operator, "iconcat")
print("api_iconcat_is_present OK")
