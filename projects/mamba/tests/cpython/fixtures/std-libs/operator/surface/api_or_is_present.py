# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_or_is_present"
# subject = "operator.or_"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.or_: api_or_is_present (surface)."""
import operator

assert hasattr(operator, "or_")
print("api_or_is_present OK")
