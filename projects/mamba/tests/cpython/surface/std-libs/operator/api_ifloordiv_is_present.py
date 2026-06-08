# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_ifloordiv_is_present"
# subject = "operator.ifloordiv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.ifloordiv: api_ifloordiv_is_present (surface)."""
import operator

assert hasattr(operator, "ifloordiv")
print("api_ifloordiv_is_present OK")
