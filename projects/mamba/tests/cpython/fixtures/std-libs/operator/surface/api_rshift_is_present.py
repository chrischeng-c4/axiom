# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_rshift_is_present"
# subject = "operator.rshift"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.rshift: api_rshift_is_present (surface)."""
import operator

assert hasattr(operator, "rshift")
print("api_rshift_is_present OK")
