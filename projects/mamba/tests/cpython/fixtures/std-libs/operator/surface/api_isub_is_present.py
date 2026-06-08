# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_isub_is_present"
# subject = "operator.isub"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.isub: api_isub_is_present (surface)."""
import operator

assert hasattr(operator, "isub")
print("api_isub_is_present OK")
