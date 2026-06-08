# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_eq_is_present"
# subject = "operator.eq"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.eq: api_eq_is_present (surface)."""
import operator

assert hasattr(operator, "eq")
print("api_eq_is_present OK")
