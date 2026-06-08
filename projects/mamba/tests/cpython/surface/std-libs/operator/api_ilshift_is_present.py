# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_ilshift_is_present"
# subject = "operator.ilshift"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.ilshift: api_ilshift_is_present (surface)."""
import operator

assert hasattr(operator, "ilshift")
print("api_ilshift_is_present OK")
